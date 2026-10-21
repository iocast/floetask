//! The add/edit dialog: the todo text with autocomplete, pickers for its
//! attributes, its notes, and the date picker it shares with the list.

mod date_picker;
mod state;
mod update;
mod view;

use std::path::PathBuf;

use iced::widget::text_editor;

use floetask_application::AppError;
use floetask_domain::{Date, NoteLink, Priority, RecurrenceUnit};

pub(crate) use date_picker::row_dialog;
pub use date_picker::{DateKey, DatePicker};
pub use state::{Editor, NoteField};
#[cfg(test)]
pub(crate) use update::NoteSave;
pub(crate) use view::view;

#[derive(Debug, Clone)]
pub enum Message {
    New,
    Action(text_editor::Action),
    Suggestion(usize),
    SuggestionMove(isize),
    Accept,
    DismissSuggestions,
    Priority(Option<Priority>),
    Status(String),
    /// Picks the file a new todo is added to, in the overview.
    File(PathBuf),
    OpenDatePicker(DateKey),
    CloseDatePicker,
    /// Shows the previous or next months in the open date picker, here or
    /// in the list's date dialog.
    PickerMonth(i32),
    PickDate(Option<Date>),
    RecurrenceCount(String),
    RecurrenceUnit(RecurrenceUnit),
    RecurrenceStrict(bool),
    ApplyRecurrence(bool),
    Pomodoros(String),
    ApplyPomodoros(bool),
    NoteAction(iced_markdown_editor::Action),
    /// The note file of the todo being edited, read from disk (`None`: missing).
    NoteLoaded(crate::app::Target, Result<Option<String>, AppError>),
    /// Asks for a Markdown file on disk to link instead of creating one.
    PickNoteFile,
    NoteFilePicked(Option<PathBuf>),
    /// Answers to "move the picked file into the notes folder?".
    MovePickedNote,
    KeepPickedNote,
    CancelPickedNote,
    /// A picked file, moved into the notes folder, ready to link.
    NoteFileMoved(Result<NoteLink, AppError>),
    /// Removes the todo's `note:` tag; the file is kept.
    UnlinkNote,
    /// A note just linked in the dialog, read from disk (`None`: missing).
    NoteLinked(NoteLink, Result<Option<String>, AppError>),
    /// Hands the dialog's note over to the note window.
    OpenNoteWindow,
    /// The dialog's note, ready for the note window: its link and its text
    /// on disk (`None`: no file yet).
    NoteHandedOver(Result<(NoteLink, Option<String>), AppError>),
    Save,
}
