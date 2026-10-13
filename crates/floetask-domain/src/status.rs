//! The `status:` extension: the workflow state of an open todo.
//!
//! todo.txt only knows open and done. `status:<value>` adds states such as
//! `doing` or `waiting`. A todo without the tag is `todo`, completion is never
//! a status, and values a user invents are kept and treated as open. See
//! features/todo-notes/status-extension.md for the full rules.

use std::collections::BTreeMap;

/// The status of an open todo without a `status:` tag.
pub const DEFAULT_STATUS: &str = "todo";

/// The four statuses with a defined meaning, in the default display order.
pub const BUILT_IN_STATUSES: [&str; 4] = ["doing", "todo", "waiting", "someday"];

/// Statuses hidden from the default list unless the user asks for them.
pub const HIDDEN_BY_DEFAULT: [&str; 1] = ["someday"];

/// A status value is a single lowercase token of `[a-z0-9_-]`.
pub fn is_valid_status(value: &str) -> bool {
    !value.is_empty()
        && value
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_' || c == '-')
}

/// The status key for a name a user types, such as `in-review` for
/// "In review": lowercase, words joined with `-`, accented letters spelled
/// in ASCII (`Prüfung` is `pruefung`), other characters dropped. Empty when
/// nothing usable is left.
pub fn key_from_label(label: &str) -> String {
    let mut key = String::new();
    for c in label.trim().to_lowercase().chars() {
        if c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_' {
            key.push(c);
        } else if let Some(ascii) = ascii_spelling(c) {
            key.push_str(ascii);
        } else if (c == '-' || c.is_whitespace()) && !key.is_empty() && !key.ends_with('-') {
            key.push('-');
        }
    }
    key.trim_end_matches('-').to_owned()
}

/// The ASCII spelling of an accented lowercase letter from the languages
/// floetask is translated into.
fn ascii_spelling(c: char) -> Option<&'static str> {
    Some(match c {
        'ä' => "ae",
        'ö' => "oe",
        'ü' => "ue",
        'ß' => "ss",
        'à' | 'á' | 'â' => "a",
        'ç' => "c",
        'è' | 'é' | 'ê' | 'ë' => "e",
        'ì' | 'í' | 'î' | 'ï' => "i",
        'ñ' => "n",
        'ò' | 'ó' | 'ô' => "o",
        'ù' | 'ú' | 'û' => "u",
        'œ' => "oe",
        _ => return None,
    })
}

/// A readable name for a key without a label: `in-review` becomes
/// "In review", so custom statuses look fine without configuration.
pub fn label_from_key(key: &str) -> String {
    let words = key.replace(['-', '_'], " ");
    let mut chars = words.trim().chars();
    match chars.next() {
        Some(first) => first.to_uppercase().chain(chars).collect(),
        None => String::new(),
    }
}

/// The statuses a user works with: their display and sort order, and which
/// ones the default list hides. Statuses found in a file but missing here
/// still work; they sort after the known ones and are always shown. A
/// status may carry a display name; the todo line only ever holds the key.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatusSet {
    order: Vec<String>,
    hidden: Vec<String>,
    labels: BTreeMap<String, String>,
}

impl Default for StatusSet {
    fn default() -> Self {
        Self::new(
            BUILT_IN_STATUSES.iter().map(|s| s.to_string()).collect(),
            HIDDEN_BY_DEFAULT.iter().map(|s| s.to_string()).collect(),
        )
    }
}

impl StatusSet {
    /// Builds a set from a user's list. Invalid and duplicate names are
    /// dropped, and built-in statuses the list leaves out are appended, so
    /// `todo`, `doing`, `waiting` and `someday` are always available.
    pub fn new(order: Vec<String>, hidden: Vec<String>) -> Self {
        let mut names: Vec<String> = Vec::new();
        let built_ins = BUILT_IN_STATUSES.iter().map(|s| s.to_string());
        for name in order.into_iter().chain(built_ins) {
            if is_valid_status(&name) && !names.contains(&name) {
                names.push(name);
            }
        }
        let hidden = hidden
            .into_iter()
            .filter(|name| is_valid_status(name) && name != DEFAULT_STATUS)
            .collect();
        Self {
            order: names,
            hidden,
            labels: BTreeMap::new(),
        }
    }

    /// Whether a status is one of the four with a defined meaning. These can
    /// be reordered and hidden but not removed.
    pub fn is_built_in(status: &str) -> bool {
        BUILT_IN_STATUSES.contains(&status)
    }

    /// Adds a custom status at the end of the order. Returns `false` for
    /// invalid or already known names.
    pub fn add(&mut self, status: &str) -> bool {
        let new = is_valid_status(status) && !self.order.iter().any(|name| name == status);
        if new {
            self.order.push(status.to_owned());
        }
        new
    }

    /// Removes a custom status. Built-in statuses stay.
    pub fn remove(&mut self, status: &str) {
        if !Self::is_built_in(status) {
            self.order.retain(|name| name != status);
            self.hidden.retain(|name| name != status);
            self.labels.remove(status);
        }
    }

