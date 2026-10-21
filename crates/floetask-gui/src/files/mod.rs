//! The file drawer: opening, watching, switching and archiving todo files,
//! the overview of all of them, and writing changes to the files.

mod update;
mod view;

use std::path::PathBuf;

use floetask_application::{AppError, FileSort};
use floetask_domain::TodoDocument;

pub(crate) use view::drawer;

#[derive(Debug, Clone)]
pub enum Message {
    Loaded(PathBuf, Result<TodoDocument, AppError>),
    Changed(PathBuf),
    /// Result of a change written to a file.
    Saved(PathBuf, Result<TodoDocument, AppError>),
    /// Result of a change that also reports how many todos it touched.
    SavedCount(PathBuf, Result<(TodoDocument, usize), AppError>, &'static str),
    OpenDialog,
    CreateDialog,
    Picked(Option<PathBuf>),
    Created(Option<PathBuf>),
    Select(usize),
    /// Shows the overview of every file.
    SelectOverview,
    Menu(Option<usize>),
    OpenDoneFile(usize),
    ToggleDrawer,
    ChangeDoneFile(usize),
    DoneFilePicked(usize, Option<PathBuf>),
    Reveal(usize),
    AskRemove(usize),
    ArchiveCompleted,
    /// Opens (`true`) or closes the sort menu.
    SortMenu(bool),
    Sort(FileSort),
    /// A file was pressed: start dragging it.
    Press(usize),
    /// The mouse entered (`true`) or left a file and its archive.
    Hover(usize, bool),
    /// The cursor left the pressed file: a press there became a drag.
    EntryLeft,
    Release,
    /// The cursor left the file list, which ends a drag without a drop.
    ListLeft,
}

/// A file being dragged in the drawer to a new place.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FileDrag {
    /// Index of the file in `AppState::files`.
    pub from: usize,
    /// The cursor left the file since the press, so releasing it is a drop,
    /// never a click.
    pub moved: bool,
}
