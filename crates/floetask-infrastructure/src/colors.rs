//! `colors.toml`: overrides for theme colours.
//!
//! ```toml
//! [light]
//! primary = "#3b82f6"
//! priority_a = "#e11d48"
//!
//! [dark]
//! background = "#111827"
//! ```

use std::path::PathBuf;

use floetask_application::ports::ColorStore;
use floetask_application::{AppError, ColorOverrides};

use crate::stores::read_toml;

pub struct TomlColorStore {
    path: PathBuf,
}

impl TomlColorStore {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }
}

impl ColorStore for TomlColorStore {
    fn load(&self) -> Result<ColorOverrides, AppError> {
        Ok(read_toml(&self.path)?.unwrap_or_default())
    }

    fn location(&self) -> PathBuf {
        self.path.clone()
    }
}

#[cfg(test)]
mod tests {
    use floetask_application::PaletteOverrides;

    use super::*;

    #[test]
    fn reads_overrides() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("colors.toml");
        std::fs::write(&path, "[dark]\nprimary = \"#ff0000\"\n").unwrap();
        let colors = TomlColorStore::new(path).load().unwrap();
        assert_eq!(colors.dark.primary.as_deref(), Some("#ff0000"));
        assert_eq!(colors.light, PaletteOverrides::default());
    }

    #[test]
    fn unknown_colour_name_is_an_error() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("colors.toml");
        std::fs::write(&path, "[dark]\nprimry = \"#ff0000\"\n").unwrap();
        assert!(TomlColorStore::new(path).load().is_err());
    }
}
