//! Local file system with optional safe writes.

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use floetask_application::AppError;
use floetask_application::ports::FileSystem;

pub struct LocalFileSystem;

impl FileSystem for LocalFileSystem {
    fn read(&self, path: &Path) -> Result<String, AppError> {
        fs::read_to_string(path).map_err(|e| AppError::io(path, e))
    }

    fn write(&self, path: &Path, content: &str, safe: bool) -> Result<(), AppError> {
        ensure_parent(path)?;
        if safe {
            safe_write(path, content)
        } else {
            fs::write(path, content).map_err(|e| AppError::io(path, e))
        }
    }

    fn write_if_unchanged(&self, path: &Path, content: &str, expected: &str, safe: bool) -> Result<(), AppError> {
        ensure_parent(path)?;
        let unchanged = |current: Option<&str>| current == Some(expected);
        if safe {
            safe_write_if(path, content, unchanged)
        } else if unchanged(read_current(path)?.as_deref()) {
            fs::write(path, content).map_err(|e| AppError::io(path, e))
        } else {
            Err(AppError::Changed(path.to_path_buf()))
        }
    }

    fn exists(&self, path: &Path) -> bool {
        path.exists()
    }

    fn remove(&self, path: &Path) -> Result<(), AppError> {
        fs::remove_file(path).map_err(|e| AppError::io(path, e))
    }
}

/// Writes `content` without risking the existing file:
///
/// 1. write and flush `<file>.tmp`, then read it back to verify,
/// 2. copy the current file to `<file>.bak`,
/// 3. atomically rename the temporary file over the target,
/// 4. delete the backup.
///
/// If anything fails, the original file is untouched or the backup remains.
pub fn safe_write(path: &Path, content: &str) -> Result<(), AppError> {
    safe_write_if(path, content, |_| true)
}

/// A [`safe_write`] that first checks the file's current content (`None`
/// when it does not exist) right before the rename, and fails with
/// [`AppError::Changed`] when `unchanged` refuses it. Checking that late
/// leaves the smallest window for another writer: a sync tool, a sync, an
/// edit. The check and the rename are still two steps; the sync's base copy
/// catches a change slipping in between.
pub(crate) fn safe_write_if(
    path: &Path,
    content: &str,
    unchanged: impl Fn(Option<&str>) -> bool,
) -> Result<(), AppError> {
    let io = |e: std::io::Error| AppError::io(path, e);
    let temporary = sibling(path, "tmp");
    let backup = sibling(path, "bak");

    let mut file = fs::File::create(&temporary).map_err(io)?;
    file.write_all(content.as_bytes()).map_err(io)?;
    file.sync_all().map_err(io)?;
    drop(file);

    if fs::read_to_string(&temporary).map_err(io)? != content {
        let _ = fs::remove_file(&temporary);
        return Err(AppError::io(path, "verification of the temporary file failed"));
    }
    let current = read_current(path);
    if !current.as_ref().is_ok_and(|current| unchanged(current.as_deref())) {
        let _ = fs::remove_file(&temporary);
        current?;
        return Err(AppError::Changed(path.to_path_buf()));
    }
    if path.exists() {
        fs::copy(path, &backup).map_err(io)?;
    }
    fs::rename(&temporary, path).map_err(io)?;
    let _ = fs::remove_file(&backup);
    Ok(())
}

/// A file's content, or `None` when it does not exist.
pub(crate) fn read_current(path: &Path) -> Result<Option<String>, AppError> {
    match fs::read_to_string(path) {
        Ok(text) => Ok(Some(text)),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(AppError::io(path, e)),
    }
}

pub(crate) fn ensure_parent(path: &Path) -> Result<(), AppError> {
    match path.parent() {
        Some(parent) if !parent.as_os_str().is_empty() => {
            fs::create_dir_all(parent).map_err(|e| AppError::io(parent, e))
        }
        _ => Ok(()),
    }
}

fn sibling(path: &Path, extension: &str) -> PathBuf {
    let mut name = path.file_name().unwrap_or_default().to_os_string();
    name.push(".");
    name.push(extension);
    path.with_file_name(name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn safe_write_replaces_and_cleans_up() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("todo.txt");
        let files = LocalFileSystem;
        files.write(&path, "one\n", true).unwrap();
        files.write(&path, "two\n", true).unwrap();
        assert_eq!(files.read(&path).unwrap(), "two\n");
        let leftovers: Vec<_> = fs::read_dir(dir.path()).unwrap().collect();
        assert_eq!(leftovers.len(), 1);
    }

    #[test]
    fn conditional_writes_refuse_a_changed_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("todo.txt");
        let files = LocalFileSystem;
        files.write(&path, "one\n", true).unwrap();
        for safe in [true, false] {
            let error = files.write_if_unchanged(&path, "two\n", "other\n", safe).unwrap_err();
            assert_eq!(error, AppError::Changed(path.clone()));
            assert_eq!(files.read(&path).unwrap(), "one\n");
        }
        files.write_if_unchanged(&path, "two\n", "one\n", true).unwrap();
        assert_eq!(files.read(&path).unwrap(), "two\n");
        assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 1);
    }

    #[test]
    fn creates_missing_directories() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("a/b/todo.txt");
        LocalFileSystem.write(&path, "x\n", false).unwrap();
        assert!(path.exists());
    }
}
