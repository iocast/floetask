//! The add/edit dialog's state and the Markdown field for a todo's note,
//! which the note window uses too.

use std::collections::BTreeSet;
use std::path::PathBuf;

use iced::widget::text_editor;
use iced_markdown_editor::{Action, EditorState, EditorStyle, Event, Output, Paper, ZoomSettings};

use floetask_application::TodoRef;
use floetask_domain::note::default_name;
use floetask_domain::{NoteError, NoteLink, NoteName, RecurrenceUnit};

use super::DatePicker;
use crate::compose::FieldText;
use crate::theme::{self, Colors};

/// The add/edit dialog.
#[derive(Debug)]
pub struct Editor {
    /// The todo file the todo is in, or is added to. Its notes live next
    /// to it.
    pub file: PathBuf,
    /// `None` when adding a new todo.
    pub target: Option<TodoRef>,
    /// The text field, which hides tokens shown elsewhere in the dialog.
    pub content: text_editor::Content,
    /// The whole todo text behind the field, see [`FieldText`].
    field: FieldText,
    pub suggestions: Vec<String>,
    pub selected_suggestion: usize,
    pub calendar: Option<DatePicker>,
    pub recurrence_count: String,
    pub recurrence_unit: RecurrenceUnit,
    pub recurrence_strict: bool,
    pub pomodoros: String,
    /// The todo's notes in Markdown, saved to its `note:` file.
    pub note: NoteField,
    /// Whether the notes editor shows its formatting toolbar. Hidden at
    /// first, so the note reads as a plain field in the dialog.
    pub note_toolbar: bool,
    /// The resize in progress, from a press on an edge until the release.
    pub resize: Option<Resize>,
    /// A file the user picked to link that lies outside the notes folder,
    /// while the dialog asks whether to move it in or keep it where it is.
    pub pending_link: Option<PathBuf>,
}

impl Editor {
    pub fn new(file: PathBuf, target: Option<TodoRef>, text: &str, colors: Colors) -> Self {
        let field = FieldText::new(text);
        let todo = floetask_domain::Todo::from_user_text(text);
        let recurrence = todo.recurrence();
        Self {
            file,
            target,
            content: text_editor::Content::with_text(&field.visible),
            field,
            suggestions: Vec::new(),
            selected_suggestion: 0,
            calendar: None,
            // 0 shows that the todo does not repeat or has no pomodoros.
            recurrence_count: recurrence.map_or(0, |r| r.count).to_string(),
            recurrence_unit: recurrence.map_or(RecurrenceUnit::Week, |r| r.unit),
            recurrence_strict: recurrence.is_some_and(|r| r.strict),
            pomodoros: todo.pomodoros().unwrap_or(0).to_string(),
            note: NoteField::new(colors),
            note_toolbar: false,
            resize: None,
            pending_link: None,
        }
    }

    /// The whole todo text, hidden tokens included.
    pub fn text(&self) -> String {
        self.field.join(&self.visible_text())
    }

    /// The text as the field shows it.
    pub fn visible_text(&self) -> String {
        let text = self.content.text();
        text.strip_suffix('\n').map(str::to_owned).unwrap_or(text)
    }

    pub fn note_text(&self) -> String {
        self.note.text().to_owned()
    }

    /// Shows a note read from disk, unless the user already started typing.
    pub fn load_note(&mut self, text: &str) {
        if self.note.text().trim().is_empty() {
            self.note.show(text);
        }
    }

    /// Whether saving should write the note file: the text differs from
    /// what was loaded. An empty note that was never written stays unwritten.
    pub fn note_changed(&self) -> bool {
        self.note.changed()
    }

    /// The note this todo links to, from its `note:` value; `None` without one.
    pub fn note_target(&self) -> Option<Result<NoteLink, NoteError>> {
        let todo = floetask_domain::Todo::from_user_text(&self.text());
        todo.note_link()
    }

    /// The name a new note gets from the todo text, e.g. `write-report.md`.
    pub fn new_note_name(&self) -> NoteName {
        default_name(&floetask_domain::Todo::from_user_text(&self.text()))
    }

    pub fn set_text(&mut self, text: &str) {
        self.field = FieldText::new(text);
        self.content = text_editor::Content::with_text(&self.field.visible);
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
        let (text, line, column) = crate::compose::insert_suggestion(&self.visible_text(), &tag, &name);
        self.content = text_editor::Content::with_text(&text);
        self.content.move_to(text_editor::Cursor {
            position: text_editor::Position { line, column },
            selection: None,
        });
        self.suggestions.clear();
    }
}

/// A resize of the dialog: which edge is dragged and the size it started
/// at. The first cursor position comes with the first move, as a press does
/// not carry one.
#[derive(Debug, Clone, Copy)]
pub struct Resize {
    pub direction: iced::window::Direction,
    pub start: iced::Size,
    pub from: Option<iced::Point>,
}

/// The smallest the dialog gets: room for the text, the notes and the
/// details panel side by side.
pub const MIN_DIALOG: iced::Size = iced::Size::new(640.0, 460.0);

