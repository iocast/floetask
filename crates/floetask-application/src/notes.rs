//! Use cases for the `note:` extension: where a todo's note file lives,
//! whether it exists, reading and writing it, and linking a file the user
//! picked, either moved into the notes folder or left where it is.
//!
//! Note files are never deleted: completing and deleting a todo leave them
//! where they are, archiving moves notes-folder notes along with the todo.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use floetask_domain::note::{new_note_content, notes_folder};
use floetask_domain::todo::Todo;
use floetask_domain::{NoteLink, NoteName, TodoDocument};

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

    /// The notes folder's name, e.g. `todo-notes` for `todo.txt`.
    pub fn folder_name(todo_file: &Path) -> String {
        let file_name = todo_file
            .file_name()
            .map(|name| name.to_string_lossy())
            .unwrap_or_default();
        notes_folder(&file_name)
    }

    /// The `<file>-notes/` folder next to the todo file.
    fn folder(todo_file: &Path) -> PathBuf {
        todo_file
            .parent()
            .unwrap_or(Path::new(""))
            .join(Self::folder_name(todo_file))
    }

    /// The note file for `name`, in the `<file>-notes/` folder next to the
    /// todo file.
    pub fn note_path(todo_file: &Path, name: &NoteName) -> PathBuf {
        name.segments()
            .fold(Self::folder(todo_file), |path, segment| path.join(segment))
    }

    /// The file a link points at: in the notes folder, or wherever an
    /// external note lives.
    pub fn link_path(todo_file: &Path, link: &NoteLink) -> PathBuf {
        match link {
            NoteLink::Folder(name) => Self::note_path(todo_file, name),
            NoteLink::External(path) => PathBuf::from(path),
        }
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
            let status = match NoteLink::parse(value) {
                Ok(link) if self.files.exists(&Self::link_path(todo_file, &link)) => NoteStatus::Present,
                Ok(_) => NoteStatus::Missing,
                Err(_) => NoteStatus::Invalid,
            };
            statuses.insert(value.to_owned(), status);
        }
        statuses
    }

    /// A todo's note and its text, creating the file with the todo text as
    /// a heading when it does not exist yet, so a broken link still opens.
    pub fn load_or_create(&self, todo_file: &Path, todo: &Todo) -> Result<(NoteLink, String), AppError> {
        let link = todo
            .note_link()
            .ok_or_else(|| AppError::Other("this todo has no note".to_owned()))??;
        let text = match self.read(todo_file, &link)? {
            Some(text) => text,
            None => {
                let text = new_note_content(todo);
                self.files.write(&Self::link_path(todo_file, &link), &text, false)?;
                text
            }
        };
        Ok((link, text))
    }

    /// Opens a note in the system's default app. Returns the file.
    pub fn open_externally(&self, todo_file: &Path, link: &NoteLink) -> Result<PathBuf, AppError> {
        let path = Self::link_path(todo_file, link);
        self.desktop.open_file(&path)?;
        Ok(path)
    }

    /// The text of a note, or `None` when its file does not exist yet.
    pub fn read(&self, todo_file: &Path, link: &NoteLink) -> Result<Option<String>, AppError> {
        let path = Self::link_path(todo_file, link);
        if self.files.exists(&path) {
            self.files.read(&path).map(Some)
        } else {
            Ok(None)
        }
    }

    /// Writes a note's text, creating the notes folder when needed.
    pub fn write(&self, todo_file: &Path, link: &NoteLink, text: &str) -> Result<(), AppError> {
        let mut content = text.to_owned();
        if !content.is_empty() && !content.ends_with('\n') {
            content.push('\n');
        }
        self.files.write(&Self::link_path(todo_file, link), &content, true)
    }

    /// The link for a file the user picked when it already lies in the
    /// notes folder under a valid name: nothing to move or ask then.
    pub fn folder_link(todo_file: &Path, picked: &Path) -> Option<NoteLink> {
        let relative = picked.strip_prefix(Self::folder(todo_file)).ok()?;
        let segments: Option<Vec<&str>> = relative.iter().map(|segment| segment.to_str()).collect();
        NoteName::of_file(&segments?.join("/")).map(NoteLink::Folder)
    }

    /// Links a picked file where it is, by its absolute path.
    pub fn external_link(picked: &Path) -> Result<NoteLink, AppError> {
        NoteLink::external(&picked.to_string_lossy())
            .ok_or_else(|| AppError::Other(format!("{} is not an absolute path", picked.display())))
    }

    /// Moves a picked file into the notes folder under a valid name that no
    /// other note uses (`My Plan.md` becomes `my-plan.md`), and links it.
    /// The original is removed only once the copy is written.
    pub fn move_into_folder(&self, todo_file: &Path, picked: &Path) -> Result<NoteLink, AppError> {
        let file_name = picked
            .file_name()
            .map(|name| name.to_string_lossy())
            .unwrap_or_default();
        let name = self.unused_name(todo_file, NoteName::from_file_name(&file_name));
        let content = self.files.read(picked)?;
        self.files.write(&Self::note_path(todo_file, &name), &content, true)?;
        self.files.remove(picked)?;
        Ok(NoteLink::Folder(name))
    }

    /// `name`, or `name-2`, `name-3`, ... if that file already exists, so a
    /// new note never overwrites one another todo links to.
    pub fn unused_name(&self, todo_file: &Path, name: NoteName) -> NoteName {
        let taken = |candidate: &NoteName| self.files.exists(&Self::note_path(todo_file, candidate));
        if !taken(&name) {
            return name;
        }
        (2..)
            .map(|n| name.with_suffix(n))
            .find(|candidate| !taken(candidate))
            .expect("some suffix is free")
    }

    /// The note that deleting `target` would leave without any todo in the
    /// document linking to it. The file itself is kept either way.
    pub fn orphaned_by_delete(document: &TodoDocument, target: &TodoRef) -> Option<NoteLink> {
        let link = Todo::parse(&target.raw).note_link()?.ok()?;
        let shared = document
            .todos()
            .filter(|entry| entry.line != target.line)
            .any(|entry| entry.todo.note_link().and_then(Result::ok).as_ref() == Some(&link));
        (!shared).then_some(link)
    }
}

