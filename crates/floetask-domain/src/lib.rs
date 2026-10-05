//! Domain layer of floetask.
//!
//! Holds the todo.txt entities and every pure business rule: parsing and
//! round-tripping lines, completion and recurrence, the search language,
//! natural-language dates, and how a list is filtered, sorted and grouped.
//!
//! This crate performs no I/O and knows nothing about files, config or the
//! GUI. "Today" is always passed in, so every rule is deterministic and
//! unit-testable.

pub mod date;
pub mod document;
pub mod human_date;
pub mod listing;
pub mod natural_date;
pub mod note;
pub mod priority;
pub mod recurrence;
pub mod search;
pub mod todo;

pub use date::{Date, WeekStart};
pub use document::{DocumentError, TodoDocument};
pub use note::{NoteError, NoteName};
pub use priority::Priority;
pub use recurrence::{Recurrence, RecurrenceUnit};
pub use todo::Todo;
