//! The file drawer: opening, watching, switching and archiving todo files,
//! and writing changes to the active one.

mod update;
mod view;

use std::path::PathBuf;

use floetask_application::AppError;
use floetask_domain::TodoDocument;

pub(crate) use view::drawer;

#[derive(Debug, Clone)]
pub enum Message {
    Loaded(PathBuf, Result<TodoDocument, AppError>),
    Changed(PathBuf),
    /// Result of a change written to the active file.
    Saved(PathBuf, Result<TodoDocument, AppError>),
    /// Result of a change that also reports how many todos it touched.
    SavedCount(PathBuf, Result<(TodoDocument, usize), AppError>, &'static str),
    OpenDialog,
    CreateDialog,
    Picked(Option<PathBuf>),
    Created(Option<PathBuf>),
    Select(usize),
    Menu(Option<usize>),
    OpenDoneFile(usize),
    ToggleDrawer,
    ChangeDoneFile(usize),
    DoneFilePicked(usize, Option<PathBuf>),
    Reveal(usize),
    AskRemove(usize),
    ArchiveCompleted,
}
