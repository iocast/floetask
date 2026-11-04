//! The todo list: grouped rows, their quick actions and keyboard selection.
//! Its card pieces (body, meta, labels) are reused by the board and calendar.

mod update;
mod view;

use std::path::PathBuf;

use floetask_domain::listing::Attribute;

use crate::app::Target;
use crate::editor::DateKey;

pub(crate) use view::{
    add_area, attribute_label, body, card_menu, collapsible, drag_ghost, file_header, group_header,
    looks_like_markdown, meta, move_dialog, status_label, status_name, view,
};

#[derive(Debug, Clone)]
pub enum Message {
    OpenTodo(Target),
    ToggleComplete(Target),
    /// Opens the menu of the todo with this `ListedTodo::id`, or closes it.
    RowMenu(Option<(usize, usize)>),
    Copy(String),
    ArchiveOne(Target),
    /// Opens the dialog that moves the todo to another registered file.
    AskMoveToFile(Target),
    /// The file picked in the move dialog's dropdown.
    MoveToFileChoice(PathBuf),
    AskDelete(Target),
    ChipFilter(Attribute, String),
    /// Collapses or expands the list group with this key.
    ToggleGroup(String),
    OpenDatePicker(Target, DateKey),
    OpenNote(Target),
    /// The todo under the mouse, by `ListedTodo::id`. `None` when the
    /// cursor left a row.
    Hover(Option<(usize, usize)>),
    /// A row was pressed: a release on it opens the todo, dragging it onto
    /// a file in the file drawer moves it there.
    Press(Target),
    /// The mouse button was released over the list.
    Release,
}

/// A row being dragged out of the list, onto a file in the file drawer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RowDrag {
    pub target: Target,
    /// The cursor left the row since the press, so releasing it is a drop,
    /// never a click.
    pub moved: bool,
}
