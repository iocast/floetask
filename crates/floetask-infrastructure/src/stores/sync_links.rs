//! `sync.toml`: the linked files, and the `sync/` folder next to it with
//! the base copy of each linked file and its notes (their content at the
//! last sync).
//!
//! Both are state: losing them is safe, since the next sync of a file
//! without a base copy treats every differing line as a conflict.

use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use floetask_application::AppError;
use floetask_application::ports::SyncLinkStore;
use floetask_application::sync::SyncLink;

use super::{read_toml, write_toml};
use crate::fs::{ensure_parent, read_current, safe_write};
use crate::remote::fnv1a;

pub struct TomlSyncLinkStore {
    file: PathBuf,
    bases: PathBuf,
}

impl TomlSyncLinkStore {
    /// `file` is `sync.toml`, `bases` the folder of base copies.
    pub fn new(file: PathBuf, bases: PathBuf) -> Self {
        Self { file, bases }
    }

    /// One folder per linked file, named after it and a hash of its path, so
    /// two `todo.txt` in different folders never share one.
    fn folder(&self, local: &Path) -> PathBuf {
        let stem = local
            .file_stem()
            .map(|stem| stem.to_string_lossy().into_owned())
            .unwrap_or_default();
        let hash = fnv1a(local.to_string_lossy().as_bytes());
        self.bases.join(format!("{stem}-{hash:016x}"))
    }

    /// The todo file's base is `file.txt`, a note's is under `notes/`.
    fn base_path(&self, local: &Path, item: &str) -> PathBuf {
        let folder = self.folder(local);
        if item.is_empty() {
            folder.join("file.txt")
        } else {
            item.split('/').fold(folder.join("notes"), |path, part| path.join(part))
        }
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(default)]
struct LinksFile {
    #[serde(rename = "link")]
    links: Vec<SyncLink>,
}

impl SyncLinkStore for TomlSyncLinkStore {
    fn load(&self) -> Result<Vec<SyncLink>, AppError> {
        Ok(read_toml::<LinksFile>(&self.file)?.unwrap_or_default().links)
    }

    fn save(&self, links: &[SyncLink]) -> Result<(), AppError> {
        write_toml(&self.file, &LinksFile { links: links.to_vec() })
    }

    fn base(&self, local: &Path, item: &str) -> Result<Option<String>, AppError> {
        read_current(&self.base_path(local, item))
    }

    fn set_base(&self, local: &Path, item: &str, content: &str) -> Result<(), AppError> {
        let path = self.base_path(local, item);
        ensure_parent(&path)?;
        safe_write(&path, content)
    }

    fn remove_bases(&self, local: &Path) -> Result<(), AppError> {
        let folder = self.folder(local);
        match fs::remove_dir_all(&folder) {
            Err(e) if e.kind() != std::io::ErrorKind::NotFound => Err(AppError::io(&folder, e)),
            _ => Ok(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use floetask_application::sync::StoreLocation;

    use super::*;

    #[test]
    fn keeps_links_and_base_copies() {
        let dir = tempfile::tempdir().unwrap();
        let store = TomlSyncLinkStore::new(dir.path().join("sync.toml"), dir.path().join("sync"));
        assert!(store.load().unwrap().is_empty());
        let link = SyncLink {
            local: PathBuf::from("/projects/acme/todo.txt"),
            store: StoreLocation::Folder {
                path: PathBuf::from("/OneDrive/acme"),
            },
            name: "acme.txt".to_owned(),
        };
        store.save(std::slice::from_ref(&link)).unwrap();
        assert_eq!(store.load().unwrap(), std::slice::from_ref(&link));
        let text = fs::read_to_string(dir.path().join("sync.toml")).unwrap();
        assert!(
            text.contains("[[link]]") && text.contains("kind = \"folder\""),
            "{text}"
        );

        assert_eq!(store.base(&link.local, "").unwrap(), None);
        store.set_base(&link.local, "", "a\n").unwrap();
        store.set_base(&link.local, "sub/plan.md", "# Plan\n").unwrap();
        assert_eq!(store.base(&link.local, "").unwrap().as_deref(), Some("a\n"));
        assert_eq!(
            store.base(&link.local, "sub/plan.md").unwrap().as_deref(),
            Some("# Plan\n")
        );
        assert_eq!(store.base(Path::new("/other/todo.txt"), "").unwrap(), None);
        store.remove_bases(&link.local).unwrap();
        assert_eq!(store.base(&link.local, "").unwrap(), None);
        store.remove_bases(&link.local).unwrap();
    }
}
