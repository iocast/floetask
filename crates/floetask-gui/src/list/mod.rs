//! The todo list: grouped rows, their quick actions and keyboard selection.
//! Its card pieces (body, meta, labels) are reused by the board and calendar.

mod update;
mod view;

use floetask_application::TodoRef;
use floetask_domain::listing::Attribute;

use crate::editor::DateKey;

pub(crate) use view::{
    attribute_label, body, collapsible, group_header, looks_like_markdown, meta, status_label, status_name, view,
};

#[derive(Debug, Clone)]
pub enum Message {
    OpenTodo(TodoRef),
    ToggleComplete(TodoRef),
    RowMenu(Option<usize>),
    Copy(String),
    ArchiveOne(TodoRef),
    AskDelete(TodoRef),
    ChipFilter(Attribute, String),
    /// Collapses or expands the list group with this key.
    ToggleGroup(String),
    OpenDatePicker(TodoRef, DateKey),
    OpenNote(TodoRef),
    /// The line of the todo under the mouse.
    Hover(Option<usize>),
}
