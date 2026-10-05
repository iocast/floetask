//! The `note:` extension: links a todo to a notes file in a `<file>-notes/`
//! folder next to the todo file (`todo.txt` keeps its notes in `todo-notes/`). See `features/note-extension.md`.
//!
//! This module only validates names and derives text; finding, creating and
//! opening the file is the application layer's job.

use thiserror::Error;

use crate::todo::Todo;

/// The extension key, as in `note:report-outline.md`.
pub const NOTE_KEY: &str = "note";

/// Suffix of the notes folder, added to the todo file name without its
/// extension.
const NOTES_FOLDER_SUFFIX: &str = "-notes";

/// Extension assumed when a note name has none.
const DEFAULT_EXTENSION: &str = "md";

/// Longest name [`suggest_name`] produces, without the extension.
const SUGGESTION_MAX_LEN: usize = 40;

#[derive(Debug, Clone, Error, PartialEq, Eq)]
pub enum NoteError {
    #[error("the note name is empty")]
    Empty,
    #[error("`{0}` is not allowed in a note name; use lowercase a-z, 0-9, `.`, `_` and `-`")]
    InvalidCharacter(char),
    #[error("a note name must stay inside the notes folder")]
    OutsideNotesFolder,
}

/// A validated `note:` value: a relative path inside the notes folder, with
/// the default extension added when the value has none.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct NoteName(String);

impl NoteName {
    /// Checks a `note:` value against the spec's filename rules: lowercase
    /// `[a-z0-9._-]` segments separated by `/`, no leading `/`, and no `.` or
    /// `..` segments that could leave the notes folder.
    pub fn parse(value: &str) -> Result<Self, NoteError> {
        if value.is_empty() {
            return Err(NoteError::Empty);
        }
        if let Some(bad) = value
            .chars()
            .find(|&c| !(c.is_ascii_lowercase() || c.is_ascii_digit() || "._-/".contains(c)))
        {
            return Err(NoteError::InvalidCharacter(bad));
        }
        if value
            .split('/')
            .any(|segment| segment.is_empty() || segment == "." || segment == "..")
        {
            return Err(NoteError::OutsideNotesFolder);
        }
        let file_name = value.rsplit('/').next().unwrap_or(value);
        let has_extension = file_name
            .rfind('.')
            .is_some_and(|dot| dot > 0 && dot + 1 < file_name.len());
        Ok(if has_extension {
            Self(value.to_owned())
        } else {
            Self(format!("{value}.{DEFAULT_EXTENSION}"))
        })
    }

    /// The path relative to the notes folder, `/`-separated.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// The path segments, for joining onto a platform path.
    pub fn segments(&self) -> impl Iterator<Item = &str> {
        self.0.split('/')
    }

    /// The same name with `-n` before the extension, to pick a file that
    /// does not exist yet: `write-report.md` becomes `write-report-2.md`.
    pub fn with_suffix(&self, n: u32) -> NoteName {
        let file_start = self.0.rfind('/').map_or(0, |slash| slash + 1);
        match self.0[file_start..].rfind('.').filter(|&dot| dot > 0) {
            Some(dot) => {
                let dot = file_start + dot;
                NoteName(format!("{}-{n}{}", &self.0[..dot], &self.0[dot..]))
            }
            None => NoteName(format!("{}-{n}", self.0)),
        }
    }
}

impl std::fmt::Display for NoteName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

/// The folder next to a todo file that its note names are relative to:
/// `todo.txt` gets `todo-notes`, `work.todo.txt` gets `work.todo-notes`.
/// Each todo file has its own folder, so two lists never share notes.
pub fn notes_folder(todo_file_name: &str) -> String {
    let stem = match todo_file_name.rfind('.') {
        Some(dot) if dot > 0 => &todo_file_name[..dot],
        _ => todo_file_name,
    };
    format!("{stem}{NOTES_FOLDER_SUFFIX}")
}

impl Todo {
    /// The raw `note:` value, valid or not.
    pub fn note(&self) -> Option<&str> {
        self.extension(NOTE_KEY)
    }

    /// The note this todo links to, validated.
    pub fn note_name(&self) -> Option<Result<NoteName, NoteError>> {
        self.note().map(NoteName::parse)
    }

    pub fn with_note(&self, name: Option<&NoteName>) -> Todo {
        self.with_extension(NOTE_KEY, name.map(NoteName::as_str))
    }
}

/// Name used when the todo text gives nothing to slugify.
const FALLBACK_NAME: &str = "note";

/// The name for a new note: the slugified todo text, or `note.md`.
pub fn default_name(todo: &Todo) -> NoteName {
    suggest_name(todo).unwrap_or_else(|| NoteName(format!("{FALLBACK_NAME}.{DEFAULT_EXTENSION}")))
}

