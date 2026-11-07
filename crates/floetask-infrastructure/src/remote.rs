//! Remote stores: where shared copies of synced todo files live.
//!
//! The folder store keeps them in a folder that another tool syncs between
//! computers (the OneDrive app, Dropbox, Syncthing) or that several
//! computers reach (a network share). floetask only reads and writes local
//! files there; moving them between computers is the other tool's job.

use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

use floetask_application::AppError;
use floetask_application::ports::{RemoteStore, RemoteStores, Version};
use floetask_application::sync::StoreLocation;

use crate::fs::{ensure_parent, read_current, safe_write_if};

/// Opens the store a link points at.
pub struct LocalStores;

impl RemoteStores for LocalStores {
    fn open(&self, location: &StoreLocation) -> Arc<dyn RemoteStore> {
        match location {
            StoreLocation::Folder { path } => Arc::new(FolderRemoteStore::new(path.clone())),
        }
    }
}

/// Shared copies as files in a folder. A file's version is a hash of its
/// content, so a write can check that nobody changed it since it was read.
pub struct FolderRemoteStore {
    folder: PathBuf,
}

impl FolderRemoteStore {
    pub fn new(folder: PathBuf) -> Self {
        Self { folder }
    }

    fn path(&self, name: &str) -> PathBuf {
        name.split('/').fold(self.folder.clone(), |path, part| path.join(part))
    }

    /// An unplugged drive or a network share that is down: offline, not a
    /// missing shared copy.
    fn reachable(&self) -> Result<(), AppError> {
        if self.folder.is_dir() {
            Ok(())
        } else {
            Err(AppError::io(&self.folder, "the folder cannot be reached"))
        }
    }
}

impl RemoteStore for FolderRemoteStore {
    fn get(&self, name: &str) -> Result<Option<(String, Version)>, AppError> {
        self.reachable()?;
        // Reading a cloud-only file (OneDrive Files On-Demand) downloads it.
        Ok(read_current(&self.path(name))?.map(|text| {
            let version = version(&text);
            (text, version)
        }))
    }

    fn put(&self, name: &str, content: &str, expected: Option<&Version>) -> Result<Version, AppError> {
        self.reachable()?;
        let path = self.path(name);
        ensure_parent(&path)?;
        safe_write_if(&path, content, |current| current.map(version).as_ref() == expected)?;
        Ok(version(content))
    }

    fn conflict_copies(&self, name: &str) -> Result<Vec<String>, AppError> {
        let path = self.path(name);
        let (Some(folder), Some(file_name)) = (path.parent(), path.file_name()) else {
            return Ok(Vec::new());
        };
        let file_name = file_name.to_string_lossy();
        let prefix = name
            .rsplit_once('/')
            .map(|(folder, _)| format!("{folder}/"))
            .unwrap_or_default();
        let entries = match fs::read_dir(folder) {
            Ok(entries) => entries,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(e) => return Err(AppError::io(folder, e)),
        };
        let mut copies: Vec<String> = entries
            .filter_map(Result::ok)
            .filter(|entry| entry.file_type().is_ok_and(|kind| kind.is_file()))
            .map(|entry| entry.file_name().to_string_lossy().into_owned())
            .filter(|candidate| is_conflict_copy(&file_name, candidate))
            .map(|candidate| format!("{prefix}{candidate}"))
            .collect();
        copies.sort();
        Ok(copies)
    }

    fn remove(&self, name: &str) -> Result<(), AppError> {
        let path = self.path(name);
        fs::remove_file(&path).map_err(|e| AppError::io(&path, e))
    }

    fn watch_path(&self) -> Option<PathBuf> {
        Some(self.folder.clone())
    }
}

/// The version of a file's content: a 64-bit FNV-1a hash, stable across
/// builds and platforms.
fn version(content: &str) -> Version {
    Version(format!("{:016x}", fnv1a(content.as_bytes())))
}

pub(crate) fn fnv1a(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf2_9ce4_8422_2325, |hash, &byte| {
        (hash ^ u64::from(byte)).wrapping_mul(0x0100_0000_01b3)
    })
}

