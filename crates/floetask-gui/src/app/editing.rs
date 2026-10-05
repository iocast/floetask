//! The add/edit dialog and the date pickers.

use iced::Task;
use iced::widget::operation;

use std::path::Path;

use floetask_application::{AppError, NoteService};
use floetask_domain::todo::Todo;
use floetask_domain::{NoteError, NoteName, Recurrence};

use super::{Calendar, DateKey, Dialog, Editor, Floetask, Message};
use crate::compose::edit_text;
use crate::i18n::tr;
use crate::view;

impl Floetask {
    pub(super) fn update_editing(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::NewTodo => self.open_new_editor(""),
            Message::CalendarMonth(months) => {
                if let Some(calendar) = self.active_calendar() {
                    calendar.shift_month(months);
                }
                Task::none()
            }
            Message::CalendarPick(date) => self.pick_date(date),
            Message::SaveEditor => self.save_editor(),
            Message::NoteLoaded(target, result) => match result {
                Ok(Some(text)) => {
                    if let Some(Dialog::Editor(editor)) = &mut self.dialog
                        && editor.target.as_ref() == Some(&target)
                    {
                        editor.load_note(&text);
                    }
                    Task::none()
                }
                Ok(None) => Task::none(),
                Err(error) => self.report(&error),
            },
            other => {
                let (projects, contexts) = (self.projects.clone(), self.contexts.clone());
                let today = self.today;
                let Some(Dialog::Editor(editor)) = &mut self.dialog else {
                    return Task::none();
                };
                match other {
                    Message::EditorAction(action) => {
                        editor.content.perform(action);
                        editor.refresh_suggestions(&projects, &contexts);
                        Task::none()
                    }
                    Message::EditorSuggestion(index) => {
                        editor.accept_suggestion(index);
                        operation::focus(view::EDITOR_ID)
                    }
                    Message::EditorSuggestionMove(step) => {
                        let count = editor.suggestions.len().max(1) as isize;
                        editor.selected_suggestion =
                            (editor.selected_suggestion as isize + step).rem_euclid(count) as usize;
                        Task::none()
                    }
                    Message::EditorDismissSuggestions => {
                        editor.suggestions.clear();
                        Task::none()
                    }
                    Message::EditorAccept => {
                        editor.accept_suggestion(editor.selected_suggestion);
                        Task::none()
                    }
                    Message::EditorPriority(priority) => {
                        rewrite(editor, |todo| todo.with_priority(priority));
                        Task::none()
                    }
                    Message::EditorStatus(status) => {
                        rewrite(editor, |todo| todo.with_status(Some(&status)));
                        Task::none()
                    }
                    Message::EditorOpenCalendar(key) => {
                        editor.calendar = match &editor.calendar {
                            Some(calendar) if calendar.key == key => None,
                            _ => {
                                let todo = Todo::from_user_text(&editor.text());
                                let current = match key {
                                    DateKey::Due => todo.due(),
                                    DateKey::Threshold => todo.threshold(),
                                };
                                Some(Calendar::new(key, current, today))
                            }
                        };
                        Task::none()
                    }
                    Message::EditorCloseCalendar => {
                        editor.calendar = None;
                        operation::focus(view::EDITOR_ID)
                    }
                    Message::EditorNoteAction(action) => {
                        editor.note.perform(action);
                        Task::none()
                    }
                    Message::EditorRecurrenceCount(count) => {
                        if count.chars().all(|c| c.is_ascii_digit()) && count.len() <= 3 {
                            editor.recurrence_count = count;
                        }
                        Task::none()
                    }
                    Message::EditorRecurrenceUnit(unit) => {
                        editor.recurrence_unit = unit;
                        Task::none()
                    }
                    Message::EditorRecurrenceStrict(strict) => {
                        editor.recurrence_strict = strict;
                        Task::none()
                    }
                    Message::EditorApplyRecurrence(set) => {
                        let value = set.then(|| Recurrence {
                            strict: editor.recurrence_strict,
                            count: editor.recurrence_count.parse().unwrap_or(1).max(1),
                            unit: editor.recurrence_unit,
                        });
                        rewrite(editor, |todo| {
                            todo.with_extension("rec", value.map(|r| r.to_string()).as_deref())
                        });
                        Task::none()
                    }
                    Message::EditorPomodoros(count) => {
                        if count.chars().all(|c| c.is_ascii_digit()) && count.len() <= 3 {
                            editor.pomodoros = count;
                        }
                        Task::none()
                    }
                    Message::EditorApplyPomodoros(set) => {
                        let value = editor.pomodoros.parse::<u32>().ok().filter(|_| set);
                        rewrite(editor, |todo| {
                            todo.with_extension("pm", value.map(|n| n.to_string()).as_deref())
                        });
                        Task::none()
                    }
                    _ => Task::none(),
                }
            }
        }
    }

    pub(crate) fn open_new_editor(&mut self, text: &str) -> Task<Message> {
        self.row_menu = None;
        self.dialog = Some(Dialog::Editor(Box::new(Editor::new(None, text))));
        operation::focus(view::EDITOR_ID)
    }

    /// Saves the todo and, when its notes changed, the note file first. A
    /// todo without `note:` gets one named after its text, so the user never
    /// has to name or create the file.
    fn save_editor(&mut self) -> Task<Message> {
        let Some(Dialog::Editor(editor)) = &self.dialog else {
            return Task::none();
        };
        let text = editor.text();
        if text.trim().is_empty() {
            return self.toast_error(tr("empty_todo"));
        }
        let note = match NoteSave::from_editor(editor) {
            Ok(note) => note,
            Err(error) => return self.toast_error(error.to_string()),
        };
        let Some(path) = self.active_path() else {
            return Task::none();
        };
        let target = editor.target.clone();
        self.dialog = None;
        let (todo_files, notes) = (self.services.todo_files.clone(), self.services.notes.clone());
        let (input, options) = (self.input_options(), self.file_options());
        let saved = path.clone();
        Task::perform(
            async move {
                let text = note.write(&notes, &path, text)?;
                match target {
                    Some(target) => todo_files.update(&path, &target, &text, &input, &options),
                    None => todo_files.add(&path, &text, &input, &options),
                }
            },
            move |result| Message::Saved(saved.clone(), result),
        )
    }

    fn active_calendar(&mut self) -> Option<&mut Calendar> {
        match &mut self.dialog {
            Some(Dialog::RowDate { calendar, .. }) => Some(calendar),
            Some(Dialog::Editor(editor)) => editor.calendar.as_mut(),
            _ => None,
        }
    }

    fn pick_date(&mut self, date: Option<floetask_domain::Date>) -> Task<Message> {
        match self.dialog.take() {
            Some(Dialog::RowDate { target, calendar }) => self.change_active_file(move |service, path, options| {
                service.set_date(path, &target, calendar.key.key(), date, options)
            }),
            Some(Dialog::Editor(mut editor)) => {
                if let Some(calendar) = editor.calendar.take() {
                    rewrite(&mut editor, |todo| todo.with_date_extension(calendar.key.key(), date));
                }
                self.dialog = Some(Dialog::Editor(editor));
                operation::focus(view::EDITOR_ID)
            }
            other => {
                self.dialog = other;
                Task::none()
            }
        }
    }
}

