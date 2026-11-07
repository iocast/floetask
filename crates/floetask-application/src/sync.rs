//! Use cases for keeping a todo file and its notes in step with a shared
//! copy, see `features/todo-notes/19-sync.md`.
//!
//! The local file stays the file floetask works on. A sync reads the local
//! file, the shared copy and the base copy (both sides' content at their
//! last sync), merges them line by line, writes the merge to the local file
//! and pushes it to the shared copy. Both writes only replace the content
//! the merge was based on; when either changed meanwhile, the sync reads
//! again and merges again.
//!
//! Lines changed differently on both sides are conflicts: the local file
//! keeps its own version and nothing is pushed until the user settled them,
//! so the other side's version stays in the shared copy. Conflicts are not
//! stored: the next merge of the same three versions finds them again.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};

use floetask_domain::merge::{Choice, Conflict, Merge, merge2, merge3};
use floetask_domain::note::notes_folder;
use floetask_domain::{NoteLink, NoteName, TodoDocument};

use crate::error::AppError;
use crate::notes::NoteService;
use crate::ports::{FileSystem, RemoteStore, RemoteStores, SyncLinkStore, Version};

/// How often a sync reads and merges again when a side changed while it
/// was writing, before it gives up until the next sync.
const ATTEMPTS: usize = 3;

/// Where shared copies live, as the user picked it when sharing.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum StoreLocation {
    /// A folder another tool syncs (the OneDrive app, Dropbox, Syncthing)
    /// or several computers reach (a network share).
    Folder { path: PathBuf },
}

/// A local todo file linked to a shared copy.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SyncLink {
    pub local: PathBuf,
    pub store: StoreLocation,
    /// The shared copy's name in the store, such as `acme.txt`.
    pub name: String,
}

impl SyncLink {
    /// Where the shared copy is, for the user.
    pub fn shared_location(&self) -> String {
        match &self.store {
            StoreLocation::Folder { path } => path.join(&self.name).display().to_string(),
        }
    }
}

/// How a linked file stands after a sync.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SyncStatus {
    InSync,
    /// This many conflicts wait to be settled, in the file and its notes.
    Conflict(usize),
    /// The store could not be reached; edits stay local until it can.
    Offline(AppError),
    /// The shared copy was deleted or renamed. The local file stays.
    SharedMissing,
}

/// What a sync did.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyncReport {
    pub status: SyncStatus,
    /// The sync wrote the local file or one of its notes.
    pub local_changed: bool,
    /// Conflict copies a sync tool made that could not be merged without
    /// conflicts; they stay next to the shared copy for the user.
    pub copies_left: Vec<String>,
}

impl SyncReport {
    fn status(status: SyncStatus) -> Self {
        Self {
            status,
            local_changed: false,
            copies_left: Vec::new(),
        }
    }
}

/// The conflicts of one synced file: the todo file (`item` empty) or one
/// of its notes (`item` the note name).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemConflicts {
    pub item: String,
    pub conflicts: Vec<Conflict>,
}

/// The result of sharing a file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShareOutcome {
    Shared,
    /// A shared copy of that name exists already: join it or pick another
    /// name.
    Exists,
}

#[derive(Clone)]
pub struct SyncService {
    files: Arc<dyn FileSystem>,
    stores: Arc<dyn RemoteStores>,
    links: Arc<dyn SyncLinkStore>,
    /// Syncs never overlap; one requested while another runs waits for it.
    running: Arc<Mutex<()>>,
}

/// One synced file: the todo file or a note.
struct Item {
    /// Empty for the todo file, else the note name.
    key: String,
    local: PathBuf,
    shared: String,
}

/// Why syncing one file stopped.
enum Failure {
    /// The store could not be reached.
    Offline(AppError),
    /// The local file could not be read or written.
    Local(AppError),
}

/// Changes the shared side of a file before it is merged, given the base:
/// the todo file merges in conflict copies a sync tool made of it.
type Prepare<'a> = dyn FnMut(Option<&str>, &mut String) -> Result<(), AppError> + 'a;

/// How syncing one file went.
enum Outcome {
    Clean { local_changed: bool },
    Conflicts(usize),
    SharedMissing,
}

impl SyncService {
    pub fn new(files: Arc<dyn FileSystem>, stores: Arc<dyn RemoteStores>, links: Arc<dyn SyncLinkStore>) -> Self {
        Self {
            files,
            stores,
            links,
            running: Arc::new(Mutex::new(())),
        }
    }