/// Whether `candidate` is a copy a sync tool made of `file_name` when two
/// computers changed it at once:
///
/// - OneDrive: `acme-DESKTOP-AB12.txt`, the computer's name added, sometimes
///   with `-1`. Only an upper-case name counts, so `acme-old.txt` or
///   `acme-2026.txt` are never taken for one.
/// - Dropbox, Nextcloud: `acme (conflicted copy 2026-11-07).txt`, also with
///   a user's name in the brackets.
/// - Syncthing: `acme.sync-conflict-20261107-101500-ABCDEFG.txt`.
///
/// Safe-write leftovers (`acme.txt.tmp`, `acme.txt.bak`) never match.
pub(crate) fn is_conflict_copy(file_name: &str, candidate: &str) -> bool {
    let (stem, extension) = match file_name.rfind('.') {
        Some(dot) if dot > 0 => file_name.split_at(dot),
        _ => (file_name, ""),
    };
    let Some(middle) = candidate
        .strip_prefix(stem)
        .and_then(|rest| rest.strip_suffix(extension))
        .filter(|middle| !middle.is_empty())
    else {
        return false;
    };
    if middle.starts_with(" (") && middle.ends_with(')') && middle.contains("conflicted copy") {
        return true;
    }
    if middle.starts_with(".sync-conflict-") {
        return true;
    }
    middle.strip_prefix('-').is_some_and(|machine| {
        let machine = match machine.rsplit_once('-') {
            Some((name, number)) if !number.is_empty() && number.bytes().all(|b| b.is_ascii_digit()) => name,
            _ => machine,
        };
        is_computer_name(machine)
    })
}

/// A Windows computer name as OneDrive writes it: up to 15 upper-case
/// letters, digits and hyphens, with at least one letter.
fn is_computer_name(name: &str) -> bool {
    (1..=15).contains(&name.len())
        && !name.starts_with('-')
        && name
            .bytes()
            .all(|b| b.is_ascii_uppercase() || b.is_ascii_digit() || b == b'-')
        && name.bytes().any(|b| b.is_ascii_uppercase())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn writes_only_over_the_version_read() {
        let dir = tempfile::tempdir().unwrap();
        let store = FolderRemoteStore::new(dir.path().to_path_buf());
        assert_eq!(store.get("acme.txt").unwrap(), None);
        let first = store.put("acme.txt", "a\n", None).unwrap();
        assert!(matches!(store.put("acme.txt", "b\n", None), Err(AppError::Changed(_))));
        let (text, version) = store.get("acme.txt").unwrap().unwrap();
        assert_eq!((text.as_str(), &version), ("a\n", &first));
        store.put("acme.txt", "b\n", Some(&version)).unwrap();
        assert!(matches!(
            store.put("acme.txt", "c\n", Some(&version)),
            Err(AppError::Changed(_))
        ));
        store.put("acme-notes/plan.md", "# Plan\n", None).unwrap();
        assert!(dir.path().join("acme-notes").join("plan.md").exists());
    }

    #[test]
    fn a_missing_folder_is_offline() {
        let dir = tempfile::tempdir().unwrap();
        let store = FolderRemoteStore::new(dir.path().join("unplugged"));
        assert!(store.get("acme.txt").is_err());
        assert!(store.put("acme.txt", "a\n", None).is_err());
    }

    #[test]
    fn finds_conflict_copies_of_the_sync_tools() {
        for copy in [
            "acme-DESKTOP-AB12.txt",
            "acme-LAPTOP-1.txt",
            "acme (conflicted copy 2026-11-07).txt",
            "acme (Kim's conflicted copy 2026-11-07).txt",
            "acme.sync-conflict-20261107-101500-ABCDEFG.txt",
        ] {
            assert!(is_conflict_copy("acme.txt", copy), "{copy}");
        }
        for other in [
            "acme.txt",
            "acme.txt.tmp",
            "acme.txt.bak",
            "acme-old.txt",
            "acme-2026.txt",
            "acme-notes",
            "acme-DESKTOP.md",
            "other-DESKTOP.txt",
        ] {
            assert!(!is_conflict_copy("acme.txt", other), "{other}");
        }
    }

    #[test]
    fn lists_conflict_copies_next_to_the_shared_copy() {
        let dir = tempfile::tempdir().unwrap();
        let store = FolderRemoteStore::new(dir.path().to_path_buf());
        for name in ["acme.txt", "acme-PC.txt", "acme-old.txt", "acme-notes/plan-PC.md"] {
            store.put(name, "x\n", None).unwrap();
        }
        assert_eq!(store.conflict_copies("acme.txt").unwrap(), ["acme-PC.txt"]);
        assert_eq!(
            store.conflict_copies("acme-notes/plan.md").unwrap(),
            ["acme-notes/plan-PC.md"]
        );
        store.remove("acme-PC.txt").unwrap();
        assert!(store.conflict_copies("acme.txt").unwrap().is_empty());
    }
}
