//! Opening, saving and closing the note window.

use std::path::{Path, PathBuf};

use iced::{Size, Task, window};
use iced_markdown_editor::Event;

use floetask_application::{NoteService, WindowState};
use floetask_domain::NoteLink;

use super::NoteWindow;
use crate::app::{Dialog, Floetask, Message};
use crate::editor::NoteField;
use crate::{editor, note_window};

/// The note window opens as an ordinary window with the system's frame,
/// where it was last; the first time, the system places it.
fn settings(last: &WindowState) -> window::Settings {
    window::Settings {
        size: Size::new(last.width, last.height),
        position: match (last.x, last.y) {
            (Some(x), Some(y)) => window::Position::Specific(iced::Point::new(x, y)),
            _ => window::Position::default(),
        },
        min_size: Some(Size::new(420.0, 320.0)),
        exit_on_close_request: false,
        icon: crate::logo::window_icon(),
        ..window::Settings::default()
    }
}

impl Floetask {
    pub(crate) fn update_note_window(&mut self, message: note_window::Message) -> Task<Message> {
        match message {
            note_window::Message::Loaded(todo_file, result) => match result {
                Ok((link, text)) => {
                    // The note may have just been created: update its link.
                    self.refresh();
                    self.show_note_window(todo_file, link, &text, None)
                }
                Err(error) => self.report(&error),
            },
            note_window::Message::Action(action) => {
                let Some(window) = &mut self.note_window else {
                    return Task::none();
                };
                let output = window.note.update(action);
                let mut tasks = vec![
                    output
                        .task
                        .map(|action| Message::NoteWindow(note_window::Message::Action(action))),
                ];
                for event in output.events {
                    match event {
                        Event::Changed => tasks.push(window.schedule_autosave()),
                        // Ctrl-clicked links open like the links on todo cards.
                        Event::LinkClicked(url) => tasks.push(Task::done(Message::OpenLink(url))),
                        _ => {}
                    }
                }
                Task::batch(tasks)
            }
            note_window::Message::Autosave(revision) => match &self.note_window {
                Some(window) if window.revision == revision => self.save_note_window().unwrap_or_else(Task::none),
                _ => Task::none(),
            },
            note_window::Message::Save => self.save_note_window().unwrap_or_else(Task::none),
            note_window::Message::Saved(path, text, result) => self.note_window_saved(&path, text, result),
            note_window::Message::OpenExternally => {
                let Some(window) = &self.note_window else {
                    return Task::none();
                };
                let (notes, todo_file, link) = (
                    self.services.notes.clone(),
                    window.todo_file.clone(),
                    window.link.clone(),
                );
                Task::perform(async move { notes.open_externally(&todo_file, &link) }, |result| {
                    Message::NoteWindow(note_window::Message::OpenedExternally(result))
                })
            }
            note_window::Message::OpenedExternally(result) => match result {
                // The note may have just been created: update its link.
                Ok(_) => {
                    self.refresh();
                    Task::none()
                }
                Err(error) => self.report(&error),
            },
        }
    }

    /// Shows a note in the note window, opening the window or reusing it;
    /// the note it showed before is saved first. `text` is the note on disk,
    /// `unsaved` text typed in the dialog that is not on disk yet.
    pub(crate) fn show_note_window(
        &mut self,
        todo_file: PathBuf,
        link: NoteLink,
        text: &str,
        unsaved: Option<String>,
    ) -> Task<Message> {
        let path = NoteService::link_path(&todo_file, &link);
        let mut tasks = Vec::new();
        let showing_other = self.note_window.as_ref().is_some_and(|window| window.path() != path);
        let opening = self.note_window.is_none();
        if showing_other {
            tasks.push(self.release_note_window());
        }
        let colors = self.colors();
        let window = match &mut self.note_window {
            Some(window) => {
                tasks.push(window::gain_focus(window.id));
                window
            }
            None => {
                let (id, open) = window::open(settings(&self.state.note_window));
                tasks.push(open.map(|_| Message::Noop));
                self.note_window.insert(NoteWindow {
                    id,
                    todo_file: todo_file.clone(),
                    link: link.clone(),
                    note: NoteField::new(colors),
                    revision: 0,
                    error: None,
                })
            }
        };
        // Opening the note the window already shows keeps what is typed in it.
        if opening || showing_other {
            window.todo_file = todo_file;
            window.link = link;
            window.error = None;
            window.note.show(text);
        }
        if let Some(unsaved) = unsaved {
            window.note.replace(&unsaved);
            tasks.push(window.schedule_autosave());
        }
        Task::batch(tasks)
    }