    pub fn links(&self) -> Result<Vec<SyncLink>, AppError> {
        self.links.load()
    }

    /// Local folders whose changes mean a shared copy changed, for the file
    /// watcher.
    pub fn watch_paths(&self, links: &[SyncLink]) -> Vec<PathBuf> {
        let paths: BTreeSet<PathBuf> = links
            .iter()
            .filter_map(|link| self.stores.open(&link.store).watch_path())
            .collect();
        paths.into_iter().collect()
    }

    /// Whether the shared copy `name` exists in `store`.
    pub fn exists(&self, store: &StoreLocation, name: &str) -> Result<bool, AppError> {
        Ok(self.stores.open(store).get(name)?.is_some())
    }

    /// Links `local` to a new shared copy `name` in `store` and copies the
    /// file there. Its notes follow with the next sync.
    pub fn share(&self, local: &Path, store: StoreLocation, name: &str) -> Result<ShareOutcome, AppError> {
        let _running = self.lock();
        let remote = self.stores.open(&store);
        if remote.get(name)?.is_some() {
            return Ok(ShareOutcome::Exists);
        }
        let content = self.files.read(local)?;
        match remote.put(name, &content, None) {
            Err(AppError::Changed(_)) => return Ok(ShareOutcome::Exists),
            result => result?,
        };
        self.links.remove_bases(local)?;
        self.links.set_base(local, "", &content)?;
        self.save_link(SyncLink {
            local: local.to_path_buf(),
            store,
            name: name.to_owned(),
        })?;
        Ok(ShareOutcome::Shared)
    }

    /// Links `local` to the existing shared copy `name` in `store`. A
    /// missing local file becomes a copy of it; an empty shared copy a copy
    /// of the local file. Otherwise both are merged, every differing line a
    /// conflict, since there is no base.
    pub fn join(&self, local: &Path, store: StoreLocation, name: &str) -> Result<SyncReport, AppError> {
        let _running = self.lock();
        let remote = self.stores.open(&store);
        let (content, version) = remote
            .get(name)?
            .ok_or_else(|| AppError::io(name, "the shared copy does not exist"))?;
        self.links.remove_bases(local)?;
        if !self.files.exists(local) {
            self.files.write(local, &content, true)?;
            self.links.set_base(local, "", &content)?;
        } else if content.trim().is_empty() {
            let mine = self.files.read(local)?;
            remote.put(name, &mine, Some(&version))?;
            self.links.set_base(local, "", &mine)?;
        }
        self.save_link(SyncLink {
            local: local.to_path_buf(),
            store,
            name: name.to_owned(),
        })?;
        self.sync_locked(local, &[])
    }

    /// Syncs a linked file and the notes its todos link to. Notes at a path
    /// in `skip` (open in an editor) wait for a later sync, so the editor
    /// never overwrites a merge.
    pub fn sync(&self, local: &Path, skip: &[PathBuf]) -> Result<SyncReport, AppError> {
        let _running = self.lock();
        self.sync_locked(local, skip)
    }

    /// The open conflicts of a linked file and its notes, found by merging
    /// again without writing anything.
    pub fn conflicts(&self, local: &Path) -> Result<Vec<ItemConflicts>, AppError> {
        let link = self.find(local)?;
        let store = self.stores.open(&link.store);
        let mut found = Vec::new();
        for item in self.items(&link)? {
            if let Some((merge, ..)) = self.merge_item(store.as_ref(), &link, &item)?
                && merge.has_conflicts()
            {
                found.push(ItemConflicts {
                    item: item.key,
                    conflicts: merge.conflicts().cloned().collect(),
                });
            }
        }
        Ok(found)
    }

    /// Settles conflicts with the user's choices, one list per file in the
    /// order [`Self::conflicts`] returned them, writes the result to both
    /// sides and syncs the rest. Fails with [`AppError::Changed`] when a
    /// side changed since the conflicts were shown.
    pub fn resolve(
        &self,
        local: &Path,
        choices: &[(String, Vec<Choice>)],
        skip: &[PathBuf],
    ) -> Result<SyncReport, AppError> {
        let _running = self.lock();
        let link = self.find(local)?;
        let store = self.stores.open(&link.store);
        let items = self.items(&link)?;
        for (key, choices) in choices {
            let Some(item) = items.iter().find(|item| &item.key == key) else {
                continue;
            };
            let changed = || AppError::Changed(item.local.clone());
            let (merge, mine, theirs, version) = self.merge_item(store.as_ref(), &link, item)?.ok_or_else(changed)?;
            if merge.conflicts().count() != choices.len() {
                return Err(changed());
            }
            let text = merge.resolve(choices);
            if text != mine {
                self.files.write_if_unchanged(&item.local, &text, &mine, true)?;
            }
            if text != theirs {
                store.put(&item.shared, &text, Some(&version))?;
            }
            self.links.set_base(&link.local, &item.key, &text)?;
        }
        self.sync_locked(local, skip)
    }

