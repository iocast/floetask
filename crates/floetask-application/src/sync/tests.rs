use std::collections::{BTreeMap, HashMap};
use std::sync::Mutex;

use super::*;

#[derive(Default)]
struct MemoryFiles(Mutex<HashMap<PathBuf, String>>);

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
        self.0.lock().unwrap().remove(path);
        Ok(())
    }
}

/// A store keeping files in memory; a file's version is its content.
#[derive(Default)]
struct MemoryStore {
    files: Mutex<BTreeMap<String, String>>,
    offline: Mutex<bool>,
}

impl MemoryStore {
    fn get_text(&self, name: &str) -> Option<String> {
        self.files.lock().unwrap().get(name).cloned()
    }
    fn set(&self, name: &str, content: &str) {
        self.files.lock().unwrap().insert(name.to_owned(), content.to_owned());
    }
    fn check(&self) -> Result<(), AppError> {
        if *self.offline.lock().unwrap() {
            Err(AppError::io("/store", "unreachable"))
        } else {
            Ok(())
        }
    }
}

impl RemoteStore for MemoryStore {
    fn get(&self, name: &str) -> Result<Option<(String, Version)>, AppError> {
        self.check()?;
        Ok(self.get_text(name).map(|text| (text.clone(), Version(text))))
    }
    fn put(&self, name: &str, content: &str, expected: Option<&Version>) -> Result<Version, AppError> {
        self.check()?;
        let current = self.get_text(name).map(Version);
        if current.as_ref() != expected {
            return Err(AppError::Changed(PathBuf::from(name)));
        }
        self.set(name, content);
        Ok(Version(content.to_owned()))
    }
    fn conflict_copies(&self, name: &str) -> Result<Vec<String>, AppError> {
        let stem = name.trim_end_matches(".txt");
        Ok(self
            .files
            .lock()
            .unwrap()
            .keys()
            .filter(|key| key.starts_with(&format!("{stem}-conflict")))
            .cloned()
            .collect())
    }
    fn remove(&self, name: &str) -> Result<(), AppError> {
        self.files.lock().unwrap().remove(name);
        Ok(())
    }
    fn watch_path(&self) -> Option<PathBuf> {
        Some(PathBuf::from("/shared"))
    }
}

struct Stores(Arc<MemoryStore>);

impl RemoteStores for Stores {
    fn open(&self, _: &StoreLocation) -> Arc<dyn RemoteStore> {
        self.0.clone()
    }
}

#[derive(Default)]
struct MemoryLinks {
    links: Mutex<Vec<SyncLink>>,
    bases: Mutex<HashMap<(PathBuf, String), String>>,
}

impl SyncLinkStore for MemoryLinks {
    fn load(&self) -> Result<Vec<SyncLink>, AppError> {
        Ok(self.links.lock().unwrap().clone())
    }
    fn save(&self, links: &[SyncLink]) -> Result<(), AppError> {
        *self.links.lock().unwrap() = links.to_vec();
        Ok(())
    }
    fn base(&self, local: &Path, item: &str) -> Result<Option<String>, AppError> {
        Ok(self
            .bases
            .lock()
            .unwrap()
            .get(&(local.to_owned(), item.to_owned()))
            .cloned())
    }
    fn set_base(&self, local: &Path, item: &str, content: &str) -> Result<(), AppError> {
        self.bases
            .lock()
            .unwrap()
            .insert((local.to_owned(), item.to_owned()), content.to_owned());
        Ok(())
    }
    fn remove_bases(&self, local: &Path) -> Result<(), AppError> {
        self.bases.lock().unwrap().retain(|(path, _), _| path != local);
        Ok(())
    }
}

const LOCAL: &str = "/projects/acme/todo.txt";
const SHARED: &str = "acme.txt";

struct Setup {
    service: SyncService,
    files: Arc<MemoryFiles>,
    store: Arc<MemoryStore>,
}

impl Setup {
    fn new(local: &str) -> Self {
        let files = Arc::new(MemoryFiles::default());
        files.write(Path::new(LOCAL), local, false).unwrap();
        let store = Arc::new(MemoryStore::default());
        let service = SyncService::new(
            files.clone(),
            Arc::new(Stores(store.clone())),
            Arc::new(MemoryLinks::default()),
        );
        Self { service, files, store }
    }

