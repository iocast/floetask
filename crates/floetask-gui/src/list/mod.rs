//! The todo list: grouped rows, their quick actions and keyboard selection.
//! Its card pieces (body, meta, labels) are reused by the board and calendar.

mod update;
mod view;

use floetask_domain::listing::Attribute;

use crate::app::Target;
use crate::editor::DateKey;

pub(crate) use view::{
    attribute_label, body, card_menu, collapsible, file_header, group_header, looks_like_markdown, meta, status_label,
    status_name, view,
};

#[derive(Debug, Clone)]
pub enum Message {
    OpenTodo(Target),
    ToggleComplete(Target),
    /// Opens the menu of the todo with this `ListedTodo::id`, or closes it.
    RowMenu(Option<(usize, usize)>),
    Copy(String),
    ArchiveOne(Target),
    AskDelete(Target),
    ChipFilter(Attribute, String),
    /// Collapses or expands the list group with this key.
    ToggleGroup(String),
    OpenDatePicker(Target, DateKey),
    OpenNote(Target),
    /// The todo under the mouse, by `ListedTodo::id`.
    Hover(Option<(usize, usize)>),
}
