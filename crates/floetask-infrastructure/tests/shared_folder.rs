//! Two people share a todo file through a synced folder, each with their
//! own local file somewhere else, on the real file system. The folder sync
//! tool is played by copying the shared copy between their two folders.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use floetask_application::ports::FileSystem;
use floetask_application::sync::{ShareOutcome, StoreLocation, SyncService, SyncStatus};
use floetask_infrastructure::fs::LocalFileSystem;
use floetask_infrastructure::remote::LocalStores;
use floetask_infrastructure::stores::sync_links::TomlSyncLinkStore;

struct Person {
    sync: SyncService,
    local: PathBuf,
    folder: PathBuf,
}

impl Person {
    fn new(root: &Path, who: &str, local: &str) -> Self {
        let home = root.join(who);
        let folder = home.join("OneDrive - Company").join("acme");
        fs::create_dir_all(&folder).unwrap();
        let state = home.join("state");
        let sync = SyncService::new(
            Arc::new(LocalFileSystem),
            Arc::new(LocalStores),
            Arc::new(TomlSyncLinkStore::new(state.join("sync.toml"), state.join("sync"))),
        );
        Self {
            sync,
            local: home.join(local),
            folder,
        }
    }

    fn store(&self) -> StoreLocation {
        StoreLocation::Folder {
            path: self.folder.clone(),
        }
    }

    fn read(&self) -> String {
        fs::read_to_string(&self.local).unwrap()
    }

    fn write(&self, content: &str) {
        LocalFileSystem.write(&self.local, content, true).unwrap();
    }

    fn sync(&self) -> SyncStatus {
        self.sync.sync(&self.local, &[]).unwrap().status
    }
}

/// What the OneDrive app does: copies the shared folder over.
fn onedrive(from: &Person, to: &Person) {
    for entry in fs::read_dir(&from.folder).unwrap() {
        let entry = entry.unwrap();
        if entry.file_type().unwrap().is_file() {
            fs::copy(entry.path(), to.folder.join(entry.file_name())).unwrap();
        }
    }
}

#[test]
fn two_people_keep_their_own_files_in_step() {
    let root = tempfile::tempdir().unwrap();
    let me = Person::new(root.path(), "me", "projects/acme/todo.txt");
    let colleague = Person::new(root.path(), "colleague", "work/acme/tasks.txt");

    me.write("(A) Call dentist\nBuy milk\n");
    let outcome = me.sync.share(&me.local, me.store(), "acme.txt").unwrap();
    assert_eq!(outcome, ShareOutcome::Shared);
    onedrive(&me, &colleague);

    colleague
        .sync
        .join(&colleague.local, colleague.store(), "acme.txt")
        .unwrap();
    assert_eq!(colleague.read(), "(A) Call dentist\nBuy milk\n");

    // Both change something at once. My change reaches the colleague's
    // folder before their floetask synced their own change.
    me.write("(A) Call dentist\nx 2026-11-07 Buy milk\n");
    colleague.write("(A) Call dentist\nBuy milk\nPay rent due:2026-11-10\n");
    assert_eq!(me.sync(), SyncStatus::InSync);
    onedrive(&me, &colleague);
    assert_eq!(colleague.sync(), SyncStatus::InSync);
    onedrive(&colleague, &me);
    assert_eq!(me.sync(), SyncStatus::InSync);

    let expected = "(A) Call dentist\nx 2026-11-07 Buy milk\nPay rent due:2026-11-10\n";
    assert_eq!(me.read(), expected);
    assert_eq!(colleague.read(), expected);
    // Nothing but the shared copy in the shared folder: no leftovers.
    let names: Vec<_> = fs::read_dir(&me.folder)
        .unwrap()
        .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    assert_eq!(names, ["acme.txt"]);
}

#[test]
fn a_onedrive_conflict_copy_is_merged_in() {
    let root = tempfile::tempdir().unwrap();
    let me = Person::new(root.path(), "me", "todo.txt");
    me.write("one\ntwo\n");
    me.sync.share(&me.local, me.store(), "acme.txt").unwrap();
    // OneDrive kept the colleague's upload next to mine.
    fs::write(me.folder.join("acme-DESKTOP-K7.txt"), "one\ntwo\nthree\n").unwrap();
    assert_eq!(me.sync(), SyncStatus::InSync);
    assert_eq!(me.read(), "one\ntwo\nthree\n");
    assert!(!me.folder.join("acme-DESKTOP-K7.txt").exists());
}