    /// Removes the link and its base copies. The local file and the shared
    /// copy stay as they are.
    pub fn unlink(&self, local: &Path) -> Result<(), AppError> {
        let _running = self.lock();
        let mut links = self.links.load()?;
        links.retain(|link| link.local != local);
        self.links.save(&links)?;
        self.links.remove_bases(local)
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, ()> {
        self.running.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn find(&self, local: &Path) -> Result<SyncLink, AppError> {
        self.links
            .load()?
            .into_iter()
            .find(|link| link.local == local)
            .ok_or_else(|| AppError::Other(format!("{} is not shared", local.display())))
    }

    fn save_link(&self, link: SyncLink) -> Result<(), AppError> {
        let mut links = self.links.load()?;
        links.retain(|existing| existing.local != link.local);
        links.push(link);
        self.links.save(&links)
    }

    fn sync_locked(&self, local: &Path, skip: &[PathBuf]) -> Result<SyncReport, AppError> {
        let link = self.find(local)?;
        let store = self.stores.open(&link.store);
        let mut report = SyncReport::status(SyncStatus::InSync);
        let mut conflicts = 0;
        for item in self.items(&link)? {
            if skip.contains(&item.local) {
                continue;
            }
            let outcome = if item.key.is_empty() {
                self.sync_file(store.as_ref(), &link, &item, &mut report.copies_left)
            } else {
                self.sync_item(store.as_ref(), &link, &item, &mut |_, _| Ok(()))
            };
            match outcome {
                Ok(Outcome::Clean { local_changed }) => report.local_changed |= local_changed,
                Ok(Outcome::Conflicts(count)) => conflicts += count,
                // Without the shared copy there is nothing to sync notes with.
                Ok(Outcome::SharedMissing) => return Ok(SyncReport::status(SyncStatus::SharedMissing)),
                Err(Failure::Offline(error)) => {
                    report.status = SyncStatus::Offline(error);
                    return Ok(report);
                }
                Err(Failure::Local(error)) => return Err(error),
            }
        }
        if conflicts > 0 {
            report.status = SyncStatus::Conflict(conflicts);
        }
        Ok(report)
    }

    /// The todo file and the notes its todos link to by name. Notes linked
    /// by an absolute path stay where they are and are not synced.
    fn items(&self, link: &SyncLink) -> Result<Vec<Item>, AppError> {
        let content = self.files.read(&link.local)?;
        let names: BTreeSet<NoteName> = TodoDocument::parse(&content, &[])
            .todos()
            .filter_map(|entry| match entry.todo.note_link()? {
                Ok(NoteLink::Folder(name)) => Some(name),
                _ => None,
            })
            .collect();
        let folder = notes_folder(&link.name);
        let file = Item {
            key: String::new(),
            local: link.local.clone(),
            shared: link.name.clone(),
        };
        let notes = names.into_iter().map(|name| Item {
            key: name.as_str().to_owned(),
            local: NoteService::note_path(&link.local, &name),
            shared: format!("{folder}/{name}"),
        });
        Ok(std::iter::once(file).chain(notes).collect())
    }

    /// The todo file: like a note, but conflict copies a sync tool made of
    /// the shared copy are merged into it first, and a missing shared copy
    /// is reported instead of created again.
    fn sync_file(
        &self,
        store: &dyn RemoteStore,
        link: &SyncLink,
        item: &Item,
        copies_left: &mut Vec<String>,
    ) -> Result<Outcome, Failure> {
        let mut merged_copies = Vec::new();
        let outcome = self.sync_item(store, link, item, &mut |base, shared| {
            merged_copies.clear();
            copies_left.clear();
            for copy in store.conflict_copies(&item.shared)? {
                let Some((content, _)) = store.get(&copy)? else {
                    continue;
                };
                let merge = merge3(base.unwrap_or_default(), shared, &content);
                if merge.has_conflicts() || base.is_none() {
                    copies_left.push(copy);
                } else {
                    *shared = merge.text();
                    merged_copies.push(copy);
                }
            }
            Ok(())
        })?;
        if matches!(outcome, Outcome::Clean { .. }) {
            for copy in merged_copies {
                store.remove(&copy).map_err(Failure::Offline)?;
            }
        }
        Ok(outcome)
    }

    /// Syncs one file with its shared copy. `prepare` may change the shared
    /// side before the merge, given the base.
    fn sync_item(
        &self,
        store: &dyn RemoteStore,
        link: &SyncLink,
        item: &Item,
        prepare: &mut Prepare<'_>,
    ) -> Result<Outcome, Failure> {
        let is_file = item.key.is_empty();
        for _ in 0..ATTEMPTS {
            let local = if is_file || self.files.exists(&item.local) {
                Some(self.files.read(&item.local).map_err(Failure::Local)?)
            } else {
                None
            };
            let shared = store.get(&item.shared).map_err(Failure::Offline)?;
            let base = self.links.base(&link.local, &item.key).map_err(Failure::Local)?;
            let (local, (stored, version)) = match (local, shared) {
                (None, None) => return Ok(Outcome::Clean { local_changed: false }),
                (Some(_), None) if is_file => return Ok(Outcome::SharedMissing),
                // A new note on this side: copy it over.
                (Some(local), None) => {
                    match store.put(&item.shared, &local, None) {
                        Err(AppError::Changed(_)) => continue,
                        result => result.map_err(Failure::Offline)?,
                    };
                    self.set_base(link, item, &local)?;
                    return Ok(Outcome::Clean { local_changed: false });
                }
                // A new note on the other side: copy it here.
                (None, Some((shared, _))) => {
                    self.files.write(&item.local, &shared, true).map_err(Failure::Local)?;
                    self.set_base(link, item, &shared)?;
                    return Ok(Outcome::Clean { local_changed: true });
                }
                (Some(local), Some(shared)) => (local, shared),
            };
            let mut shared = stored.clone();
            prepare(base.as_deref(), &mut shared).map_err(Failure::Offline)?;
            let merge = merge(base.as_deref(), &local, &shared);
            let text = merge.text();
            if text != local {
                match self.files.write_if_unchanged(&item.local, &text, &local, true) {
                    Err(AppError::Changed(_)) => continue,
                    result => result.map_err(Failure::Local)?,
                }
            }
            let conflicts = merge.conflicts().count();
            if conflicts > 0 {
                return Ok(Outcome::Conflicts(conflicts));
            }
            if text != stored {
                match store.put(&item.shared, &text, Some(&version)) {
                    Err(AppError::Changed(_)) => continue,
                    result => result.map_err(Failure::Offline)?,
                };
            }
            if base.as_deref() != Some(text.as_str()) {
                self.set_base(link, item, &text)?;
            }
            return Ok(Outcome::Clean {
                local_changed: text != local,
            });
        }
        Err(Failure::Local(AppError::Changed(item.local.clone())))
    }

    /// The merge of an item's two sides and what it was based on: the local
    /// text, the shared text and its version. `None` when a side is missing.
    fn merge_item(
        &self,
        store: &dyn RemoteStore,
        link: &SyncLink,
        item: &Item,
    ) -> Result<Option<(Merge, String, String, Version)>, AppError> {
        if !self.files.exists(&item.local) {
            return Ok(None);
        }
        let local = self.files.read(&item.local)?;
        let Some((shared, version)) = store.get(&item.shared)? else {
            return Ok(None);
        };
        let base = self.links.base(&link.local, &item.key)?;
        Ok(Some((merge(base.as_deref(), &local, &shared), local, shared, version)))
    }

    fn set_base(&self, link: &SyncLink, item: &Item, content: &str) -> Result<(), Failure> {
        self.links
            .set_base(&link.local, &item.key, content)
            .map_err(Failure::Local)
    }
}

/// A three-way merge, or without a base (a file that just joined, lost base
/// copies) a merge where every difference is a conflict.
fn merge(base: Option<&str>, local: &str, shared: &str) -> Merge {
    match base {
        Some(base) => merge3(base, local, shared),
        None => merge2(local, shared),
    }
}

#[cfg(test)]
mod tests;
