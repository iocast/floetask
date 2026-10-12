//! The `note:` extension: links a todo to a notes file in a `<file>-notes/`
//! folder next to the todo file (`todo.txt` keeps its notes in `todo-notes/`),
//! or to a Markdown file elsewhere by its absolute path. See
//! `features/todo-notes/note-extension.md`.
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

/// Extensions of the files that can be linked as notes: Markdown and text.
const LINKABLE_EXTENSIONS: [&str; 3] = ["md", "markdown", "txt"];

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

    /// The name linking to an existing file in the notes folder, given its
    /// `/`-separated path relative to that folder. `None` for files that are
    /// not Markdown or text, and for names breaking the rules. A name without
    /// extension is refused too: `parse` would add `.md` and point elsewhere.
    pub fn of_file(relative: &str) -> Option<NoteName> {
        let name = Self::parse(relative).ok().filter(|name| name.0 == relative)?;
        let extension = relative.rsplit_once('.').map(|(_, extension)| extension)?;
        LINKABLE_EXTENSIONS.contains(&extension).then_some(name)
    }

    /// A valid name for a file moved into the notes folder: its name
    /// slugified like [`suggest_name`], keeping a Markdown or text extension.
    /// `My Plan.MD` becomes `my-plan.md`.
    pub fn from_file_name(file_name: &str) -> NoteName {
        let (stem, extension) = match file_name.rsplit_once('.') {
            Some((stem, extension)) if !stem.is_empty() => (stem, extension.to_lowercase()),
            _ => (file_name, String::new()),
        };
        let extension = if LINKABLE_EXTENSIONS.contains(&extension.as_str()) {
            extension
        } else {
            DEFAULT_EXTENSION.to_owned()
        };
        let stem = slugify(stem);
        let stem = if stem.is_empty() { FALLBACK_NAME } else { &stem };
        NoteName(format!("{stem}.{extension}"))
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

/// Where a `note:` value points: a file in the todo file's notes folder,
/// or a file the user keeps somewhere else, linked by its absolute path.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum NoteLink {
    Folder(NoteName),
    /// An absolute, `/`-separated path. Its `note:` value is percent-encoded
    /// to stay one todo.txt extension (see [`encode`]).
    External(String),
}

impl NoteLink {
    /// Reads a `note:` value: an absolute path (`/...`, `C%3A/...`) links a
    /// file outside the notes folder, anything else is a folder name.
    pub fn parse(value: &str) -> Result<Self, NoteError> {
        let path = decode(value);
        if is_absolute(&path) {
            Ok(Self::External(path))
        } else {
            NoteName::parse(value).map(Self::Folder)
        }
    }

    /// A link to a file outside the notes folder, given its absolute path
    /// (`\` separators are accepted). `None` for a relative path.
    pub fn external(path: &str) -> Option<Self> {
        let path = path.replace('\\', "/");
        is_absolute(&path).then_some(Self::External(path))
    }

    /// The `note:` value for this link.
    pub fn value(&self) -> String {
        match self {
            Self::Folder(name) => name.as_str().to_owned(),
            Self::External(path) => encode(path),
        }
    }

    /// The file's name without its folders, for a short label.
    pub fn file_name(&self) -> &str {
        let path = match self {
            Self::Folder(name) => name.as_str(),
            Self::External(path) => path,
        };
        path.rsplit('/').next().unwrap_or(path)
    }
}

impl std::fmt::Display for NoteLink {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Folder(name) => write!(f, "{name}"),
            Self::External(path) => f.write_str(path),
        }
    }
}

/// A Unix path, a Windows drive path or a UNC path (`//server/share`).
fn is_absolute(path: &str) -> bool {
    let bytes = path.as_bytes();
    path.starts_with('/')
        || (bytes.len() > 2 && bytes[0].is_ascii_alphabetic() && bytes[1] == b':' && matches!(bytes[2], b'/' | b'\\'))
}

/// Percent-encodes a path so it reads back as one todo.txt extension value:
/// whitespace splits tokens, a `:` turns the token into text, and so does a
/// value starting with `//` (it looks like a URL), as a UNC path does.
fn encode(path: &str) -> String {
    let mut encoded = String::with_capacity(path.len());
    for (i, c) in path.chars().enumerate() {
        let unc_start = i == 0 && path.starts_with("//");
        if c == '%' || c == ':' || c.is_whitespace() || unc_start {
            let mut buffer = [0; 4];
            for byte in c.encode_utf8(&mut buffer).bytes() {
                encoded.push_str(&format!("%{byte:02X}"));
            }
        } else {
            encoded.push(c);
        }
    }
    encoded
}