/// Carries the notes of todos that move to another todo file (archiving to
/// the done file) into that file's notes folder, so their links keep working.
#[derive(Debug, Default)]
pub(crate) struct NoteTransfer {
    copies: Vec<(PathBuf, PathBuf)>,
    /// Originals no todo left behind links to any more.
    originals: Vec<PathBuf>,
}

impl NoteTransfer {
    /// Decides where each moving todo's note goes and retags a todo whose
    /// note has to take a free name because the target folder already holds
    /// a different file under its name. A note still linked from a todo in
    /// `staying` is copied, not moved. Missing or invalid notes are left as
    /// they are.
    pub(crate) fn plan(
        files: &dyn FileSystem,
        from: &Path,
        to: &Path,
        moving: Vec<Todo>,
        staying: &TodoDocument,
    ) -> (Vec<Todo>, NoteTransfer) {
        let mut transfer = NoteTransfer::default();
        let mut renamed: HashMap<NoteName, NoteName> = HashMap::new();
        let todos = moving
            .into_iter()
            .map(|todo| {
                let Some(Ok(name)) = todo.note_name() else {
                    return todo;
                };
                let target = match renamed.get(&name) {
                    Some(target) => target.clone(),
                    None => {
                        let source = NoteService::note_path(from, &name);
                        let Ok(content) = files.read(&source) else {
                            return todo;
                        };
                        let target = transfer.free_target(files, to, &name, &content);
                        if !files.exists(&NoteService::note_path(to, &target)) {
                            transfer
                                .copies
                                .push((source.clone(), NoteService::note_path(to, &target)));
                        }
                        if !links_to(staying, &name) {
                            transfer.originals.push(source);
                        }
                        renamed.insert(name.clone(), target.clone());
                        target
                    }
                };
                if target == name {
                    todo
                } else {
                    todo.with_note(Some(&target))
                }
            })
            .collect();
        (todos, transfer)
    }

    /// `name`, or the first `name-n` that is free in the target folder. A
    /// file there with the same content counts as free: it is the same note.
    fn free_target(&self, files: &dyn FileSystem, to: &Path, name: &NoteName, content: &str) -> NoteName {
        let usable = |candidate: &NoteName| {
            let path = NoteService::note_path(to, candidate);
            let planned = self.copies.iter().any(|(_, target)| *target == path);
            !planned && (!files.exists(&path) || files.read(&path).is_ok_and(|existing| existing == content))
        };
        if usable(name) {
            return name.clone();
        }
        (2..)
            .map(|n| name.with_suffix(n))
            .find(|candidate| usable(candidate))
            .expect("some suffix is free")
    }

    pub(crate) fn copy(&self, files: &dyn FileSystem) -> Result<(), AppError> {
        for (source, target) in &self.copies {
            files.write(target, &files.read(source)?, true)?;
        }
        Ok(())
    }

    /// Best effort: both todo files are already saved, so a leftover note
    /// only costs disk space.
    pub(crate) fn remove_originals(&self, files: &dyn FileSystem) {
        for original in &self.originals {
            let _ = files.remove(original);
        }
    }
}

