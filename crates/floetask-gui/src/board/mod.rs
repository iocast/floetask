//! The status board: one column per status, cards dragged between them, and
//! the dialog that edits a file's columns.

mod update;
mod view;

use floetask_application::TodoRef;
use floetask_domain::listing::LaneId;

pub(crate) use view::{columns_dialog, view};

#[derive(Debug, Clone)]
pub enum Message {
    /// Turns the board's one-board-per-group layout on or off.
    Grouped(bool),
    /// Collapses or expands the board group with this key.
    ToggleGroup(String),
    /// A card was pressed in a lane: start dragging it.
    Press(TodoRef, LaneId),
    /// The mouse entered (`true`) or left a lane.
    Hover(LaneId, bool),
    /// The cursor left a card: a press there became a drag.
    CardLeft,
    Release,
    OpenColumns,
    ColumnInput(String),
    /// Adds the given column, or the typed one for `None`.
    ColumnAdd(Option<String>),
    ColumnMove(usize, isize),
    ColumnRemove(usize),
    ColumnsReset,
}

/// A card being dragged on the board.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoardDrag {
    pub target: TodoRef,
    /// Lane the card was picked up from.
    pub from: LaneId,
    /// The cursor left the card since the press, so releasing it is a drop,
    /// never a click.
    pub moved: bool,
}
