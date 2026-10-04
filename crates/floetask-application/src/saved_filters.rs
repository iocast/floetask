//! Named search queries the user saved.

use floetask_domain::Date;
use floetask_domain::search::Query;
use floetask_domain::todo::Todo;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SavedFilter {
    pub name: String,
    pub query: String,
    /// Todos matching this filter never trigger notifications.
    pub suppress_notifications: bool,
}

impl SavedFilter {
    /// Whether `todo` matches this filter's query (plain text or expression).
    pub fn matches(&self, todo: &Todo, today: Date) -> bool {
        match Query::interpret(&self.query) {
            floetask_domain::search::Interpretation::Ready(query) => !query.is_empty() && query.matches(todo, today),
            floetask_domain::search::Interpretation::Incomplete => false,
        }
    }
}

/// Adds or replaces a filter by name.
pub fn upsert(filters: &mut Vec<SavedFilter>, filter: SavedFilter) {
    match filters.iter_mut().find(|existing| existing.name == filter.name) {
        Some(existing) => *existing = filter,
        None => filters.push(filter),
    }
}
