//! Edits that return a new `Todo`, touching only the affected tokens.

use super::Todo;
use super::tokens::{TokenKind, classify, remove_span, tokens};
use crate::date::{Date, format_iso};
use crate::priority::Priority;

/// The leading part of a todo.txt line.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct Head {
    pub complete: bool,
    pub completed: Option<Date>,
    pub priority: Option<Priority>,
    pub created: Option<Date>,
}

impl Todo {
    pub(super) fn head(&self) -> Head {
        Head {
            complete: self.complete,
            completed: self.completed,
            priority: self.priority,
            created: self.created,
        }
    }

    /// Rebuilds the line from a head and a description. If both are
    /// unchanged the original raw line is kept.
    pub(super) fn rebuild(&self, head: Head, description: &str) -> Todo {
        if head == self.head() && description == self.description {
            return self.clone();
        }
        let mut parts: Vec<String> = Vec::new();
        if head.complete {
            parts.push("x".to_owned());
            if let Some(completed) = head.completed {
                parts.push(format_iso(completed));
            }
        } else if let Some(priority) = head.priority {
            parts.push(format!("({priority})"));
        }
        if let Some(created) = head.created {
            parts.push(format_iso(created));
        }
        if !description.is_empty() {
            parts.push(description.to_owned());
        }
        Todo::parse(&parts.join(" "))
    }

    pub fn with_priority(&self, priority: Option<Priority>) -> Todo {
        self.rebuild(
            Head {
                priority,
                ..self.head()
            },
            &self.description,
        )
    }

    pub fn with_created(&self, created: Option<Date>) -> Todo {
        self.rebuild(Head { created, ..self.head() }, &self.description)
    }

    /// Sets `key:value`, replacing the first occurrence in place, or removes
    /// every occurrence when `value` is `None`.
    pub fn with_extension(&self, key: &str, value: Option<&str>) -> Todo {
        let description = set_extension(&self.description, key, value);
        self.rebuild(self.head(), &description)
    }

    pub fn with_date_extension(&self, key: &str, date: Option<Date>) -> Todo {
        self.with_extension(key, date.map(format_iso).as_deref())
    }

    pub fn with_project_renamed(&self, from: &str, to: &str) -> Todo {
        self.map_tokens(|kind| match kind {
            TokenKind::Project(name) if name == from => Some(Some(format!("+{to}"))),
            _ => None,
        })
    }

    pub fn with_context_renamed(&self, from: &str, to: &str) -> Todo {
        self.map_tokens(|kind| match kind {
            TokenKind::Context(name) if name == from => Some(Some(format!("@{to}"))),
            _ => None,
        })
    }

    pub fn without_project(&self, name: &str) -> Todo {
        self.map_tokens(|kind| match kind {
            TokenKind::Project(found) if found == name => Some(None),
            _ => None,
        })
    }

    pub fn without_context(&self, name: &str) -> Todo {
        self.map_tokens(|kind| match kind {
            TokenKind::Context(found) if found == name => Some(None),
            _ => None,
        })
    }

    /// Applies `change` to every description token. `Some(Some(text))`
    /// replaces the token, `Some(None)` removes it, `None` keeps it.
    fn map_tokens(&self, change: impl Fn(TokenKind<'_>) -> Option<Option<String>>) -> Todo {
        let mut description = self.description.clone();
        // Walk backwards so earlier spans stay valid while editing.
        let found: Vec<_> = tokens(&self.description)
            .into_iter()
            .filter_map(|token| change(classify(token.text)).map(|c| (token.start, token.end, c)))
            .collect();
        if found.is_empty() {
            return self.clone();
        }
        for (start, end, replacement) in found.into_iter().rev() {
            description = match replacement {
                Some(text) => format!("{}{}{}", &description[..start], text, &description[end..]),
                None => remove_span(&description, start, end),
            };
        }
        self.rebuild(self.head(), description.trim_end())
    }
}

fn set_extension(description: &str, key: &str, value: Option<&str>) -> String {
    let matches: Vec<_> = tokens(description)
        .into_iter()
        .filter(|token| matches!(classify(token.text), TokenKind::Extension(k, _) if k == key))
        .collect();

    match (value, matches.first()) {
        (Some(value), Some(first)) => format!(
            "{}{key}:{value}{}",
            &description[..first.start],
            &description[first.end..]
        ),
        (Some(value), None) if description.is_empty() => format!("{key}:{value}"),
        (Some(value), None) => format!("{} {key}:{value}", description.trim_end()),
        (None, _) => {
            let mut result = description.to_owned();
            for token in matches.iter().rev() {
                result = remove_span(&result, token.start, token.end);
            }
            result.trim_end().to_owned()
        }
    }
}