impl Resize {
    /// The size after the cursor moved by `delta` since the first move,
    /// between [`MIN_DIALOG`] and `max`. The card is centred, so it grows
    /// on both sides: twice the distance keeps the dragged edge under the
    /// cursor.
    pub fn size_for(&self, delta: iced::Vector, max: iced::Size) -> iced::Size {
        use iced::window::Direction as D;
        let (x, y) = match self.direction {
            D::North => (0.0, -1.0),
            D::South => (0.0, 1.0),
            D::East => (1.0, 0.0),
            D::West => (-1.0, 0.0),
            D::NorthEast => (1.0, -1.0),
            D::NorthWest => (-1.0, -1.0),
            D::SouthEast => (1.0, 1.0),
            D::SouthWest => (-1.0, 1.0),
        };
        let clamp = |value: f32, min: f32, max: f32| value.min(max).max(min);
        iced::Size::new(
            clamp(self.start.width + 2.0 * x * delta.x, MIN_DIALOG.width, max.width),
            clamp(self.start.height + 2.0 * y * delta.y, MIN_DIALOG.height, max.height),
        )
    }
}

/// The Markdown editor for a todo's note. It remembers the Markdown it was
/// loaded with, so saving can tell whether the user changed the note.
pub struct NoteField {
    state: EditorState,
    /// The editor's Markdown, rebuilt on each change instead of in `view`.
    markdown: String,
    /// The Markdown as loaded; empty while the file is missing or loading.
    /// It comes from the editor's own serialiser like `markdown`, so a note
    /// the user did not touch is never rewritten in the editor's format.
    loaded: String,
}

impl NoteField {
    pub(crate) fn new(colors: Colors) -> Self {
        let state = EditorState::new()
            .paper(Paper::fill())
            .zoom_settings(ZoomSettings::disabled())
            .style(theme::markdown_editor(colors))
            .language(crate::i18n::language().editor());
        let markdown = state.markdown();
        Self {
            state,
            loaded: markdown.clone(),
            markdown,
        }
    }

    pub fn state(&self) -> &EditorState {
        &self.state
    }

    /// Forwards a widget action, keeping the Markdown up to date.
    pub fn update(&mut self, action: Action) -> Output {
        let output = self.state.update(action);
        if output.events.contains(&Event::Changed) {
            self.markdown = self.state.markdown();
        }
        output
    }

    /// The note in Markdown, without the final newline.
    pub fn text(&self) -> &str {
        self.markdown.strip_suffix('\n').unwrap_or(&self.markdown)
    }

    /// Replaces the note with one read from disk.
    pub fn show(&mut self, text: &str) {
        self.replace(text);
        self.loaded = self.markdown.clone();
    }

    /// Replaces the note as if the user had typed `text`.
    pub fn replace(&mut self, text: &str) {
        self.state.set_markdown(text);
        self.markdown = self.state.markdown();
    }

    /// The whole note in Markdown, as it is written to the file.
    pub fn markdown(&self) -> &str {
        &self.markdown
    }

    pub fn changed(&self) -> bool {
        self.markdown != self.loaded
    }

    /// Records that `saved` is now on disk. Typing that went on while it
    /// was being written still counts as a change.
    pub fn mark_saved(&mut self, saved: String) {
        self.loaded = saved;
    }

    /// Follows a theme change; the editor draws its own colours.
    pub fn set_style(&mut self, style: EditorStyle) {
        if *self.state.current_style() != style {
            self.state.set_style(style);
        }
    }

    /// Follows a change of the interface language; the editor translates
    /// its own toolbar and status bar.
    pub fn set_language(&mut self, language: iced_markdown_editor::Language) {
        self.state.set_language(language);
    }
}

impl std::fmt::Debug for NoteField {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NoteField")
            .field("markdown", &self.markdown)
            .field("loaded", &self.loaded)
            .finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use iced::window::Direction;
    use iced::{Size, Vector};

    fn resize(direction: Direction) -> Resize {
        Resize {
            direction,
            start: Size::new(820.0, 580.0),
            from: None,
        }
    }

    #[test]
    fn the_dragged_edge_follows_the_cursor() {
        let max = Size::new(2000.0, 2000.0);
        let delta = Vector::new(30.0, 20.0);
        assert_eq!(
            resize(Direction::SouthEast).size_for(delta, max),
            Size::new(880.0, 620.0)
        );
        assert_eq!(
            resize(Direction::NorthWest).size_for(delta, max),
            Size::new(760.0, 540.0)
        );
        assert_eq!(resize(Direction::East).size_for(delta, max), Size::new(880.0, 580.0));
        assert_eq!(resize(Direction::North).size_for(delta, max), Size::new(820.0, 540.0));
    }

    #[test]
    fn the_size_stays_between_the_minimum_and_the_window() {
        let max = Size::new(900.0, 600.0);
        let grip = resize(Direction::SouthEast);
        assert_eq!(grip.size_for(Vector::new(500.0, 500.0), max), max);
        assert_eq!(grip.size_for(Vector::new(-500.0, -500.0), max), MIN_DIALOG);
    }
}