fn links_to(document: &TodoDocument, name: &NoteName) -> bool {
    document
        .todos()
        .any(|entry| entry.todo.note_name().and_then(Result::ok).as_ref() == Some(name))
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
        fn remove(&self, path: &Path) -> Result<(), AppError> {
            self.files
                .lock()
                .unwrap()
                .remove(path)
                .map(drop)
                .ok_or_else(|| AppError::io(path, "missing"))
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
        PathBuf::from("/lists/todo-notes").join(name)
    }

    #[test]
    fn loading_a_missing_note_creates_it_with_a_heading() {
        let (service, fake) = service();
        let todo = Todo::parse("(A) Write report +work note:report-outline");
        let (link, text) = service.load_or_create(&todo_file(), &todo).unwrap();
        assert_eq!(link.to_string(), "report-outline.md");
        assert_eq!(text, "# Write report\n");
        assert_eq!(fake.read(&note_file("report-outline.md")).unwrap(), "# Write report\n");
    }

    #[test]
    fn loading_an_existing_note_keeps_its_content() {
        let (service, fake) = service();
        fake.write(&note_file("a.md"), "mine", false).unwrap();
        let (_, text) = service
            .load_or_create(&todo_file(), &Todo::parse("Task note:a.md"))
            .unwrap();
        assert_eq!(text, "mine");
        assert_eq!(fake.read(&note_file("a.md")).unwrap(), "mine");
    }

    #[test]
    fn invalid_names_are_never_opened() {
        let (service, fake) = service();
        let result = service.load_or_create(&todo_file(), &Todo::parse("Task note:../secret"));
        assert!(matches!(result, Err(AppError::Note(_))));
        assert!(fake.files.lock().unwrap().is_empty());
    }

    #[test]
    fn notes_open_in_the_default_app() {
        let (service, fake) = service();
        let link = NoteLink::parse("a.md").unwrap();
        assert_eq!(service.open_externally(&todo_file(), &link), Ok(note_file("a.md")));
        assert_eq!(*fake.opened.lock().unwrap(), vec![note_file("a.md")]);
    }

    #[test]
    fn new_notes_get_a_free_name_and_a_final_newline() {
        let (service, fake) = service();
        let name = NoteName::parse("plan.md").unwrap();
        let link = NoteLink::Folder(name.clone());
        assert_eq!(service.read(&todo_file(), &link), Ok(None));
        service.write(&todo_file(), &link, "# Plan").unwrap();
        assert_eq!(service.read(&todo_file(), &link), Ok(Some("# Plan\n".to_owned())));
        assert_eq!(service.unused_name(&todo_file(), name.clone()).to_string(), "plan-2.md");
        fake.write(&note_file("plan-2.md"), "", false).unwrap();
        assert_eq!(service.unused_name(&todo_file(), name).to_string(), "plan-3.md");
    }

    #[test]
    fn external_notes_are_read_and_written_where_they_are() {
        let (service, fake) = service();
        let picked = PathBuf::from("/home/me/My Notes/Plan.md");
        fake.write(&picked, "# Plan\n", false).unwrap();
        let link = NoteService::external_link(&picked).unwrap();
        assert_eq!(NoteService::link_path(&todo_file(), &link), picked);
        assert_eq!(service.read(&todo_file(), &link), Ok(Some("# Plan\n".to_owned())));
        service.write(&todo_file(), &link, "# Plan\n\nMore").unwrap();
        assert_eq!(fake.read(&picked).unwrap(), "# Plan\n\nMore\n");
    }

    #[test]
    fn picked_files_move_into_the_notes_folder_under_a_free_name() {
        let (service, fake) = service();
        fake.write(&note_file("my-plan.md"), "taken", false).unwrap();
        let picked = PathBuf::from("/home/me/My Plan.MD");
        fake.write(&picked, "# Plan\n", false).unwrap();
        let link = service.move_into_folder(&todo_file(), &picked).unwrap();
        assert_eq!(link.to_string(), "my-plan-2.md");
        assert_eq!(fake.read(&note_file("my-plan-2.md")).unwrap(), "# Plan\n");
        assert!(!fake.exists(&picked));
        assert_eq!(fake.read(&note_file("my-plan.md")).unwrap(), "taken");
    }

    #[test]
    fn files_already_in_the_notes_folder_link_by_name() {
        let link = |path: &str| NoteService::folder_link(&todo_file(), Path::new(path)).map(|l| l.to_string());
        assert_eq!(link("/lists/todo-notes/work/q4.md"), Some("work/q4.md".to_owned()));
        assert_eq!(link("/lists/todo-notes/My Plan.md"), None);
        assert_eq!(link("/lists/done-notes/q4.md"), None);
    }

    #[test]
    fn statuses_mark_broken_links() {
        let (service, fake) = service();
        fake.write(&note_file("here.md"), "", false).unwrap();
        fake.write(Path::new("/home/me/a b.md"), "", false).unwrap();
        let document = TodoDocument::parse(
            "One note:here\nTwo note:gone.md\nThree note:Bad\nFour\nFive note:/home/me/a%20b.md\n",
            &[],
        );
        let statuses = service.statuses(&todo_file(), &document);
        assert_eq!(statuses["here"], NoteStatus::Present);
        assert_eq!(statuses["gone.md"], NoteStatus::Missing);
        assert_eq!(statuses["Bad"], NoteStatus::Invalid);
        assert_eq!(statuses["/home/me/a%20b.md"], NoteStatus::Present);
        assert_eq!(statuses.len(), 4);
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
