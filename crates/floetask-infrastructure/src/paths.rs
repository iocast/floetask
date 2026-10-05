//! Where floetask keeps its files, following the XDG Base Directory
//! Specification (<https://specifications.freedesktop.org/basedir-spec/latest/>):
//!
//! | Directory | Default | Holds |
//! |---|---|---|
//! | config | `~/.config/floetask` | `config.toml`, `colors.toml`, `filters.toml`: what the user configures |
//! | state | `~/.local/state/floetask` | `state.toml`, `notified.toml`: where you left off, history |
//!
//! Saved filters are configuration: the user writes them on purpose and may
//! want to keep them under version control next to `config.toml`. Nothing
//! goes in the data or cache directories yet.
//!
//! The `XDG_*_HOME` variables override the defaults. The same layout is used
//! on Windows and macOS, relative to the user's home directory.

use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

const APP_DIR: &str = "floetask";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppPaths {
    pub config_file: PathBuf,
    pub config_dir: PathBuf,
    pub state_dir: PathBuf,
    /// The data directory older versions kept `filters.toml` in; read only to
    /// move that file, see [`migrate_legacy_files`].
    legacy_data_dir: PathBuf,
}

impl AppPaths {
    /// Resolves paths from the environment. `config_override` comes from the
    /// `--config` command-line flag.
    pub fn resolve(config_override: Option<PathBuf>) -> Self {
        let home = home_dir();
        let base = |variable: &str, fallback: &[&str]| -> PathBuf {
            env::var_os(variable)
                .map(PathBuf::from)
                .filter(|path| path.is_absolute())
                .unwrap_or_else(|| fallback.iter().fold(home.clone(), |path, part| path.join(part)))
                .join(APP_DIR)
        };
        let config_dir = base("XDG_CONFIG_HOME", &[".config"]);
        let config_file = config_override.unwrap_or_else(|| config_dir.join("config.toml"));
        Self {
            config_dir: config_file.parent().map(Path::to_path_buf).unwrap_or(config_dir),
            config_file,
            state_dir: base("XDG_STATE_HOME", &[".local", "state"]),
            legacy_data_dir: base("XDG_DATA_HOME", &[".local", "share"]),
        }
    }

    /// Colour overrides live next to the config file.
    pub fn colors_file(&self) -> PathBuf {
        self.config_dir.join("colors.toml")
    }

    /// Saved filters live next to the config file.
    pub fn filters_file(&self) -> PathBuf {
        self.config_dir.join("filters.toml")
    }

    pub fn state_file(&self) -> PathBuf {
        self.state_dir.join("state.toml")
    }

    pub fn notified_file(&self) -> PathBuf {
        self.state_dir.join("notified.toml")
    }
}

/// Moves files from where older versions kept them, so an upgrade keeps the
/// user's saved filters. Never overwrites a file that already exists at the
/// new location.
pub fn migrate_legacy_files(paths: &AppPaths) -> io::Result<()> {
    let legacy_filters = paths.legacy_data_dir.join("filters.toml");
    let filters = paths.filters_file();
    if !legacy_filters.is_file() || filters.exists() {
        return Ok(());
    }
    if let Some(parent) = filters.parent() {
        fs::create_dir_all(parent)?;
    }
    // A rename fails across file systems; copy and remove instead.
    if fs::rename(&legacy_filters, &filters).is_err() {
        fs::copy(&legacy_filters, &filters)?;
        fs::remove_file(&legacy_filters)?;
    }
    // Only succeeds when nothing else is left in the old directory.
    let _ = fs::remove_dir(&paths.legacy_data_dir);
    Ok(())
}

fn home_dir() -> PathBuf {
    env::home_dir().unwrap_or_else(|| PathBuf::from("."))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn paths_in(root: &Path) -> AppPaths {
        AppPaths {
            config_file: root.join("config/floetask/config.toml"),
            config_dir: root.join("config/floetask"),
            state_dir: root.join("state/floetask"),
            legacy_data_dir: root.join("share/floetask"),
        }
    }

    #[test]
    fn config_override_moves_colors_and_filters_next_to_it() {
        let paths = AppPaths::resolve(Some(PathBuf::from("/tmp/custom/floetask.toml")));
        assert_eq!(paths.config_file, PathBuf::from("/tmp/custom/floetask.toml"));
        assert_eq!(paths.colors_file(), PathBuf::from("/tmp/custom/colors.toml"));
        assert_eq!(paths.filters_file(), PathBuf::from("/tmp/custom/filters.toml"));
        assert!(paths.state_file().ends_with("floetask/state.toml"));
    }

    #[test]
    fn migration_moves_legacy_filters_into_config_dir() {
        let root = tempfile::tempdir().unwrap();
        let paths = paths_in(root.path());
        fs::create_dir_all(&paths.legacy_data_dir).unwrap();
        fs::write(paths.legacy_data_dir.join("filters.toml"), "old").unwrap();

        migrate_legacy_files(&paths).unwrap();

        assert_eq!(fs::read_to_string(paths.filters_file()).unwrap(), "old");
        assert!(!paths.legacy_data_dir.exists());
    }

    #[test]
    fn migration_keeps_existing_filters() {
        let root = tempfile::tempdir().unwrap();
        let paths = paths_in(root.path());
        fs::create_dir_all(&paths.legacy_data_dir).unwrap();
        fs::write(paths.legacy_data_dir.join("filters.toml"), "old").unwrap();
        fs::create_dir_all(&paths.config_dir).unwrap();
        fs::write(paths.filters_file(), "new").unwrap();

        migrate_legacy_files(&paths).unwrap();

        assert_eq!(fs::read_to_string(paths.filters_file()).unwrap(), "new");
        assert!(paths.legacy_data_dir.join("filters.toml").exists());
    }
}
