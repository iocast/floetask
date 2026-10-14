//! The search bar and saved filters.

mod update;
mod view;

use floetask_application::{AppError, SavedFilter};
use floetask_domain::search::Query;

pub(crate) use view::{field, save_filter_dialog, saved_panel, summary};

#[derive(Debug, Clone)]
pub enum Message {
    Focus,
    Input(String),
    SubmitAsTodo,
    ToggleSaved,
    AskSaveFilter,
    SaveFilterName(String),
    ApplySaved(usize),
    AskDeleteSaved(usize),
    ToggleSuppress(usize),
    SavedFiltersLoaded(Result<Vec<SavedFilter>, AppError>),
}

/// What is typed in the search field and the query it stands for.
#[derive(Debug)]
pub struct SearchState {
    pub input: String,
    /// The last query that parsed, shown while an expression is incomplete.
    pub query: Query,
    pub saved_open: bool,
}

impl Default for SearchState {
    fn default() -> Self {
        Self {
            input: String::new(),
            query: Query::Empty,
            saved_open: false,
        }
    }
}
