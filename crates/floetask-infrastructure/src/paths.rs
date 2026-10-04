//! Where floetask keeps its files, following the XDG Base Directory split:
//!
//! | Directory | Default | Holds |
//! |---|---|---|
//! | config | `~/.config/floetask` | `config.toml`, `colors.toml`: settings the user edits |
//! | data | `~/.local/share/floetask` | `filters.toml`: saved searches, worth keeping |
//! | state | `~/.local/state/floetask` | `state.toml`, `notified.toml`: where you left off, history |
//!
//! The `XDG_*_HOME` variables override the defaults. The same layout is used
//! on Windows and macOS, relative to the user's home directory.

use std::env;
use std::path::{Path, PathBuf};

const APP_DIR: &str = "floetask";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppPaths {
    pub config_file: PathBuf,
    pub config_dir: PathBuf,
    pub data_dir: PathBuf,
    pub state_dir: PathBuf,
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
            data_dir: base("XDG_DATA_HOME", &[".local", "share"]),
            state_dir: base("XDG_STATE_HOME", &[".local", "state"]),
        }
    }

    /// Colour overrides live next to the config file.
    pub fn colors_file(&self) -> PathBuf {
        self.config_dir.join("colors.toml")
    }

    pub fn filters_file(&self) -> PathBuf {
        self.data_dir.join("filters.toml")
    }

    pub fn state_file(&self) -> PathBuf {
        self.state_dir.join("state.toml")
    }

    pub fn notified_file(&self) -> PathBuf {
        self.state_dir.join("notified.toml")
    }
}

fn home_dir() -> PathBuf {
    env::home_dir().unwrap_or_else(|| PathBuf::from("."))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_override_moves_colors_next_to_it() {
        let paths = AppPaths::resolve(Some(PathBuf::from("/tmp/custom/floetask.toml")));
        assert_eq!(paths.config_file, PathBuf::from("/tmp/custom/floetask.toml"));
        assert_eq!(paths.colors_file(), PathBuf::from("/tmp/custom/colors.toml"));
        assert!(paths.state_file().ends_with("floetask/state.toml"));
        assert!(paths.filters_file().ends_with("floetask/filters.toml"));
    }
}
