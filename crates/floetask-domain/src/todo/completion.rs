//! Completing and un-completing todos, including recurrence.

use super::Todo;
use super::edit::Head;
use crate::date::{Date, add_days};
use crate::priority::Priority;

/// Result of completing a todo.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Completion {
    pub completed: Todo,
    /// The next occurrence, when the todo has a `rec:` rule.
    pub next: Option<Todo>,
}

impl Todo {
    /// Marks the todo complete on `today`.
    ///
    /// The creation date is set to today when missing, the priority moves to
    /// `pri:X`, the `status:` tag is removed (done is not a status), and a
    /// recurring todo produces its next occurrence, which starts as `todo`.
    pub fn complete(&self, today: Date) -> Completion {
        if self.complete {
            return Completion {
                completed: self.clone(),
                next: None,
            };
        }
        let next = self.next_occurrence(today);
        let without_status = self.with_status(None);
        let with_stored_priority = match self.priority {
            Some(priority) => without_status.with_extension("pri", Some(&priority.to_string())),
            None => without_status,
        };
        let completed = with_stored_priority.rebuild(
            Head {
                complete: true,
                completed: Some(today),
                priority: None,
                created: Some(self.created.unwrap_or(today)),
            },
            &with_stored_priority.description,
        );
        Completion { completed, next }
    }

    /// Reverses completion and restores the priority stored in `pri:`.
    pub fn uncomplete(&self) -> Todo {
        if !self.complete {
            return self.clone();
        }
        let priority: Option<Priority> = self.stored_priority;
        let without_stored = self.with_extension("pri", None);
        without_stored.rebuild(
            Head {
                complete: false,
                completed: None,
                priority,
                created: self.created,
            },
            &without_stored.description,
        )
    }

    pub fn toggle_complete(&self, today: Date) -> Completion {
        if self.complete {
            Completion {
                completed: self.uncomplete(),
                next: None,
            }
        } else {
            self.complete(today)
        }
    }

    /// The copy created when a recurring todo is completed on `today`.
    fn next_occurrence(&self, today: Date) -> Option<Todo> {
        let rec = self.rec?;
        let (due, threshold) = if rec.strict {
            (
                rec.advance(self.due.unwrap_or(today)),
                self.threshold.map(|t| rec.advance(t)),
            )
        } else {
            let due = rec.advance(today);
            // Keep the original distance between threshold and due date.
            let threshold = self.threshold.map(|t| {
                let gap = (self.due.unwrap_or(today) - t).num_days();
                add_days(due, -gap)
            });
            (due, threshold)
        };

        let mut next = self
            .with_status(None)
            .with_date_extension("due", Some(due))
            .with_created(Some(today));
        if let Some(threshold) = threshold {
            next = next.with_date_extension("t", Some(threshold));
        }
        Some(next)
    }
}
