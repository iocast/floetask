use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;

use floetask_domain::date::parse_iso;

use super::*;

#[derive(Default)]
struct MemoryFiles(Mutex<HashMap<PathBuf, String>>);

impl MemoryFiles {
    fn with(files: &[(&str, &str)]) -> Arc<Self> {
        let map = files.iter().map(|(p, c)| (PathBuf::from(p), (*c).to_owned())).collect();
        Arc::new(Self(Mutex::new(map)))
    }

    fn get(&self, path: &str) -> String {
        self.0.lock().unwrap().get(Path::new(path)).cloned().unwrap_or_default()
    }
}

impl FileSystem for MemoryFiles {
    fn read(&self, path: &Path) -> Result<String, AppError> {
        self.0
            .lock()
            .unwrap()
            .get(path)
            .cloned()
            .ok_or_else(|| AppError::io(path, "missing"))
    }
    fn write(&self, path: &Path, content: &str, _safe: bool) -> Result<(), AppError> {
        self.0.lock().unwrap().insert(path.to_owned(), content.to_owned());
        Ok(())
    }
    fn exists(&self, path: &Path) -> bool {
        self.0.lock().unwrap().contains_key(path)
    }
    fn remove(&self, path: &Path) -> Result<(), AppError> {
        self.0
            .lock()
            .unwrap()
            .remove(path)
            .map(drop)
            .ok_or_else(|| AppError::io(path, "missing"))
    }
}

struct FixedClock;
impl Clock for FixedClock {
    fn today(&self) -> Date {
        parse_iso("2024-03-13").unwrap()
    }
}

fn service(files: &Arc<MemoryFiles>) -> TodoFileService {
    TodoFileService::new(files.clone(), Arc::new(FixedClock))
}

fn target(line: usize, raw: &str) -> TodoRef {
    TodoRef {
        line,
        raw: raw.to_owned(),
    }
}

const TODO: &str = "todo.txt";
const DONE: &str = "done.txt";

#[test]
fn add_converts_dates_and_appends_creation_date() {
    let files = MemoryFiles::with(&[(TODO, "first\n")]);
    let input = InputOptions {
        append_creation_date: true,
        convert_relative_dates: true,
        ..Default::default()
    };
    service(&files)
        .add(
            Path::new(TODO),
            "Call bob due:tomorrow",
            &input,
            &FileOptions::default(),
        )
        .unwrap();
    assert_eq!(files.get(TODO), "first\n2024-03-13 Call bob due:2024-03-14\n");
}

#[test]
fn bulk_add_creates_one_todo_per_line() {
    let files = MemoryFiles::with(&[(TODO, "")]);
    let input = InputOptions {
        bulk: true,
        ..Default::default()
    };
    let svc = service(&files);
    assert_eq!(svc.prepare_new_todos("a\n\nb\nc", &input).len(), 3);
    svc.add(Path::new(TODO), "a\nb", &input, &FileOptions::default())
        .unwrap();
    assert_eq!(files.get(TODO), "a\nb\n");
}

#[test]
fn empty_input_is_rejected() {
    let files = MemoryFiles::with(&[(TODO, "")]);
    let result = service(&files).add(
        Path::new(TODO),
        "   ",
        &InputOptions::default(),
        &FileOptions::default(),
    );
    assert_eq!(result.unwrap_err(), AppError::EmptyTodo);
}

#[test]
fn completing_recurring_todo_inserts_next_occurrence() {
    let files = MemoryFiles::with(&[(TODO, "a\nWater plants due:2024-03-13 rec:1w\nb\n")]);
    service(&files)
        .toggle_complete(
            Path::new(TODO),
            &target(1, "Water plants due:2024-03-13 rec:1w"),
            &FileOptions::default(),
        )
        .unwrap();
    assert_eq!(
        files.get(TODO),
        "a\nx 2024-03-13 2024-03-13 Water plants due:2024-03-13 rec:1w\n2024-03-13 Water plants due:2024-03-20 rec:1w\nb\n"
    );
}

#[test]
fn stale_edits_are_refused() {
    let files = MemoryFiles::with(&[(TODO, "changed outside\n")]);
    let result = service(&files).delete(
        Path::new(TODO),
        &target(0, "what the user saw"),
        &FileOptions::default(),
    );
    assert!(matches!(result, Err(AppError::Document(_))));
    assert_eq!(files.get(TODO), "changed outside\n");
}

