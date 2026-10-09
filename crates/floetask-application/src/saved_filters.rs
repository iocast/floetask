//! Named search queries the user saved.

use floetask_domain::search::Query;
use floetask_domain::todo::Todo;
use floetask_domain::{Date, WeekStart};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SavedFilter {
    pub name: String,
    pub query: String,
    /// Todos matching this filter never trigger notifications.
    pub suppress_notifications: bool,
}

impl SavedFilter {
    /// Whether `todo` matches this filter's query (plain text or expression).
    pub fn matches(&self, todo: &Todo, today: Date, week_start: WeekStart) -> bool {
        match Query::interpret(&self.query) {
            floetask_domain::search::Interpretation::Ready(query) => {
                !query.is_empty() && query.matches(todo, today, week_start)
            }
            floetask_domain::search::Interpretation::Incomplete => false,
        }
    }
}

/// The filters a new installation starts with. Once the user saves or
/// deletes a filter, their own list replaces these.
pub fn default_filters() -> Vec<SavedFilter> {
    vec![SavedFilter {
        name: "overdue".to_owned(),
        // Due today or earlier.
        query: "due: < today+1d".to_owned(),
        suppress_notifications: false,
    }]
}

/// Adds or replaces a filter by name.
pub fn upsert(filters: &mut Vec<SavedFilter>, filter: SavedFilter) {
    match filters.iter_mut().find(|existing| existing.name == filter.name) {
        Some(existing) => *existing = filter,
        None => filters.push(filter),
    }
}

#[cfg(test)]
mod tests {
    use floetask_domain::date::parse_iso;

    use super::*;

    #[test]
    fn overdue_matches_todos_due_today_or_earlier() {
        let overdue = &default_filters()[0];
        let today = parse_iso("2026-10-06").unwrap();
        let matches = |todo: &Todo| overdue.matches(todo, today, WeekStart::Monday);
        let due = |date: &str| Todo::parse(&format!("Pay rent due:{date}"));
        assert!(matches(&due("2026-10-01")));
        assert!(matches(&due("2026-10-06")));
        assert!(!matches(&due("2026-10-07")));
        assert!(!matches(&Todo::parse("No date")));
    }
}