    /// A file shared with `content` on both sides.
    fn shared(content: &str) -> Self {
        let setup = Self::new(content);
        assert_eq!(setup.share().unwrap(), ShareOutcome::Shared);
        setup
    }

    fn share(&self) -> Result<ShareOutcome, AppError> {
        self.service.share(Path::new(LOCAL), location(), SHARED)
    }

    fn sync(&self) -> SyncReport {
        self.service.sync(Path::new(LOCAL), &[]).unwrap()
    }

    fn local(&self) -> String {
        self.files.read(Path::new(LOCAL)).unwrap()
    }

    fn set_local(&self, content: &str) {
        self.files.write(Path::new(LOCAL), content, false).unwrap();
    }

    fn remote(&self) -> String {
        self.store.get_text(SHARED).unwrap_or_default()
    }
}

fn location() -> StoreLocation {
    StoreLocation::Folder {
        path: PathBuf::from("/shared"),
    }
}

const BASE: &str = "(A) Call dentist\nBuy milk\n";

#[test]
fn sharing_copies_the_file_once() {
    let setup = Setup::shared(BASE);
    assert_eq!(setup.remote(), BASE);
    assert_eq!(setup.service.links().unwrap().len(), 1);
    let other = Setup::new("other\n");
    other.store.set(SHARED, BASE);
    assert_eq!(other.share().unwrap(), ShareOutcome::Exists);
    assert_eq!(other.remote(), BASE);
}

#[test]
fn changes_on_both_sides_merge_into_both() {
    let setup = Setup::shared(BASE);
    setup.set_local("(A) Call dentist\nx 2026-11-07 Buy milk\n");
    setup
        .store
        .set(SHARED, "(A) Call dentist\nBuy milk\nPay rent due:2026-11-10\n");
    let report = setup.sync();
    let expected = "(A) Call dentist\nx 2026-11-07 Buy milk\nPay rent due:2026-11-10\n";
    assert_eq!(report.status, SyncStatus::InSync);
    assert!(report.local_changed);
    assert_eq!(setup.local(), expected);
    assert_eq!(setup.remote(), expected);
    // Nothing changed since: the next sync does nothing.
    assert!(!setup.sync().local_changed);
}

#[test]
fn conflicts_keep_the_local_version_and_push_nothing_until_settled() {
    let setup = Setup::shared(BASE);
    setup.set_local("(B) Call dentist\nBuy milk\n");
    let theirs = "(A) Call dentist +health\nx Buy milk\n";
    setup.store.set(SHARED, theirs);
    assert_eq!(setup.sync().status, SyncStatus::Conflict(1));
    assert_eq!(setup.local(), "(B) Call dentist\nx Buy milk\n");
    assert_eq!(setup.remote(), theirs);
    // The conflict is found again by the next sync.
    assert_eq!(setup.sync().status, SyncStatus::Conflict(1));

    let found = setup.service.conflicts(Path::new(LOCAL)).unwrap();
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].conflicts[0].local, ["(B) Call dentist"]);
    let report = setup
        .service
        .resolve(Path::new(LOCAL), &[(String::new(), vec![Choice::Mine])], &[])
        .unwrap();
    assert_eq!(report.status, SyncStatus::InSync);
    assert_eq!(setup.local(), "(B) Call dentist\nx Buy milk\n");
    assert_eq!(setup.remote(), setup.local());
}

#[test]
fn resolving_fails_when_the_conflicts_changed() {
    let setup = Setup::shared(BASE);
    setup.set_local("(B) Call dentist\nBuy milk\n");
    setup.store.set(SHARED, "(C) Call dentist\nBuy milk\n");
    setup.sync();
    let error = setup
        .service
        .resolve(
            Path::new(LOCAL),
            &[(String::new(), vec![Choice::Mine, Choice::Mine])],
            &[],
        )
        .unwrap_err();
    assert!(matches!(error, AppError::Changed(_)));
}

#[test]
fn joining_copies_the_shared_file_or_merges_without_a_base() {
    let setup = Setup::new("");
    setup.files.remove(Path::new(LOCAL)).unwrap();
    setup.store.set(SHARED, BASE);
    let report = setup.service.join(Path::new(LOCAL), location(), SHARED).unwrap();
    assert_eq!(report.status, SyncStatus::InSync);
    assert_eq!(setup.local(), BASE);

    let existing = Setup::new("(A) Call dentist\nWalk dog\n");
    existing.store.set(SHARED, BASE);
    let report = existing.service.join(Path::new(LOCAL), location(), SHARED).unwrap();
    assert_eq!(report.status, SyncStatus::Conflict(1));
    let report = existing
        .service
        .resolve(Path::new(LOCAL), &[(String::new(), vec![Choice::Both])], &[])
        .unwrap();
    assert_eq!(report.status, SyncStatus::InSync);
    assert_eq!(existing.local(), "(A) Call dentist\nWalk dog\nBuy milk\n");
    assert_eq!(existing.remote(), existing.local());
}