    /// Sets the display name of a known status. A blank name, or the key
    /// itself, removes the label so the default name applies.
    pub fn set_label(&mut self, status: &str, label: &str) {
        let label = label.trim();
        if label.is_empty() || label == status {
            self.labels.remove(status);
        } else if self.order.iter().any(|name| name == status) {
            self.labels.insert(status.to_owned(), label.to_owned());
        }
    }

    /// The user's display name for a status, if they gave one.
    pub fn label(&self, status: &str) -> Option<&str> {
        self.labels.get(status).map(String::as_str)
    }

    /// Moves the status at `index` one step up (`-1`) or down (`1`).
    pub fn move_status(&mut self, index: usize, step: isize) {
        let target = index as isize + step;
        if index < self.order.len() && target >= 0 && (target as usize) < self.order.len() {
            self.order.swap(index, target as usize);
        }
    }

    /// Hides a status from the default list or shows it again. `todo` is
    /// always shown, since it is what a todo without a tag has.
    pub fn set_hidden(&mut self, status: &str, hidden: bool) {
        self.hidden.retain(|name| name != status);
        if hidden && status != DEFAULT_STATUS && is_valid_status(status) {
            self.hidden.push(status.to_owned());
        }
    }

    /// Every known status in display order, for pickers.
    pub fn names(&self) -> &[String] {
        &self.order
    }

    /// Statuses hidden from the default list.
    pub fn hidden(&self) -> &[String] {
        &self.hidden
    }

    pub fn is_hidden(&self, status: &str) -> bool {
        self.hidden.iter().any(|name| name == status)
    }

    /// Position in the display order. Unknown statuses rank after every
    /// known one.
    pub fn rank(&self, status: &str) -> usize {
        self.order
            .iter()
            .position(|name| name == status)
            .unwrap_or(self.order.len())
    }

    /// Orders two statuses by rank, then unknown ones by name.
    pub fn compare(&self, a: &str, b: &str) -> std::cmp::Ordering {
        self.rank(a).cmp(&self.rank(b)).then_with(|| a.cmp(b))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_values() {
        assert!(is_valid_status("in-review"));
        assert!(is_valid_status("stage_2"));
        assert!(!is_valid_status("Doing"));
        assert!(!is_valid_status(""));
        assert!(!is_valid_status("a b"));
    }

    #[test]
    fn keys_and_labels_convert() {
        assert_eq!(key_from_label("In review"), "in-review");
        assert_eq!(key_from_label("  Waiting -- on  Bob! "), "waiting-on-bob");
        assert_eq!(key_from_label("stage_2"), "stage_2");
        assert_eq!(key_from_label("?!"), "");
        assert_eq!(key_from_label("In Prüfung"), "in-pruefung");
        assert_eq!(key_from_label("En révision"), "en-revision");
        assert_eq!(label_from_key("in-review"), "In review");
        assert_eq!(label_from_key("stage_2"), "Stage 2");
    }

    #[test]
    fn labels_belong_to_known_statuses() {
        let mut set = StatusSet::default();
        set.add("in-review");
        set.set_label("in-review", " In Review ");
        set.set_label("unknown", "Unknown");
        assert_eq!(set.label("in-review"), Some("In Review"));
        assert_eq!(set.label("unknown"), None);
        set.set_label("in-review", "in-review");
        assert_eq!(set.label("in-review"), None, "the key is no label");
        set.set_label("in-review", "Peer review");
        set.remove("in-review");
        set.add("in-review");
        assert_eq!(set.label("in-review"), None, "removing drops the label");
    }

    #[test]
    fn default_order_follows_the_spec() {
        let set = StatusSet::default();
        assert_eq!(set.names(), ["doing", "todo", "waiting", "someday"]);
        assert!(set.is_hidden("someday"));
        assert!(!set.is_hidden("waiting"));
        assert!(set.rank("doing") < set.rank("todo"));
        assert_eq!(set.rank("review"), 4);
    }

    #[test]
    fn manage_custom_statuses() {
        let mut set = StatusSet::default();
        assert!(set.add("in-review"));
        assert!(!set.add("in-review"));
        assert!(!set.add("Bad"));
        set.move_status(4, -1);
        assert_eq!(set.names(), ["doing", "todo", "waiting", "in-review", "someday"]);
        set.set_hidden("in-review", true);
        set.set_hidden("todo", true);
        assert_eq!(set.hidden(), ["someday", "in-review"]);
        set.remove("in-review");
        set.remove("waiting");
        assert_eq!(set.names(), ["doing", "todo", "waiting", "someday"]);
        assert_eq!(set.hidden(), ["someday"]);
    }

    #[test]
    fn custom_list_keeps_built_ins_and_drops_invalid_names() {
        let set = StatusSet::new(
            vec!["review".into(), "doing".into(), "Bad".into(), "review".into()],
            vec!["blocked".into(), "todo".into()],
        );
        assert_eq!(set.names(), ["review", "doing", "todo", "waiting", "someday"]);
        assert_eq!(set.hidden(), ["blocked"]);
        assert_eq!(set.compare("zeta", "alpha"), std::cmp::Ordering::Greater);
    }
}
