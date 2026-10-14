//! The note window: a second window with the Markdown editor, opened from a
//! todo's note chip or from the add/edit dialog. There is only one; opening
//! another note reuses it. Edits are saved once typing pauses, on `Ctrl+S`,
//! and before the window closes or shows another note.

mod update;
mod view;

use std::path::PathBuf;
use std::time::Duration;

use iced::{Task, window};

use floetask_application::{AppError, NoteService};
use floetask_domain::NoteLink;

use crate::app::Message as AppMessage;
use crate::editor::NoteField;

pub(crate) use view::view;

#[derive(Debug, Clone)]
pub enum Message {
    /// A note opened from a todo card, created first when missing.
    Loaded(PathBuf, Result<(NoteLink, String), AppError>),
    Action(iced_markdown_editor::Action),
    /// Saves the note window if this is still its latest edit.
    Autosave(u64),
    Save,
    /// A save finished: the file, the text written, the result.
    Saved(PathBuf, String, Result<(), AppError>),
    OpenExternally,
    /// Opening the note in the system's app finished.
    OpenedExternally(Result<PathBuf, AppError>),
}

/// How long typing has to pause before the note is saved.
const AUTOSAVE_DELAY: Duration = Duration::from_millis(800);

pub struct NoteWindow {
    pub id: window::Id,
    /// The todo file whose notes folder `link` is relative to.
    pub todo_file: PathBuf,
    pub link: NoteLink,
    pub note: NoteField,
    /// Bumped on every edit, so only the latest pending autosave writes.
    revision: u64,
    /// Why the last save failed, until a save succeeds.
    pub error: Option<String>,
}

impl NoteWindow {
    pub fn path(&self) -> PathBuf {
        NoteService::link_path(&self.todo_file, &self.link)
    }

    /// The note's name for the title: `todo-notes/plan.md`, or the file name
    /// of a note kept elsewhere.
    pub fn title(&self) -> String {
        match &self.link {
            NoteLink::Folder(name) => format!("{}/{name}", NoteService::folder_name(&self.todo_file)),
            NoteLink::External(_) => self.link.file_name().to_owned(),
        }
    }

    fn schedule_autosave(&mut self) -> Task<AppMessage> {
        self.revision += 1;
        let revision = self.revision;
        Task::perform(tokio::time::sleep(AUTOSAVE_DELAY), move |()| {
            AppMessage::NoteWindow(Message::Autosave(revision))
        })
    }
}

impl std::fmt::Debug for NoteWindow {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NoteWindow")
            .field("id", &self.id)
            .field("link", &self.link)
            .field("note", &self.note)
            .finish_non_exhaustive()
    }
}
