//! The filter drawer: attribute filters, view toggles and sorting, plus the
//! dialog that renames or removes a project or context.

mod update;
mod view;

use floetask_application::DrawerTab;
use floetask_domain::listing::Attribute;

pub(crate) use view::{rename_dialog, view};

#[derive(Debug, Clone)]
pub enum Message {
    Toggle,
    Tab(DrawerTab),
    ToggleSection(Attribute),
    FilterValue(Attribute, String, bool),
    ToggleCategory(Attribute),
    ViewToggle(ViewToggle, bool),
    MoveSort(usize, isize),
    InvertSort(usize),
    FileOrder(bool),
    CompletedLast(bool),
    AskRename(Attribute, String),
    RenameInput(String),
    AskRemoveValue(Attribute, String),
    ResetFilters,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewToggle {
    Completed,
    Hidden,
    HiddenAttributes,
    HiddenStatuses,
    FutureThreshold,
    FutureDue,
}