#[test]
fn joining_an_empty_shared_file_fills_it() {
    let setup = Setup::new(BASE);
    setup.store.set(SHARED, "");
    assert!(setup.service.exists(&location(), SHARED).unwrap());
    let report = setup.service.join(Path::new(LOCAL), location(), SHARED).unwrap();
    assert_eq!(report.status, SyncStatus::InSync);
    assert_eq!(setup.remote(), BASE);
}

#[test]
fn notes_sync_with_the_shared_notes_folder() {
    let setup = Setup::shared("Plan launch note:plan\n");
    let note = PathBuf::from("/projects/acme/todo-notes/plan.md");
    setup.files.write(&note, "# Plan\n", false).unwrap();
    setup.sync();
    assert_eq!(setup.store.get_text("acme-notes/plan.md").unwrap(), "# Plan\n");

    setup.store.set("acme-notes/plan.md", "# Plan\n\n- budget\n");
    setup.files.write(&note, "# Plan\n- dates\n", false).unwrap();
    let report = setup.sync();
    assert_eq!(report.status, SyncStatus::InSync);
    assert_eq!(setup.files.read(&note).unwrap(), "# Plan\n- dates\n\n- budget\n");

    // A note open in an editor waits.
    setup.store.set("acme-notes/plan.md", "# Plan v2\n");
    setup
        .service
        .sync(Path::new(LOCAL), std::slice::from_ref(&note))
        .unwrap();
    assert_eq!(setup.files.read(&note).unwrap(), "# Plan\n- dates\n\n- budget\n");
}

#[test]
fn new_notes_on_the_other_side_are_copied_here() {
    let setup = Setup::shared(BASE);
    setup.store.set(SHARED, "Plan launch note:plan\n");
    setup.store.set("acme-notes/plan.md", "# Plan\n");
    setup.set_local("Plan launch note:plan\n");
    setup.sync();
    let note = Path::new("/projects/acme/todo-notes/plan.md");
    assert_eq!(setup.files.read(note).unwrap(), "# Plan\n");
}

#[test]
fn conflict_copies_are_merged_and_removed() {
    let setup = Setup::shared(BASE);
    setup
        .store
        .set("acme-conflict-PC.txt", "(A) Call dentist\nBuy milk\nFrom the copy\n");
    setup.set_local("(A) Call dentist\nx Buy milk\n");
    let report = setup.sync();
    assert_eq!(report.status, SyncStatus::InSync);
    assert_eq!(setup.local(), "(A) Call dentist\nx Buy milk\nFrom the copy\n");
    assert_eq!(setup.remote(), setup.local());
    assert!(setup.store.get_text("acme-conflict-PC.txt").is_none());

    setup
        .store
        .set("acme-conflict-PC.txt", "(C) Call dentist\nx Buy milk\nFrom the copy\n");
    setup.store.set(SHARED, "(B) Call dentist\nx Buy milk\nFrom the copy\n");
    let report = setup.sync();
    assert_eq!(report.copies_left, ["acme-conflict-PC.txt"]);
}

#[test]
fn a_missing_shared_copy_or_store_is_reported() {
    let setup = Setup::shared(BASE);
    *setup.store.offline.lock().unwrap() = true;
    assert!(matches!(setup.sync().status, SyncStatus::Offline(_)));
    *setup.store.offline.lock().unwrap() = false;
    setup.store.remove(SHARED).unwrap();
    assert_eq!(setup.sync().status, SyncStatus::SharedMissing);
    assert_eq!(setup.local(), BASE);
}

#[test]
fn unlinking_keeps_both_files() {
    let setup = Setup::shared(BASE);
    setup.service.unlink(Path::new(LOCAL)).unwrap();
    assert!(setup.service.links().unwrap().is_empty());
    assert_eq!(setup.local(), BASE);
    assert_eq!(setup.remote(), BASE);
}
