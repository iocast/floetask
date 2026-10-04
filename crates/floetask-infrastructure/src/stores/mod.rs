//! TOML-backed stores. Each store owns its file format (a DTO) and maps it
//! to and from application types, so serialisation never leaks inward.

pub mod filters;
pub mod notified;
pub mod settings;
pub mod state;

use std::fs;
use std::path::Path;

use serde::Serialize;
use serde::de::DeserializeOwned;

use floetask_application::AppError;

use crate::fs::{ensure_parent, safe_write};

/// Reads a TOML file; `Ok(None)` when it does not exist.
pub(crate) fn read_toml<T: DeserializeOwned>(path: &Path) -> Result<Option<T>, AppError> {
    match fs::read_to_string(path) {
        Ok(text) => toml::from_str(&text)
            .map(Some)
            .map_err(|e| AppError::Config(format!("{}: {e}", path.display()))),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(AppError::io(path, e)),
    }
}

pub(crate) fn write_toml<T: Serialize>(path: &Path, value: &T) -> Result<(), AppError> {
    let text = toml::to_string_pretty(value).map_err(|e| AppError::Config(e.to_string()))?;
    ensure_parent(path)?;
    safe_write(path, &text)
}