    /// Saves and closes the note window, remembering where it was.
    pub(crate) fn close_note_window(&mut self) -> Task<Message> {
        let release = self.release_note_window();
        match self.note_window.take() {
            Some(window) => Task::batch([release, self.persist_state(), window::close(window.id)]),
            None => release,
        }
    }

    /// Follows the note window's moves and resizes, so it opens there again.
    pub(crate) fn on_note_window_event(&mut self, event: window::Event) -> Task<Message> {
        match event {
            window::Event::CloseRequested => self.close_note_window(),
            window::Event::Resized(size) => {
                self.state.note_window.width = size.width;
                self.state.note_window.height = size.height;
                Task::none()
            }
            window::Event::Moved(position) => {
                self.state.note_window.x = Some(position.x);
                self.state.note_window.y = Some(position.y);
                Task::none()
            }
            _ => Task::none(),
        }
    }

    /// Saves the note window's changes before it stops showing its note.
    /// Once that note is on disk, the dialog, which kept its hands off it
    /// meanwhile, reads it back.
    fn release_note_window(&mut self) -> Task<Message> {
        let path = self.note_window.as_ref().map(NoteWindow::path);
        match (self.save_note_window(), path) {
            (Some(save), _) => save,
            (None, Some(path)) => self.reload_dialog_note(&path),
            (None, None) => Task::none(),
        }
    }

    /// Writes the note window's changes; `None` when there are none.
    fn save_note_window(&self) -> Option<Task<Message>> {
        let window = self.note_window.as_ref().filter(|window| window.note.changed())?;
        let notes = self.services.notes.clone();
        let (todo_file, link, path) = (window.todo_file.clone(), window.link.clone(), window.path());
        let text = window.note.markdown().to_owned();
        Some(Task::perform(
            async move {
                let result = notes.write(&todo_file, &link, &text);
                (path, text, result)
            },
            |(path, text, result)| Message::NoteWindow(note_window::Message::Saved(path, text, result)),
        ))
    }

    fn note_window_saved(
        &mut self,
        path: &Path,
        text: String,
        result: Result<(), floetask_application::AppError>,
    ) -> Task<Message> {
        let window = self.note_window.as_mut().filter(|window| window.path() == path);
        match (result, window) {
            (Ok(()), Some(window)) => {
                window.note.mark_saved(text);
                window.error = None;
                self.refresh();
                Task::none()
            }
            (Ok(()), None) => {
                self.refresh();
                self.reload_dialog_note(path)
            }
            (Err(error), window) => {
                if let Some(window) = window {
                    window.error = Some(crate::i18n::error_text(&error));
                }
                self.report(&error)
            }
        }
    }

    /// Reads the dialog's note back from disk if it is the file at `path`.
    fn reload_dialog_note(&self, path: &Path) -> Task<Message> {
        let Some(Dialog::Editor(editor)) = &self.dialog else {
            return Task::none();
        };
        let todo_file = editor.file.clone();
        let Some(Ok(link)) = editor.note_target() else {
            return Task::none();
        };
        if NoteService::link_path(&todo_file, &link) != path {
            return Task::none();
        }
        let notes = self.services.notes.clone();
        Task::perform(
            async move {
                let text = notes.read(&todo_file, &link);
                (link, text)
            },
            |(link, text)| Message::Editor(editor::Message::NoteLinked(link, text)),
        )
    }

    /// Whether the dialog's note is the one open in the note window. The
    /// dialog then leaves it to the window, so the two never overwrite
    /// each other.
    pub(crate) fn dialog_note_in_window(&self) -> bool {
        let (Some(Dialog::Editor(editor)), Some(window)) = (&self.dialog, &self.note_window) else {
            return false;
        };
        matches!(editor.note_target(), Some(Ok(link)) if NoteService::link_path(&editor.file, &link) == window.path())
    }

    /// Saves the note window on exit, while there is no time for a task.
    pub(crate) fn save_note_window_now(&self) {
        if let Some(window) = self.note_window.as_ref().filter(|window| window.note.changed())
            && let Err(error) = self
                .services
                .notes
                .write(&window.todo_file, &window.link, window.note.markdown())
        {
            eprintln!("floetask: {error}");
        }
    }
}