#[test]
fn archive_moves_completed_to_done_file() {
    let files = MemoryFiles::with(&[(TODO, "x 2024-01-01 old\nopen\n"), (DONE, "x 2023-01-01 older\n")]);
    let (document, count) = service(&files)
        .archive_completed(Path::new(TODO), Path::new(DONE), &FileOptions::default())
        .unwrap();
    assert_eq!(count, 1);
    assert_eq!(document.todo_count(), 1);
    assert_eq!(files.get(TODO), "open\n");
    assert_eq!(files.get(DONE), "x 2023-01-01 older\nx 2024-01-01 old\n");
}

#[test]
fn archive_one_creates_done_file() {
    let files = MemoryFiles::with(&[(TODO, "keep\nmove me\n")]);
    service(&files)
        .archive_one(
            Path::new(TODO),
            Path::new(DONE),
            &target(1, "move me"),
            &FileOptions::default(),
        )
        .unwrap();
    assert_eq!(files.get(TODO), "keep\n");
    assert_eq!(files.get(DONE), "move me\n");
}

#[test]
fn rename_and_remove_across_file() {
    let files = MemoryFiles::with(&[(TODO, "a +work\nb +work +home\nc +Work\n")]);
    let svc = service(&files);
    let (_, changed) = svc
        .rename_attribute(
            Path::new(TODO),
            Attribute::Projects,
            "work",
            "job",
            &FileOptions::default(),
        )
        .unwrap();
    assert_eq!(changed, 2);
    let (_, removed) = svc
        .remove_attribute(Path::new(TODO), Attribute::Projects, "home", &FileOptions::default())
        .unwrap();
    assert_eq!(removed, 1);
    assert_eq!(files.get(TODO), "a +job\nb +job\nc +Work\n");
}

#[test]
fn excluded_lines_survive_edits() {
    let files = MemoryFiles::with(&[(TODO, "## notes\ntask\n")]);
    let options = FileOptions {
        exclude_prefixes: vec!["##".into()],
        safe_writes: true,
    };
    let svc = service(&files);
    let document = svc.load(Path::new(TODO), &options).unwrap();
    assert_eq!(document.todo_count(), 1);
    svc.update(
        Path::new(TODO),
        &target(1, "task"),
        "task +p",
        &InputOptions::default(),
        &options,
    )
    .unwrap();
    assert_eq!(files.get(TODO), "## notes\ntask +p\n");
}

#[test]
fn archiving_moves_notes_into_the_done_notes_folder() {
    let note = |folder: &str, name: &str| Path::new(folder).join(name).to_string_lossy().into_owned();
    let todo =
        "x 2024-01-01 a note:a.md\nx 2024-01-01 shared note:s.md\nopen note:s.md\nx 2024-01-01 clash note:c.md\n";
    let files = MemoryFiles::with(&[
        (TODO, todo),
        (&note("todo-notes", "a.md"), "A\n"),
        (&note("todo-notes", "s.md"), "S\n"),
        (&note("todo-notes", "c.md"), "new C\n"),
        (&note("done-notes", "c.md"), "old C\n"),
    ]);
    service(&files)
        .archive_completed(Path::new(TODO), Path::new(DONE), &FileOptions::default())
        .unwrap();

    // Moved: no todo in todo.txt links to it any more.
    assert_eq!(files.get(&note("done-notes", "a.md")), "A\n");
    assert!(!files.exists(Path::new(&note("todo-notes", "a.md"))));
    // Copied: the open todo still links to it.
    assert_eq!(files.get(&note("done-notes", "s.md")), "S\n");
    assert_eq!(files.get(&note("todo-notes", "s.md")), "S\n");
    // A different note already has the name in done-notes: take a free one.
    assert_eq!(files.get(&note("done-notes", "c.md")), "old C\n");
    assert_eq!(files.get(&note("done-notes", "c-2.md")), "new C\n");
    assert!(!files.exists(Path::new(&note("todo-notes", "c.md"))));
    assert_eq!(
        files.get(DONE),
        "x 2024-01-01 a note:a.md\nx 2024-01-01 shared note:s.md\nx 2024-01-01 clash note:c-2.md\n"
    );
    assert_eq!(files.get(TODO), "open note:s.md\n");
}
