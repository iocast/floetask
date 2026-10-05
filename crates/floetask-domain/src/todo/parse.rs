use super::tokens::{TokenKind, classify, tokens};
use super::{KNOWN_KEYS, MULTILINE_SEPARATOR, Todo};
use crate::date::parse_iso;
use crate::priority::Priority;
use crate::recurrence::Recurrence;
use crate::status::is_valid_status;

pub(super) fn parse(line: &str) -> Todo {
    let mut rest = line;
    let mut complete = false;
    let mut completed = None;
    let mut created = None;
    let mut priority = None;

    if let Some(after_mark) = rest.strip_prefix("x ") {
        complete = true;
        rest = after_mark.trim_start_matches(' ');
        if let Some((date, after)) = take_date(rest) {
            completed = Some(date);
            rest = after;
            if let Some((date, after)) = take_date(rest) {
                created = Some(date);
                rest = after;
            }
        }
    } else {
        if let Some((marker, after)) = split_first_word(rest)
            && let Some(parsed) = Priority::parse_marker(marker)
        {
            priority = Some(parsed);
            rest = after;
        }
        if let Some((date, after)) = take_date(rest) {
            created = Some(date);
            rest = after;
        }
    }

    let description = rest.to_owned();
    let mut todo = Todo {
        raw: line.to_owned(),
        complete,
        completed,
        priority,
        created,
        body: visible_text(&description, false),
        description,
        projects: Vec::new(),
        contexts: Vec::new(),
        extensions: Vec::new(),
        due: None,
        threshold: None,
        rec: None,
        pm: None,
        hidden: false,
        stored_priority: None,
        status_tag: None,
    };
    read_tags(&mut todo);
    todo
}

fn read_tags(todo: &mut Todo) {
    let description = todo.description.clone();
    for token in tokens(&description) {
        match classify(token.text) {
            TokenKind::Project(name) => push_unique(&mut todo.projects, name),
            TokenKind::Context(name) => push_unique(&mut todo.contexts, name),
            TokenKind::Extension(key, value) => {
                todo.extensions.push((key.to_owned(), value.to_owned()));
                apply_known_extension(todo, key, value);
            }
            TokenKind::Word => {}
        }
    }
}

/// Fills the typed field for a known key. The first occurrence wins.
fn apply_known_extension(todo: &mut Todo, key: &str, value: &str) {
    match key {
        "due" if todo.due.is_none() => todo.due = parse_iso(value),
        "t" if todo.threshold.is_none() => todo.threshold = parse_iso(value),
        "rec" if todo.rec.is_none() => todo.rec = Recurrence::parse(value),
        "pm" if todo.pm.is_none() => todo.pm = value.parse().ok(),
        "h" => todo.hidden |= value == "1",
        "status" if todo.status_tag.is_none() && is_valid_status(value) => todo.status_tag = Some(value.to_owned()),
        "pri" if todo.stored_priority.is_none() => todo.stored_priority = value.chars().next().and_then(Priority::new),
        _ => {}
    }
}

fn push_unique(list: &mut Vec<String>, name: &str) {
    if !list.iter().any(|existing| existing == name) {
        list.push(name.to_owned());
    }
}

/// The description without tags, with line breaks restored. Contexts stay
/// in place when `keep_contexts` is set, since they often read as part of the
/// sentence ("talk to @alice").
pub(super) fn visible_text(description: &str, keep_contexts: bool) -> String {
    let keep = |word: &str| !is_tag(word) || keep_contexts && matches!(classify(word), TokenKind::Context(_));
    description
        .split(MULTILINE_SEPARATOR)
        .map(|line| {
            line.split([' ', '\t'])
                .filter(|word| !word.is_empty() && keep(word))
                .collect::<Vec<_>>()
                .join(" ")
        })
        .collect::<Vec<_>>()
        .join("\n")
        .trim()
        .to_owned()
}

fn is_tag(word: &str) -> bool {
    match classify(word) {
        TokenKind::Project(_) | TokenKind::Context(_) => true,
        TokenKind::Extension(key, _) => KNOWN_KEYS.contains(&key),
        TokenKind::Word => false,
    }
}

fn split_first_word(text: &str) -> Option<(&str, &str)> {
    let (word, rest) = text.split_once(' ').unwrap_or((text, ""));
    (!word.is_empty()).then(|| (word, rest.trim_start_matches(' ')))
}

fn take_date(text: &str) -> Option<(crate::date::Date, &str)> {
    let (word, rest) = split_first_word(text)?;
    parse_iso(word).map(|date| (date, rest))
}
