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

use serde::Deserialize;

use floetask_application::ports::ColorStore;
use floetask_application::{AppError, ColorOverrides, PaletteOverrides};

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
        let file: ColorsFile = read_toml(&self.path)?.unwrap_or_default();
        Ok(ColorOverrides {
            light: file.light.into(),
            dark: file.dark.into(),
        })
    }

    fn location(&self) -> PathBuf {
        self.path.clone()
    }
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct ColorsFile {
    light: PaletteFile,
    dark: PaletteFile,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct PaletteFile {
    background: Option<String>,
    text: Option<String>,
    primary: Option<String>,
    project: Option<String>,
    success: Option<String>,
    warning: Option<String>,
    danger: Option<String>,
    navigation: Option<String>,
    priority_a: Option<String>,
    priority_b: Option<String>,
    priority_c: Option<String>,
    priority_other: Option<String>,
}

impl From<PaletteFile> for PaletteOverrides {
    fn from(f: PaletteFile) -> Self {
        Self {
            background: f.background,
            text: f.text,
            primary: f.primary,
            project: f.project,
            success: f.success,
            warning: f.warning,
            danger: f.danger,
            navigation: f.navigation,
            priority_a: f.priority_a,
            priority_b: f.priority_b,
            priority_c: f.priority_c,
            priority_other: f.priority_other,
        }
    }
}

#[cfg(test)]
mod tests {
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
}
