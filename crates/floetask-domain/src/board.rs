//! The status board: todos laid out in columns by workflow status.
//!
//! A column is either a status (`todo`, `doing`, a custom one) or `done`,
//! which holds completed todos, since done is not a status. Moving a todo to
//! a column sets its status, or completes it for `done`.

use crate::date::Date;
use crate::status::{DEFAULT_STATUS, is_valid_status};
use crate::todo::Todo;

/// Key of the column for completed todos.
pub const DONE_COLUMN: &str = "done";

/// Columns of a new board, left to right.
pub const DEFAULT_COLUMNS: [&str; 4] = ["todo", "doing", "waiting", DONE_COLUMN];

/// One board column.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Column {
    /// Open todos with this status.
    Status(String),
    /// Completed todos.
    Done,
}

impl Column {
    /// Reads a column key from configuration. `None` for names that are not
    /// valid status values.
    pub fn parse(key: &str) -> Option<Self> {
        match key {
            DONE_COLUMN => Some(Column::Done),
            status if is_valid_status(status) => Some(Column::Status(status.to_owned())),
            _ => None,
        }
    }

    /// The key written to configuration and used as the column title key.
    pub fn key(&self) -> &str {
        match self {
            Column::Status(status) => status,
            Column::Done => DONE_COLUMN,
        }
    }

    /// Whether a todo belongs in this column.
    pub fn holds(&self, todo: &Todo) -> bool {
        match self {
            Column::Status(status) => todo.status() == Some(status.as_str()),
            Column::Done => todo.is_complete(),
        }
    }
}

/// The ordered columns of one file's board. Never empty and without
/// duplicates.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoardColumns(Vec<Column>);

impl Default for BoardColumns {
    fn default() -> Self {
        Self::from_keys(DEFAULT_COLUMNS)
    }
}

impl BoardColumns {
    /// Builds columns from configured keys, dropping invalid and duplicate
    /// ones. An empty result falls back to the default columns.
    pub fn from_keys<S: AsRef<str>>(keys: impl IntoIterator<Item = S>) -> Self {
        let mut columns: Vec<Column> = Vec::new();
        for column in keys.into_iter().filter_map(|key| Column::parse(key.as_ref())) {
            if !columns.contains(&column) {
                columns.push(column);
            }
        }
        if columns.is_empty() {
            Self::default()
        } else {
            Self(columns)
        }
    }

    pub fn columns(&self) -> &[Column] {
        &self.0
    }

    pub fn keys(&self) -> Vec<String> {
        self.0.iter().map(|column| column.key().to_owned()).collect()
    }

    /// Whether some column shows open todos with `status`.
    pub fn has_status(&self, status: &str) -> bool {
        self.0
            .iter()
            .any(|column| matches!(column, Column::Status(s) if s == status))
    }
}

impl Todo {
    /// The lines that replace this todo when it is dropped on `column`:
    /// completing it for `done` (a recurring todo adds its next occurrence),
    /// otherwise reopening it if needed and setting the status in place.
    pub fn moved_to(&self, column: &Column, today: Date) -> Vec<Todo> {
        match column {
            Column::Done if self.is_complete() => vec![self.clone()],
            Column::Done => {
                let completion = self.complete(today);
                std::iter::once(completion.completed).chain(completion.next).collect()
            }
            Column::Status(status) => {
                let open = if self.is_complete() {
                    self.uncomplete()
                } else {
                    self.clone()
                };
                let status = Some(status.as_str()).filter(|s| *s != DEFAULT_STATUS);
                vec![open.with_status(status)]
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::date::parse_iso;

    fn today() -> Date {
        parse_iso("2026-10-05").unwrap()
    }

    #[test]
    fn columns_from_keys_validate_and_fall_back() {
        let columns = BoardColumns::from_keys(["doing", "Bad", "in-review", "done", "doing"]);
        assert_eq!(columns.keys(), ["doing", "in-review", "done"]);
        assert!(columns.has_status("in-review"));
        assert_eq!(BoardColumns::from_keys(["Nope"]), BoardColumns::default());
        assert_eq!(BoardColumns::default().keys(), ["todo", "doing", "waiting", "done"]);
    }

    #[test]
    fn moving_sets_status_in_place() {
        let todo = Todo::parse("(A) Write report status:doing +work");
        let moved = todo.moved_to(&Column::Status("waiting".into()), today());
        assert_eq!(moved[0].raw(), "(A) Write report status:waiting +work");
        let back = moved[0].moved_to(&Column::Status("todo".into()), today());
        assert_eq!(back[0].raw(), "(A) Write report +work");
    }

    #[test]
    fn moving_to_done_completes_and_back_reopens() {
        let todo = Todo::parse("(A) Water plants status:doing rec:1w");
        let moved = todo.moved_to(&Column::Done, today());
        assert_eq!(moved.len(), 2);
        assert!(moved[0].is_complete());
        assert_eq!(moved[1].status(), Some("todo"));

        let reopened = moved[0].moved_to(&Column::Status("doing".into()), today());
        assert!(!reopened[0].is_complete());
        assert_eq!(reopened[0].status(), Some("doing"));
        assert_eq!(reopened[0].priority(), todo.priority());
    }
}
