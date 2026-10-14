//! Editing a todo in the dialog, its note, and picking dates.

use iced::Task;
use iced::widget::operation;

use std::path::{Path, PathBuf};

use floetask_application::{AppError, NoteService};
use floetask_domain::note::new_note_content;
use floetask_domain::todo::Todo;
use floetask_domain::{NoteError, NoteLink, NoteName, Recurrence};
use iced_markdown_editor::Event;

use super::{DateKey, DatePicker, Editor};
use crate::app::{Dialog, Floetask, Message};
use crate::compose::edit_text;
use crate::i18n::{self, tr};
use crate::{editor, files, view};

impl Floetask {
    pub(crate) fn update_editor(&mut self, message: editor::Message) -> Task<Message> {
        match message {
            editor::Message::New => self.open_new_editor(""),
            editor::Message::PickerMonth(months) => {
                if let Some(calendar) = self.active_calendar() {
                    calendar.shift_month(months);
                }
                Task::none()
            }
            editor::Message::PickDate(date) => self.pick_date(date),
            editor::Message::Save => self.save_editor(),
            editor::Message::NoteLoaded(target, result) => match result {
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
            editor::Message::PickNoteFile => {
                let folder = self
                    .active_path()
                    .map(|path| path.with_file_name(NoteService::folder_name(&path)));
                Task::perform(pick_note_file(folder), |path| {
                    Message::Editor(editor::Message::NoteFilePicked(path))
                })
            }
            editor::Message::NoteFilePicked(picked) => {
                let (Some(picked), Some(todo_file)) = (picked, self.active_path()) else {
                    return Task::none();
                };
                // A file already in the notes folder links as it is; any
                // other file could move there or stay put, so ask.
                match NoteService::folder_link(&todo_file, &picked) {
                    Some(link) => self.link_note(link),
                    None => {
                        if let Some(Dialog::Editor(editor)) = &mut self.dialog {
                            editor.pending_link = Some(picked);
                        }
                        Task::none()
                    }
                }
            }
            editor::Message::MovePickedNote => {
                let (Some(picked), Some(todo_file)) = (self.take_pending_link(), self.active_path()) else {
                    return Task::none();
                };
                let notes = self.services.notes.clone();
                Task::perform(async move { notes.move_into_folder(&todo_file, &picked) }, |result| {
                    Message::Editor(editor::Message::NoteFileMoved(result))
                })
            }
            editor::Message::KeepPickedNote => match self
                .take_pending_link()
                .map(|picked| NoteService::external_link(&picked))
            {
                Some(Ok(link)) => self.link_note(link),
                Some(Err(error)) => self.report(&error),
                None => Task::none(),
            },
            editor::Message::CancelPickedNote => {
                self.take_pending_link();
                Task::none()
            }
            editor::Message::NoteFileMoved(result) => match result {
                Ok(link) => self.link_note(link),
                Err(error) => self.report(&error),
            },
            editor::Message::NoteLinked(link, result) => {
                let text = match result {
                    Ok(text) => text.unwrap_or_default(),
                    Err(error) => return self.report(&error),
                };
                // Ignore a read that a later link or unlink overtook.
                if let Some(Dialog::Editor(editor)) = &mut self.dialog
                    && editor.note_target() == Some(Ok(link))
                {
                    editor.note.show(&text);
                }
                Task::none()
            }
            editor::Message::OpenNoteWindow => self.hand_note_to_window(),
            editor::Message::NoteHandedOver(result) => match result {
                Ok((link, text)) => self.note_handed_over(link, text),
                Err(error) => self.report(&error),
            },
            other => {
                let (projects, contexts) = (self.projects.clone(), self.contexts.clone());
                let today = self.today;
                let Some(Dialog::Editor(editor)) = &mut self.dialog else {
                    return Task::none();
                };
                match other {
                    editor::Message::Action(action) => {
                        editor.content.perform(action);
                        editor.refresh_suggestions(&projects, &contexts);
                        Task::none()
                    }
                    editor::Message::Suggestion(index) => {
                        editor.accept_suggestion(index);
                        operation::focus(view::EDITOR_ID)
                    }
                    editor::Message::SuggestionMove(step) => {
                        let count = editor.suggestions.len().max(1) as isize;
                        editor.selected_suggestion =
                            (editor.selected_suggestion as isize + step).rem_euclid(count) as usize;
                        Task::none()
                    }
                    editor::Message::DismissSuggestions => {
                        editor.suggestions.clear();
                        Task::none()
                    }
                    editor::Message::Accept => {
                        editor.accept_suggestion(editor.selected_suggestion);
                        Task::none()
                    }
                    editor::Message::Priority(priority) => {
                        rewrite(editor, |todo| todo.with_priority(priority));
                        Task::none()
                    }
                    editor::Message::Status(status) => {
                        rewrite(editor, |todo| todo.with_status(Some(&status)));
                        Task::none()
                    }
                    editor::Message::OpenDatePicker(key) => {
                        editor.calendar = match &editor.calendar {
                            Some(calendar) if calendar.key == key => None,
                            _ => {
                                let todo = Todo::from_user_text(&editor.text());
                                let current = match key {
                                    DateKey::Due => todo.due(),
                                    DateKey::Threshold => todo.threshold(),
                                };
                                Some(DatePicker::new(key, current, today))
                            }
                        };
                        Task::none()
                    }
                    editor::Message::CloseDatePicker => {
                        editor.calendar = None;
                        operation::focus(view::EDITOR_ID)
                    }
                    editor::Message::NoteAction(action) => {
                        let output = editor.note.update(action);
                        // Ctrl-clicked links open like the links on todo cards.
                        let links = output.events.into_iter().filter_map(|event| match event {
                            Event::LinkClicked(url) => Some(Task::done(Message::OpenLink(url))),
                            _ => None,
                        });
                        Task::batch(
                            links.chain([output
                                .task
                                .map(|action| Message::Editor(editor::Message::NoteAction(action)))]),
                        )
                    }
                    editor::Message::UnlinkNote => {
                        rewrite(editor, |todo| todo.with_note_link(None));
                        editor.note.show("");
                        Task::none()
                    }
                    editor::Message::RecurrenceCount(count) => {
                        if count.chars().all(|c| c.is_ascii_digit()) && count.len() <= 3 {
                            editor.recurrence_count = count;
                        }
                        Task::none()
                    }
                    editor::Message::RecurrenceUnit(unit) => {
                        editor.recurrence_unit = unit;
                        Task::none()
                    }
                    editor::Message::RecurrenceStrict(strict) => {
                        editor.recurrence_strict = strict;
                        Task::none()
                    }
                    editor::Message::ApplyRecurrence(set) => {
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
                    editor::Message::Pomodoros(count) => {
                        if count.chars().all(|c| c.is_ascii_digit()) && count.len() <= 3 {
                            editor.pomodoros = count;
                        }
                        Task::none()
                    }
                    editor::Message::ApplyPomodoros(set) => {
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
        self.dialog = Some(Dialog::Editor(Box::new(Editor::new(None, text, self.colors()))));
        operation::focus(view::EDITOR_ID)
    }

    fn take_pending_link(&mut self) -> Option<PathBuf> {
        match &mut self.dialog {
            Some(Dialog::Editor(editor)) => editor.pending_link.take(),
            _ => None,
        }
    }

    /// Points the todo's `note:` at an existing file and shows that note in
    /// place of the notes typed so far, which belonged to the old link.
    fn link_note(&mut self, link: NoteLink) -> Task<Message> {
        let path = self.active_path();
        let (Some(Dialog::Editor(editor)), Some(path)) = (&mut self.dialog, path) else {
            return Task::none();
        };
        rewrite(editor, |todo| todo.with_note_link(Some(&link)));
        let notes = self.services.notes.clone();
        Task::perform(
            async move {
                let text = notes.read(&path, &link);
                (link, text)
            },
            |(link, text)| Message::Editor(editor::Message::NoteLinked(link, text)),
        )
    }

    /// Opens the dialog's note in the note window. A todo without `note:`
    /// gets a new note named after its text first, as saving would do.
    fn hand_note_to_window(&mut self) -> Task<Message> {
        let (Some(Dialog::Editor(editor)), Some(todo_file)) = (&self.dialog, self.active_path()) else {
            return Task::none();
        };
        let target = match editor.note_target().transpose() {
            Ok(target) => target,
            Err(error) => return self.toast_error(i18n::error_text(&error.into())),
        };
        if self.dialog_note_in_window()
            && let Some(window) = &self.note_window
        {
            return iced::window::gain_focus(window.id);
        }
        let new_name = editor.new_note_name();
        let notes = self.services.notes.clone();
        Task::perform(
            async move {
                let link = match target {
                    Some(link) => link,
                    None => NoteLink::Folder(notes.unused_name(&todo_file, new_name)),
                };
                let text = notes.read(&todo_file, &link)?;
                Ok((link, text))
            },
            |result| Message::Editor(editor::Message::NoteHandedOver(result)),
        )
    }

    /// Moves the dialog's note into the note window. Text typed in the
    /// dialog but not saved goes along and is saved from there.
    fn note_handed_over(&mut self, link: NoteLink, on_disk: Option<String>) -> Task<Message> {
        let todo_file = self.active_path();
        let (Some(Dialog::Editor(editor)), Some(todo_file)) = (&mut self.dialog, todo_file) else {
            return Task::none();
        };
        if editor.note_target().is_none() {
            rewrite(editor, |todo| todo.with_note_link(Some(&link)));
        }
        let unsaved = if editor.note.changed() {
            Some(editor.note.markdown().to_owned())
        } else if on_disk.is_none() {
            Some(new_note_content(&Todo::from_user_text(&editor.text())))
        } else {
            None
        };
        let on_disk = on_disk.unwrap_or_default();
        // The dialog leaves the note to the window from now on.
        editor.note.show(unsaved.as_deref().unwrap_or(&on_disk));
        self.show_note_window(todo_file, link, &on_disk, unsaved)
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
        // A note open in the note window is saved from there.
        let note = match NoteSave::from_editor(editor) {
            Ok(_) if self.dialog_note_in_window() => NoteSave::Keep,
            Ok(note) => note,
            Err(error) => return self.toast_error(i18n::error_text(&error.into())),
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
            move |result| Message::Files(files::Message::Saved(saved.clone(), result)),
        )
    }

    fn active_calendar(&mut self) -> Option<&mut DatePicker> {
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
    /// Write the file the todo's `note:` already links.
    Existing(NoteLink, String),
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
            Some(link) => NoteSave::Existing(link, body),
            None if body.trim().is_empty() => NoteSave::Keep,
            None => NoteSave::New(editor.new_note_name(), body),
        })
    }

    /// Writes the note file and returns the todo text to save.
    pub(crate) fn write(self, notes: &NoteService, todo_file: &Path, text: String) -> Result<String, AppError> {
        match self {
            NoteSave::Keep => Ok(text),
            NoteSave::Existing(link, body) => {
                notes.write(todo_file, &link, &body)?;
                Ok(text)
            }
            NoteSave::New(name, body) => {
                let link = NoteLink::Folder(notes.unused_name(todo_file, name));
                notes.write(todo_file, &link, &body)?;
                Ok(edit_text(&text, |todo| todo.with_note_link(Some(&link))))
            }
        }
    }
}

/// Asks for a Markdown or text file to link, starting in the notes folder.
async fn pick_note_file(folder: Option<PathBuf>) -> Option<PathBuf> {
    let mut dialog = rfd::AsyncFileDialog::new()
        .set_title(tr("link_note_dialog_title"))
        .add_filter(tr("markdown_files"), &["md", "markdown", "txt"]);
    if let Some(folder) = folder.filter(|folder| folder.is_dir()) {
        dialog = dialog.set_directory(folder);
    }
    dialog.pick_file().await.map(|handle| handle.path().to_path_buf())
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
