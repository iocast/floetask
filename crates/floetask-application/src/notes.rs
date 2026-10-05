//! Use cases for the `note:` extension: where a todo's note file lives,
//! whether it exists, and opening it (creating it first when missing).
//!
//! Note files are never deleted or moved: completing, archiving and
//! deleting a todo leave them where they are.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use floetask_domain::note::{NOTES_DIR, new_note_content};
use floetask_domain::todo::Todo;
use floetask_domain::{NoteName, TodoDocument};

use crate::error::AppError;
use crate::ports::{Desktop, FileSystem};
use crate::todo_files::TodoRef;

/// What a todo's `note:` value points at.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NoteStatus {
    Present,
    /// A valid name whose file does not exist yet: a broken link, not an
    /// error. Opening it creates the file.
    Missing,
    /// A value breaking the filename rules. Never opened.
    Invalid,
}

#[derive(Clone)]
pub struct NoteService {
    files: Arc<dyn FileSystem>,
    desktop: Arc<dyn Desktop>,
}

impl NoteService {
    pub fn new(files: Arc<dyn FileSystem>, desktop: Arc<dyn Desktop>) -> Self {
        Self { files, desktop }
    }

    /// The note file for `name`, in the `notes/` folder next to the todo file.
    pub fn note_path(todo_file: &Path, name: &NoteName) -> PathBuf {
        let folder = todo_file.parent().unwrap_or(Path::new("")).join(NOTES_DIR);
        name.segments().fold(folder, |path, segment| path.join(segment))
    }

    /// The status of every `note:` value in a document, keyed by the raw
    /// value, so the list can mark broken links without touching the disk.
    pub fn statuses(&self, todo_file: &Path, document: &TodoDocument) -> HashMap<String, NoteStatus> {
        let mut statuses = HashMap::new();
        for entry in document.todos() {
            let Some(value) = entry.todo.note() else { continue };
            if statuses.contains_key(value) {
                continue;
            }
            let status = match NoteName::parse(value) {
                Ok(name) if self.files.exists(&Self::note_path(todo_file, &name)) => NoteStatus::Present,
                Ok(_) => NoteStatus::Missing,
                Err(_) => NoteStatus::Invalid,
            };
            statuses.insert(value.to_owned(), status);
        }
        statuses
    }

    /// Opens a todo's note in the default app, creating it with the todo
    /// text as a heading when it does not exist yet. Returns the file.
    pub fn open(&self, todo_file: &Path, todo: &Todo) -> Result<PathBuf, AppError> {
        let name = todo
            .note_name()
            .ok_or_else(|| AppError::Other("this todo has no note".to_owned()))??;
        let path = Self::note_path(todo_file, &name);
        if !self.files.exists(&path) {
            self.files.write(&path, &new_note_content(todo), false)?;
        }
        self.desktop.open_file(&path)?;
        Ok(path)
    }

    /// The note that deleting `target` would leave without any todo in the
    /// document linking to it. The file itself is kept either way.
    pub fn orphaned_by_delete(document: &TodoDocument, target: &TodoRef) -> Option<NoteName> {
        let name = Todo::parse(&target.raw).note_name()?.ok()?;
        let shared = document
            .todos()
            .filter(|entry| entry.line != target.line)
            .any(|entry| entry.todo.note_name().and_then(Result::ok).as_ref() == Some(&name));
        (!shared).then_some(name)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use super::*;

    #[derive(Default)]
    struct Fake {
        files: Mutex<HashMap<PathBuf, String>>,
        opened: Mutex<Vec<PathBuf>>,
    }

    impl FileSystem for Fake {
        fn read(&self, path: &Path) -> Result<String, AppError> {
            self.files
                .lock()
                .unwrap()
                .get(path)
                .cloned()
                .ok_or_else(|| AppError::io(path, "missing"))
        }
        fn write(&self, path: &Path, content: &str, _safe: bool) -> Result<(), AppError> {
            self.files.lock().unwrap().insert(path.to_owned(), content.to_owned());
            Ok(())
        }
        fn exists(&self, path: &Path) -> bool {
            self.files.lock().unwrap().contains_key(path)
        }
    }

    impl Desktop for Fake {
        fn open_uri(&self, _: &str) -> Result<(), AppError> {
            Ok(())
        }
        fn reveal(&self, _: &Path) -> Result<(), AppError> {
            Ok(())
        }
        fn open_file(&self, path: &Path) -> Result<(), AppError> {
            self.opened.lock().unwrap().push(path.to_owned());
            Ok(())
        }
    }

    fn service() -> (NoteService, Arc<Fake>) {
        let fake = Arc::new(Fake::default());
        (NoteService::new(fake.clone(), fake.clone()), fake)
    }

    fn todo_file() -> PathBuf {
        PathBuf::from("/lists/todo.txt")
    }

    fn note_file(name: &str) -> PathBuf {
        PathBuf::from("/lists/notes").join(name)
    }

    #[test]
    fn opening_a_missing_note_creates_it_with_a_heading() {
        let (service, fake) = service();
        let todo = Todo::parse("(A) Write report +work note:report-outline");
        let path = service.open(&todo_file(), &todo).unwrap();
        assert_eq!(path, note_file("report-outline.md"));
        assert_eq!(fake.read(&path).unwrap(), "# Write report\n");
        assert_eq!(*fake.opened.lock().unwrap(), vec![path]);
    }

    #[test]
    fn opening_an_existing_note_keeps_its_content() {
        let (service, fake) = service();
        fake.write(&note_file("a.md"), "mine", false).unwrap();
        service.open(&todo_file(), &Todo::parse("Task note:a.md")).unwrap();
        assert_eq!(fake.read(&note_file("a.md")).unwrap(), "mine");
    }

    #[test]
    fn invalid_names_are_never_opened() {
        let (service, fake) = service();
        let result = service.open(&todo_file(), &Todo::parse("Task note:../secret"));
        assert!(matches!(result, Err(AppError::Note(_))));
        assert!(fake.opened.lock().unwrap().is_empty());
        assert!(fake.files.lock().unwrap().is_empty());
    }

    #[test]
    fn statuses_mark_broken_links() {
        let (service, fake) = service();
        fake.write(&note_file("here.md"), "", false).unwrap();
        let document = TodoDocument::parse("One note:here\nTwo note:gone.md\nThree note:Bad\nFour\n", &[]);
        let statuses = service.statuses(&todo_file(), &document);
        assert_eq!(statuses["here"], NoteStatus::Present);
        assert_eq!(statuses["gone.md"], NoteStatus::Missing);
        assert_eq!(statuses["Bad"], NoteStatus::Invalid);
        assert_eq!(statuses.len(), 3);
    }

    #[test]
    fn deleting_the_last_todo_of_a_note_orphans_it() {
        let document = TodoDocument::parse("One note:shared\nTwo note:shared.md\nThree note:own.md\n", &[]);
        let target = |line: usize| {
            let entry = document.todos().find(|entry| entry.line == line).unwrap();
            TodoRef::new(line, entry.todo)
        };
        assert_eq!(NoteService::orphaned_by_delete(&document, &target(0)), None);
        assert_eq!(
            NoteService::orphaned_by_delete(&document, &target(2)).map(|n| n.to_string()),
            Some("own.md".to_owned())
        );
    }
}
