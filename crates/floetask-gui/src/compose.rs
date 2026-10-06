//! Text logic behind the add/edit dialog: `+project` / `@context`
//! autocomplete and the pickers that rewrite the todo text in place.
//!
//! Kept free of widgets so it can be unit-tested.

use std::collections::BTreeSet;

use floetask_domain::todo::Todo;

const MAX_SUGGESTIONS: usize = 8;

/// The `+word` or `@word` being typed at the cursor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypedTag {
    pub sigil: char,
    pub line: usize,
    /// Byte range of the whole token, sigil included.
    pub start: usize,
    pub end: usize,
    pub prefix: String,
}

/// Finds the tag token around byte `column` of `line_text`.
pub fn typed_tag(line_text: &str, line: usize, column: usize) -> Option<TypedTag> {
    let column = column.min(line_text.len());
    if !line_text.is_char_boundary(column) {
        return None;
    }
    // Whitespace can be wider than one byte (a non-breaking space).
    let start = line_text[..column]
        .char_indices()
        .rfind(|(_, c)| c.is_whitespace())
        .map(|(i, c)| i + c.len_utf8())
        .unwrap_or(0);
    // The cursor must be after the sigil: right before `+work` nothing is
    // being typed yet.
    if column <= start {
        return None;
    }
    let end = line_text[column..]
        .find(char::is_whitespace)
        .map(|i| column + i)
        .unwrap_or(line_text.len());
    let token = &line_text[start..end];
    let sigil = token.chars().next().filter(|c| *c == '+' || *c == '@')?;
    Some(TypedTag {
        sigil,
        line,
        start,
        end,
        prefix: token[1..column - start].to_owned(),
    })
}

/// Known names matching what was typed: prefix matches first, then other
/// substring matches, case-insensitive. An exact match is left out.
pub fn suggestions(prefix: &str, names: &BTreeSet<String>) -> Vec<String> {
    let needle = prefix.to_lowercase();
    let (mut starts, mut contains): (Vec<&String>, Vec<&String>) = names
        .iter()
        .filter(|name| name.as_str() != prefix && name.to_lowercase().contains(&needle))
        .partition(|name| name.to_lowercase().starts_with(&needle));
    starts.append(&mut contains);
    starts.into_iter().take(MAX_SUGGESTIONS).cloned().collect()
}

/// Replaces the typed tag with the chosen name. Returns the new text and the
/// cursor position (line, byte column) right after the inserted name.
pub fn insert_suggestion(text: &str, tag: &TypedTag, name: &str) -> (String, usize, usize) {
    let mut lines: Vec<String> = text.split('\n').map(str::to_owned).collect();
    let Some(line) = lines.get_mut(tag.line) else {
        return (text.to_owned(), 0, 0);
    };
    let replacement = format!("{}{} ", tag.sigil, name);
    let rest = line[tag.end..].trim_start().to_owned();
    let column = tag.start + replacement.len();
    *line = format!("{}{}{}", &line[..tag.start], replacement, rest);
    (lines.join("\n"), tag.line, column)
}

/// Applies a todo edit (priority, dates, recurrence, ...) to the typed text.
pub fn edit_text(text: &str, edit: impl FnOnce(&Todo) -> Todo) -> String {
    let todo = Todo::from_user_text(text);
    edit(&todo).editable_text()
}

#[cfg(test)]
mod tests {
    use super::*;
    use floetask_domain::Priority;

    fn names(list: &[&str]) -> BTreeSet<String> {
        list.iter().map(|s| (*s).to_owned()).collect()
    }

    #[test]
    fn finds_tag_at_cursor() {
        let tag = typed_tag("Call +wo now", 0, 8).unwrap();
        assert_eq!((tag.sigil, tag.start, tag.end, tag.prefix.as_str()), ('+', 5, 8, "wo"));
        assert!(typed_tag("Call mom", 0, 4).is_none());
        // Right before a sigil, e.g. after moving the cursor left past `+`.
        assert!(typed_tag("Call +work", 0, 5).is_none());
        assert!(typed_tag("+work", 0, 0).is_none());
        assert!(typed_tag("", 0, 0).is_none());
        // Multi-byte whitespace and text before the tag.
        let tag = typed_tag("Grüße\u{a0}@bü", 0, 13).unwrap();
        assert_eq!((tag.sigil, tag.prefix.as_str()), ('@', "bü"));
        let tag = typed_tag("@", 2, 1).unwrap();
        assert_eq!(tag.prefix, "");
    }

    #[test]
    fn never_panics_at_any_cursor_position() {
        for line in ["Call +work @home", "+@ @+ ++", "Grüße\u{a0}@bü +", " @", "x\t+y"] {
            for column in 0..=line.len() + 2 {
                let _ = typed_tag(line, 0, column);
            }
        }
    }

    #[test]
    fn suggests_prefix_matches_first() {
        let known = names(&["work", "homework", "Workshop", "play"]);
        assert_eq!(suggestions("wor", &known), vec!["Workshop", "work", "homework"]);
        assert_eq!(suggestions("work", &known), vec!["Workshop", "homework"]);
    }

    #[test]
    fn inserts_name_and_moves_cursor() {
        let text = "first line\nCall +wo now";
        let tag = typed_tag("Call +wo now", 1, 8).unwrap();
        let (new_text, line, column) = insert_suggestion(text, &tag, "work");
        assert_eq!(new_text, "first line\nCall +work now");
        assert_eq!((line, column), (1, 11));
    }

    #[test]
    fn edits_text_through_todo() {
        let text = edit_text("Pay rent\nby check", |todo| todo.with_priority(Priority::new('A')));
        assert_eq!(text, "(A) Pay rent\nby check");
    }
}
