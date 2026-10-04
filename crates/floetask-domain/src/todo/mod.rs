//! The `Todo` entity: one todo.txt line, parsed.
//!
//! A `Todo` keeps the raw line as its source of truth. Every edit builds a
//! new line by changing only the affected tokens and then re-parses it, so
//! untouched parts of the line are preserved byte for byte.

mod completion;
mod edit;
mod parse;
mod tokens;

use crate::date::Date;
use crate::priority::Priority;
use crate::recurrence::Recurrence;

pub use completion::Completion;

/// Separator stored in the file for line breaks inside one todo (the DLE control character).
pub const MULTILINE_SEPARATOR: char = '\u{10}';

/// Extension keys with a meaning in floetask. They are shown as chips, not in
/// the body text.
pub const KNOWN_KEYS: [&str; 6] = ["due", "t", "rec", "h", "pm", "pri"];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Todo {
    raw: String,
    complete: bool,
    completed: Option<Date>,
    priority: Option<Priority>,
    created: Option<Date>,
    /// Everything after the completion mark, priority and dates, verbatim.
    description: String,
    body: String,
    projects: Vec<String>,
    contexts: Vec<String>,
    extensions: Vec<(String, String)>,
    due: Option<Date>,
    threshold: Option<Date>,
    rec: Option<Recurrence>,
    pm: Option<u32>,
    hidden: bool,
    stored_priority: Option<Priority>,
}

impl Todo {
    /// Parses one line of a todo.txt file. Never fails: anything that is not
    /// recognised syntax is kept as text.
    pub fn parse(line: &str) -> Self {
        parse::parse(line)
    }

    /// Builds a todo from text typed by the user. Line breaks become the
    /// multi-line separator so the todo stays on one file line.
    pub fn from_user_text(text: &str) -> Self {
        let single_line: String = text
            .trim()
            .lines()
            .map(str::trim_end)
            .collect::<Vec<_>>()
            .join(&MULTILINE_SEPARATOR.to_string());
        Self::parse(&single_line)
    }

    /// The line exactly as stored in the file.
    pub fn raw(&self) -> &str {
        &self.raw
    }

    /// The line as the user edits it, with line breaks restored.
    pub fn editable_text(&self) -> String {
        self.raw.replace(MULTILINE_SEPARATOR, "\n")
    }

    pub fn is_complete(&self) -> bool {
        self.complete
    }

    pub fn completed(&self) -> Option<Date> {
        self.completed
    }

    pub fn priority(&self) -> Option<Priority> {
        self.priority
    }

    pub fn created(&self) -> Option<Date> {
        self.created
    }

    pub fn description(&self) -> &str {
        &self.description
    }

    /// Text for display: the description without projects, contexts and known
    /// extensions, with line breaks restored.
    pub fn body(&self) -> &str {
        &self.body
    }

    pub fn projects(&self) -> &[String] {
        &self.projects
    }

    pub fn contexts(&self) -> &[String] {
        &self.contexts
    }

    /// All `key:value` pairs in line order, including known keys.
    pub fn extensions(&self) -> &[(String, String)] {
        &self.extensions
    }

    pub fn extension(&self, key: &str) -> Option<&str> {
        self.extensions
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v.as_str())
    }

    pub fn due(&self) -> Option<Date> {
        self.due
    }

    pub fn threshold(&self) -> Option<Date> {
        self.threshold
    }

    pub fn recurrence(&self) -> Option<Recurrence> {
        self.rec
    }

    pub fn pomodoros(&self) -> Option<u32> {
        self.pm
    }

    /// `h:1`: hidden from the list, but still feeds autocomplete.
    pub fn is_hidden(&self) -> bool {
        self.hidden
    }

    /// Priority saved in `pri:X` when the todo was completed.
    pub fn stored_priority(&self) -> Option<Priority> {
        self.stored_priority
    }

    pub fn is_empty(&self) -> bool {
        self.raw.trim().is_empty()
    }
}

#[cfg(test)]
mod tests;