/// Suggests a note name by slugifying the todo text, e.g. `write-report.md`.
/// Projects, contexts and known extensions are left out, since the body
/// excludes them.
pub fn suggest_name(todo: &Todo) -> Option<NoteName> {
    let first_line = todo.body().lines().next().unwrap_or_default();
    let mut slug = String::new();
    for c in first_line.chars().flat_map(char::to_lowercase) {
        if c.is_ascii_alphanumeric() {
            slug.push(c);
        } else if !slug.is_empty() && !slug.ends_with('-') {
            slug.push('-');
        }
        if slug.len() >= SUGGESTION_MAX_LEN {
            break;
        }
    }
    let slug = slug.trim_end_matches('-');
    NoteName::parse(slug).ok()
}

/// What a new note file starts with: the todo text as a heading.
pub fn new_note_content(todo: &Todo) -> String {
    let heading = todo.body().lines().next().unwrap_or_default().trim();
    if heading.is_empty() {
        String::new()
    } else {
        format!("# {heading}\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn name(value: &str) -> Result<String, NoteError> {
        NoteName::parse(value).map(|name| name.as_str().to_owned())
    }

    #[test]
    fn accepts_spec_names_and_adds_md() {
        assert_eq!(name("report-outline.md"), Ok("report-outline.md".into()));
        assert_eq!(name("dentist_call.txt"), Ok("dentist_call.txt".into()));
        assert_eq!(name("dentist-call"), Ok("dentist-call.md".into()));
        assert_eq!(name("work/q4.plan"), Ok("work/q4.plan".into()));
        assert_eq!(name(".hidden"), Ok(".hidden.md".into()));
    }

    #[test]
    fn rejects_names_outside_the_rules() {
        assert_eq!(name(""), Err(NoteError::Empty));
        assert_eq!(name("Report.md"), Err(NoteError::InvalidCharacter('R')));
        assert_eq!(name("a b"), Err(NoteError::InvalidCharacter(' ')));
        assert_eq!(name("a\\b"), Err(NoteError::InvalidCharacter('\\')));
        assert_eq!(name("/etc/passwd"), Err(NoteError::OutsideNotesFolder));
        assert_eq!(name("a/../../b"), Err(NoteError::OutsideNotesFolder));
        assert_eq!(name(".."), Err(NoteError::OutsideNotesFolder));
        assert_eq!(name("a//b"), Err(NoteError::OutsideNotesFolder));
    }

    #[test]
    fn reads_and_writes_the_note_extension() {
        let todo = Todo::parse("(A) Write report +work @office due:2026-10-10 note:report-outline.md");
        assert_eq!(todo.note(), Some("report-outline.md"));
        assert_eq!(todo.body(), "Write report");
        let renamed = todo.with_note(Some(&NoteName::parse("outline").unwrap()));
        assert_eq!(
            renamed.raw(),
            "(A) Write report +work @office due:2026-10-10 note:outline.md"
        );
        assert_eq!(
            renamed.with_note(None).raw(),
            "(A) Write report +work @office due:2026-10-10"
        );
        assert!(Todo::parse("Call note:Bad").note_name().unwrap().is_err());
    }

    #[test]
    fn suggests_slugified_names() {
        let suggest = |line: &str| suggest_name(&Todo::parse(line)).map(|n| n.as_str().to_owned());
        assert_eq!(
            suggest("(A) Write report +work due:2026-10-10"),
            Some("write-report.md".into())
        );
        assert_eq!(
            suggest("Call Dr. Müller, re: X-ray!"),
            Some("call-dr-m-ller-re-x-ray.md".into())
        );
        assert_eq!(suggest("+work @office"), None);
        let long = suggest(&"word ".repeat(30)).unwrap();
        assert!(long.len() <= SUGGESTION_MAX_LEN + 3 && !long.contains("-.md"));
    }

    #[test]
    fn notes_folder_is_named_after_the_todo_file() {
        assert_eq!(notes_folder("todo.txt"), "todo-notes");
        assert_eq!(notes_folder("work.todo.txt"), "work.todo-notes");
        assert_eq!(notes_folder("inbox"), "inbox-notes");
        assert_eq!(notes_folder(".todo"), ".todo-notes");
    }

    #[test]
    fn suffixes_go_before_the_extension() {
        let suffixed = |value: &str| NoteName::parse(value).unwrap().with_suffix(2).to_string();
        assert_eq!(suffixed("write-report.md"), "write-report-2.md");
        assert_eq!(suffixed("work/q4.plan.md"), "work/q4.plan-2.md");
        assert_eq!(suffixed(".hidden.md"), ".hidden-2.md");
        assert_eq!(default_name(&Todo::parse("+work")).to_string(), "note.md");
    }

    #[test]
    fn new_notes_start_with_the_todo_as_heading() {
        let todo = Todo::parse("(A) Write report +work note:r.md");
        assert_eq!(new_note_content(&todo), "# Write report\n");
    }
}
