//! Modal dialogs. Each dialog's own state lives with its feature; this is
//! the one place that knows which dialog is open.

use floetask_application::TodoRef;
use floetask_domain::board::BoardColumns;
use floetask_domain::listing::Attribute;

use crate::editor::{DatePicker, Editor};
use crate::settings::SettingsDialog;

/// An action waiting for the user's confirmation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Pending {
    DeleteTodo(TodoRef),
    RemoveFile(usize),
    RemoveValue(Attribute, String),
    DeleteSavedFilter(usize),
}

#[derive(Debug)]
pub enum Dialog {
    Editor(Box<Editor>),
    Settings(SettingsDialog),
    Confirm {
        message: String,
        detail: Option<String>,
        pending: Pending,
    },
    Rename {
        attribute: Attribute,
        from: String,
        input: String,
    },
    RowDate {
        target: TodoRef,
        calendar: DatePicker,
    },
    SaveFilter {
        name: String,
    },
    /// The active file's board columns while being edited.
    BoardColumns {
        columns: BoardColumns,
        input: String,
    },
}
