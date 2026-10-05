//! Modal dialogs and their state.

use std::collections::BTreeSet;

use iced::widget::text_editor;

use floetask_application::TodoRef;
use floetask_domain::date::first_of_month;
use floetask_domain::listing::Attribute;
use floetask_domain::note::default_name;
use floetask_domain::{Date, NoteError, NoteName, RecurrenceUnit};

/// The date extensions that have a picker.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DateKey {
    Due,
    Threshold,
}

impl DateKey {
    pub fn key(self) -> &'static str {
        match self {
            DateKey::Due => "due",
            DateKey::Threshold => "t",
        }
    }
}

/// A month calendar for picking a date.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Calendar {
    pub key: DateKey,
    /// First day of the month on display.
    pub month: Date,
    pub selected: Option<Date>,
}

impl Calendar {
    pub fn new(key: DateKey, selected: Option<Date>, today: Date) -> Self {
        Self {
            key,
            month: first_of_month(selected.unwrap_or(today)),
            selected,
        }
    }

    pub fn shift_month(&mut self, months: i32) {
        self.month = floetask_domain::date::add_months(self.month, i64::from(months));
    }
}

/// The add/edit dialog.
#[derive(Debug)]
pub struct Editor {
    /// `None` when adding a new todo.
    pub target: Option<TodoRef>,
    pub content: text_editor::Content,
    pub suggestions: Vec<String>,
    pub selected_suggestion: usize,
    pub calendar: Option<Calendar>,
    pub recurrence_count: String,
    pub recurrence_unit: RecurrenceUnit,
    pub recurrence_strict: bool,
    pub pomodoros: String,
    /// The todo's notes in Markdown, saved to its `note:` file.
    pub note: text_editor::Content,
    /// The note text as read from disk, to tell whether the user changed it.
    /// Empty while the file is missing or still loading.
    pub note_loaded: String,
}

impl Editor {
    pub fn new(target: Option<TodoRef>, text: &str) -> Self {
        Self {
            target,
            content: text_editor::Content::with_text(text),
            suggestions: Vec::new(),
            selected_suggestion: 0,
            calendar: None,
            recurrence_count: "1".to_owned(),
            recurrence_unit: RecurrenceUnit::Week,
            recurrence_strict: false,
            pomodoros: "1".to_owned(),
            note: text_editor::Content::new(),
            note_loaded: String::new(),
        }
    }

    pub fn text(&self) -> String {
        let text = self.content.text();
        text.strip_suffix('\n').map(str::to_owned).unwrap_or(text)
    }

    pub fn note_text(&self) -> String {
        let text = self.note.text();
        text.strip_suffix('\n').map(str::to_owned).unwrap_or(text)
    }

    /// Shows a note read from disk, unless the user already started typing.
    pub fn load_note(&mut self, text: &str) {
        if self.note_text().is_empty() {
            let text = text.strip_suffix('\n').unwrap_or(text);
            self.note = text_editor::Content::with_text(text);
            self.note_loaded = text.to_owned();
        }
    }

    /// Whether saving should write the note file: the text differs from
    /// what was loaded. An empty note that was never written stays unwritten.
    pub fn note_changed(&self) -> bool {
        self.note_text() != self.note_loaded
    }

    /// The note file this todo links to: its `note:` value, or the name a
    /// new note would get. `None` when the `note:` value is invalid.
    pub fn note_target(&self) -> Option<Result<NoteName, NoteError>> {
        let todo = floetask_domain::Todo::from_user_text(&self.text());
        todo.note_name()
    }

    /// The name a new note gets from the todo text, e.g. `write-report.md`.
    pub fn new_note_name(&self) -> NoteName {
        default_name(&floetask_domain::Todo::from_user_text(&self.text()))
    }

    pub fn set_text(&mut self, text: &str) {
        self.content = text_editor::Content::with_text(text);
        self.content
            .perform(text_editor::Action::Move(text_editor::Motion::DocumentEnd));
        self.suggestions.clear();
    }

    /// Recomputes autocomplete suggestions for the token at the cursor.
    pub fn refresh_suggestions(&mut self, projects: &BTreeSet<String>, contexts: &BTreeSet<String>) {
        self.suggestions = self
            .typed_tag()
            .map(|tag| {
                let names = if tag.sigil == '+' { projects } else { contexts };
                crate::compose::suggestions(&tag.prefix, names)
            })
            .unwrap_or_default();
        self.selected_suggestion = 0;
    }

    pub fn typed_tag(&self) -> Option<crate::compose::TypedTag> {
        let cursor = self.content.cursor().position;
        let line = self.content.line(cursor.line)?;
        crate::compose::typed_tag(&line.text, cursor.line, cursor.column)
    }

    /// Inserts the suggestion at `index` in place of the typed tag.
    pub fn accept_suggestion(&mut self, index: usize) {
        let (Some(tag), Some(name)) = (self.typed_tag(), self.suggestions.get(index).cloned()) else {
            return;
        };
        let (text, line, column) = crate::compose::insert_suggestion(&self.text(), &tag, &name);
        self.content = text_editor::Content::with_text(&text);
        self.content.move_to(text_editor::Cursor {
            position: text_editor::Position { line, column },
            selection: None,
        });
        self.suggestions.clear();
    }
}

/// An action waiting for the user's confirmation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Pending {
    DeleteTodo(TodoRef),
    RemoveFile(usize),
    RemoveValue(Attribute, String),
    DeleteSavedFilter(usize),
}

#[derive(Debug)]
pub enum Dialog {
    Editor(Box<Editor>),
    Settings,
    Confirm {
        message: String,
        detail: Option<String>,
        pending: Pending,
    },
    Rename {
        attribute: Attribute,
        from: String,
        input: String,
    },
    RowDate {
        target: TodoRef,
        calendar: Calendar,
    },
    SaveFilter {
        name: String,
    },
    /// The active file's board columns, as keys, while being edited.
    BoardColumns {
        columns: Vec<String>,
        input: String,
    },
}