/// Undoes [`encode`]. A `%` not followed by two hex digits stays as it is.
fn decode(value: &str) -> String {
    let bytes = value.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        let hex = value.get(i + 1..i + 3).and_then(|hex| u8::from_str_radix(hex, 16).ok());
        match (bytes[i], hex) {
            (b'%', Some(byte)) => {
                decoded.push(byte);
                i += 3;
            }
            (byte, _) => {
                decoded.push(byte);
                i += 1;
            }
        }
    }
    String::from_utf8(decoded).unwrap_or_else(|_| value.to_owned())
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

    /// Where this todo's note is, in the notes folder or elsewhere.
    pub fn note_link(&self) -> Option<Result<NoteLink, NoteError>> {
        self.note().map(NoteLink::parse)
    }

    pub fn with_note_link(&self, link: Option<&NoteLink>) -> Todo {
        self.with_extension(NOTE_KEY, link.map(NoteLink::value).as_deref())
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
    NoteName::parse(&slugify(first_line)).ok()
}

/// Lowercase ASCII letters and digits joined by single `-`, at most about
/// [`SUGGESTION_MAX_LEN`] long.
fn slugify(text: &str) -> String {
    let mut slug = String::new();
    for c in text.chars().flat_map(char::to_lowercase) {
        if c.is_ascii_alphanumeric() {
            slug.push(c);
        } else if !slug.is_empty() && !slug.ends_with('-') {
            slug.push('-');
        }
        if slug.len() >= SUGGESTION_MAX_LEN {
            break;
        }
    }
    slug.trim_end_matches('-').to_owned()
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
    fn absolute_values_link_files_outside_the_notes_folder() {
        let link = |value: &str| NoteLink::parse(value).unwrap();
        assert_eq!(link("plan"), NoteLink::Folder(NoteName::parse("plan.md").unwrap()));
        assert_eq!(
            link("C%3A/Users/Me/My%20Notes/Plan.md"),
            NoteLink::External("C:/Users/Me/My Notes/Plan.md".into())
        );
        assert_eq!(
            link("/home/me/100%25.md"),
            NoteLink::External("/home/me/100%.md".into())
        );
        assert_eq!(link("/a/b%zz.md"), NoteLink::External("/a/b%zz.md".into()));
        assert!(NoteLink::parse("../secret").is_err());

        let external = NoteLink::external(r"C:\Users\Me\My Notes\Plan.md").unwrap();
        assert_eq!(external.value(), "C%3A/Users/Me/My%20Notes/Plan.md");
        let unc = NoteLink::external("//server/share/a.md").unwrap();
        assert_eq!(unc.value(), "%2F/server/share/a.md");
        assert_eq!(external.file_name(), "Plan.md");
        assert_eq!(NoteLink::external("notes/plan.md"), None);

        let todo = Todo::parse("Plan offsite +work").with_note_link(Some(&external));
        assert_eq!(todo.raw(), "Plan offsite +work note:C%3A/Users/Me/My%20Notes/Plan.md");
        assert_eq!(todo.note_link(), Some(Ok(external)));
        assert_eq!(todo.body(), "Plan offsite");
    }

    #[test]
    fn moved_files_get_valid_names() {
        let name = |file: &str| NoteName::from_file_name(file).to_string();
        assert_eq!(name("My Plan.MD"), "my-plan.md");
        assert_eq!(name("notes.txt"), "notes.txt");
        assert_eq!(name("Über.markdown"), "ber.markdown");
        assert_eq!(name("draft"), "draft.md");
        assert_eq!(name("scan.pdf"), "scan.md");
        assert_eq!(name("!!!.md"), "note.md");
    }

    proptest::proptest! {
        #[test]
        fn external_paths_survive_the_round_trip(path in "/[^\u{0}]{0,40}") {
            let link = NoteLink::external(&path).unwrap();
            let value = link.value();
            proptest::prop_assert!(!value.contains(char::is_whitespace));
            let todo = Todo::parse("Task").with_note_link(Some(&link));
            proptest::prop_assert_eq!(todo.note_link(), Some(Ok(link)));
        }
    }

    #[test]
    fn links_only_markdown_and_text_files() {
        let of_file = |relative: &str| NoteName::of_file(relative).map(|name| name.to_string());
        assert_eq!(of_file("plan.md"), Some("plan.md".into()));
        assert_eq!(of_file("work/q4.markdown"), Some("work/q4.markdown".into()));
        assert_eq!(of_file("call.txt"), Some("call.txt".into()));
        assert_eq!(of_file("plan.md.bak"), None);
        assert_eq!(of_file("README"), None);
        assert_eq!(of_file("readme"), None);
        assert_eq!(of_file("My Plan.md"), None);
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
