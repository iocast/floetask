//! The status board: todos laid out in columns by workflow status.
//!
//! A column is either a status (`todo`, `doing`, a custom one) or `done`,
//! which holds completed todos, since done is not a status. Moving a todo to
//! a column sets its status, or completes it for `done`.

use std::collections::BTreeMap;

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
/// duplicates. A status column may carry a display name, for statuses that
/// only this board knows; global statuses are named in settings.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoardColumns {
    columns: Vec<Column>,
    labels: BTreeMap<String, String>,
}

impl Default for BoardColumns {
    fn default() -> Self {
        Self::from_keys(DEFAULT_COLUMNS)
    }
}

impl BoardColumns {
    /// Builds columns from configured keys, dropping invalid and duplicate
    /// ones. An empty result falls back to the default columns.
    pub fn from_keys<S: AsRef<str>>(keys: impl IntoIterator<Item = S>) -> Self {
        let mut board = Self {
            columns: Vec::new(),
            labels: BTreeMap::new(),
        };
        for key in keys {
            board.add(key.as_ref());
        }
        if board.columns.is_empty() {
            Self::default()
        } else {
            board
        }
    }

    pub fn columns(&self) -> &[Column] {
        &self.columns
    }

    pub fn keys(&self) -> Vec<String> {
        self.columns.iter().map(|column| column.key().to_owned()).collect()
    }

    /// The statuses that have a column, in column order.
    pub fn statuses(&self) -> impl Iterator<Item = &str> {
        self.columns.iter().filter_map(|column| match column {
            Column::Status(status) => Some(status.as_str()),
            Column::Done => None,
        })
    }

    /// Whether some column shows open todos with `status`.
    pub fn has_status(&self, status: &str) -> bool {
        self.columns
            .iter()
            .any(|column| matches!(column, Column::Status(s) if s == status))
    }

    /// Adds a column at the right end. Returns `false` for invalid keys and
    /// columns the board already has.
    pub fn add(&mut self, key: &str) -> bool {
        let new = Column::parse(key).filter(|column| !self.columns.contains(column));
        if let Some(column) = &new {
            self.columns.push(column.clone());
        }
        new.is_some()
    }

    /// Moves the column at `index` one step left (`-1`) or right (`1`).
    pub fn move_column(&mut self, index: usize, step: isize) {
        let target = index as isize + step;
        if index < self.columns.len() && target >= 0 && (target as usize) < self.columns.len() {
            self.columns.swap(index, target as usize);
        }
    }

    /// Removes the column at `index`, with its label. The last column stays,
    /// since a board needs one.
    pub fn remove(&mut self, index: usize) {
        if self.columns.len() > 1 && index < self.columns.len() {
            let column = self.columns.remove(index);
            self.labels.remove(column.key());
        }
    }

    /// Names a status column. A blank name, or the key itself, removes the
    /// label; `done` is always named by the app.
    pub fn set_label(&mut self, status: &str, label: &str) {
        let label = label.trim();
        if label.is_empty() || label == status {
            self.labels.remove(status);
        } else if self.has_status(status) {
            self.labels.insert(status.to_owned(), label.to_owned());
        }
    }

    /// The display name this board gives a status, if any.
    pub fn label(&self, status: &str) -> Option<&str> {
        self.labels.get(status).map(String::as_str)
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
    fn columns_are_edited_with_their_labels() {
        let mut columns = BoardColumns::default();
        assert!(columns.add("in-review"));
        assert!(!columns.add("in-review"));
        assert!(!columns.add("In review"));
        columns.set_label("in-review", " In review ");
        columns.set_label("done", "Finished");
        columns.set_label("missing", "Missing");
        assert_eq!(columns.label("in-review"), Some("In review"));
        assert_eq!(columns.label("done"), None);
        assert_eq!(columns.label("missing"), None);
        columns.move_column(4, -1);
        assert_eq!(columns.keys(), ["todo", "doing", "waiting", "in-review", "done"]);
        columns.remove(3);
        assert_eq!(columns.label("in-review"), None, "removing drops the label");
        assert_eq!(columns, BoardColumns::default());
        let mut single = BoardColumns::from_keys(["todo"]);
        single.remove(0);
        assert_eq!(single.keys(), ["todo"], "a board keeps one column");
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
