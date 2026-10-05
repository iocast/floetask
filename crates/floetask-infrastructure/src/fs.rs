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
    if path.exists() {
        fs::copy(path, &backup).map_err(io)?;
    }
    fs::rename(&temporary, path).map_err(io)?;
    let _ = fs::remove_file(&backup);
    Ok(())
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
    fn creates_missing_directories() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("a/b/todo.txt");
        LocalFileSystem.write(&path, "x\n", false).unwrap();
        assert!(path.exists());
    }
}