/// What saving the dialog does with the note file.
pub(crate) enum NoteSave {
    /// The notes did not change: leave the file alone.
    Keep,
    /// Write the file the todo's `note:` already names.
    Existing(NoteName, String),
    /// Create a file under a free variant of this name and add `note:`.
    New(NoteName, String),
}

impl NoteSave {
    pub(crate) fn from_editor(editor: &Editor) -> Result<Self, NoteError> {
        if !editor.note_changed() {
            return Ok(NoteSave::Keep);
        }
        let body = editor.note_text();
        Ok(match editor.note_target().transpose()? {
            Some(name) => NoteSave::Existing(name, body),
            None if body.trim().is_empty() => NoteSave::Keep,
            None => NoteSave::New(editor.new_note_name(), body),
        })
    }

    /// Writes the note file and returns the todo text to save.
    pub(crate) fn write(self, notes: &NoteService, todo_file: &Path, text: String) -> Result<String, AppError> {
        match self {
            NoteSave::Keep => Ok(text),
            NoteSave::Existing(name, body) => {
                notes.write(todo_file, &name, &body)?;
                Ok(text)
            }
            NoteSave::New(name, body) => {
                let name = notes.unused_name(todo_file, name);
                notes.write(todo_file, &name, &body)?;
                Ok(edit_text(&text, |todo| todo.with_note(Some(&name))))
            }
        }
    }
}

/// Applies a todo edit to the dialog text.
fn rewrite(editor: &mut Editor, edit: impl FnOnce(&Todo) -> Todo) {
    let text = editor.text();
    if text.trim().is_empty() {
        // Pickers on an empty field still produce a valid todo skeleton.
        editor.set_text(&edit(&Todo::parse("")).editable_text());
    } else {
        editor.set_text(&edit_text(&text, edit));
    }
}
