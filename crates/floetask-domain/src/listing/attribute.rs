use std::cmp::Ordering;

use super::DateContext;
use crate::date::{Date, format_iso};
use crate::human_date;
use crate::status::StatusSet;
use crate::todo::Todo;

/// A todo attribute that can be filtered, sorted and grouped by.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Attribute {
    Priority,
    Status,
    Projects,
    Contexts,
    Due,
    Threshold,
    Recurrence,
    Pomodoro,
    Created,
    Completed,
}

impl Attribute {
    /// Drawer order.
    pub const ALL: [Attribute; 10] = [
        Attribute::Priority,
        Attribute::Status,
        Attribute::Projects,
        Attribute::Contexts,
        Attribute::Due,
        Attribute::Threshold,
        Attribute::Recurrence,
        Attribute::Pomodoro,
        Attribute::Created,
        Attribute::Completed,
    ];

    /// Stable identifier used in config files and as translation key.
    pub fn key(self) -> &'static str {
        match self {
            Attribute::Priority => "priority",
            Attribute::Status => "status",
            Attribute::Projects => "projects",
            Attribute::Contexts => "contexts",
            Attribute::Due => "due",
            Attribute::Threshold => "t",
            Attribute::Recurrence => "rec",
            Attribute::Pomodoro => "pm",
            Attribute::Created => "created",
            Attribute::Completed => "completed",
        }
    }

    pub fn from_key(key: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|attribute| attribute.key() == key)
    }

    pub fn is_date(self) -> bool {
        matches!(
            self,
            Attribute::Due | Attribute::Threshold | Attribute::Created | Attribute::Completed
        )
    }

    /// Projects and contexts can be renamed or removed across a file.
    pub fn is_renamable(self) -> bool {
        matches!(self, Attribute::Projects | Attribute::Contexts)
    }

    pub fn date_of(self, todo: &Todo) -> Option<Date> {
        match self {
            Attribute::Due => todo.due(),
            Attribute::Threshold => todo.threshold(),
            Attribute::Created => todo.created(),
            Attribute::Completed => todo.completed(),
            _ => None,
        }
    }

    /// The values a todo has for this attribute, as shown in the drawer and
    /// used by attribute filters. Dates in human-friendly mode yield every
    /// bucket they fall in.
    pub fn values(self, todo: &Todo, dates: &DateContext) -> Vec<String> {
        match self {
            Attribute::Priority => todo.priority().map(|p| p.to_string()).into_iter().collect(),
            Attribute::Status => todo.status().map(str::to_owned).into_iter().collect(),
            Attribute::Projects => todo.projects().to_vec(),
            Attribute::Contexts => todo.contexts().to_vec(),
            Attribute::Recurrence => todo.recurrence().map(|r| r.to_string()).into_iter().collect(),
            Attribute::Pomodoro => todo.pomodoros().map(|n| n.to_string()).into_iter().collect(),
            date_attribute => match date_attribute.date_of(todo) {
                None => Vec::new(),
                Some(date) if dates.human_friendly => {
                    let is_due = date_attribute == Attribute::Due;
                    let buckets = human_date::buckets(date, dates.today, dates.week_start, is_due);
                    if buckets.is_empty() {
                        vec![format_iso(date)]
                    } else {
                        buckets.into_iter().map(|b| b.key().to_owned()).collect()
                    }
                }
                Some(date) => vec![format_iso(date)],
            },
        }
    }

    /// Compares two todos by this attribute in ascending order. Todos missing
    /// the attribute are `None`, so the caller can keep them last. Statuses
    /// follow the user's status order.
    pub fn compare(self, a: &Todo, b: &Todo, statuses: &StatusSet) -> Option<Ordering> {
        fn both<T>(a: Option<T>, b: Option<T>, cmp: impl Fn(&T, &T) -> Ordering) -> Option<Ordering> {
            match (a, b) {
                (Some(a), Some(b)) => Some(cmp(&a, &b)),
                _ => None,
            }
        }
        let first_name = |names: &[String]| names.first().map(|n| n.to_lowercase());
        match self {
            Attribute::Priority => both(a.priority(), b.priority(), Ord::cmp),
            Attribute::Status => both(a.status(), b.status(), |x, y| statuses.compare(x, y)),
            Attribute::Projects => both(first_name(a.projects()), first_name(b.projects()), Ord::cmp),
            Attribute::Contexts => both(first_name(a.contexts()), first_name(b.contexts()), Ord::cmp),
            Attribute::Recurrence => both(a.recurrence(), b.recurrence(), |x, y| x.semantic_cmp(y)),
            Attribute::Pomodoro => both(a.pomodoros(), b.pomodoros(), Ord::cmp),
            date_attribute => both(date_attribute.date_of(a), date_attribute.date_of(b), Ord::cmp),
        }
    }

    pub fn has_value(self, todo: &Todo) -> bool {
        match self {
            Attribute::Priority => todo.priority().is_some(),
            Attribute::Status => todo.status().is_some(),
            Attribute::Projects => !todo.projects().is_empty(),
            Attribute::Contexts => !todo.contexts().is_empty(),
            Attribute::Recurrence => todo.recurrence().is_some(),
            Attribute::Pomodoro => todo.pomodoros().is_some(),
            date_attribute => date_attribute.date_of(todo).is_some(),
        }
    }
}
