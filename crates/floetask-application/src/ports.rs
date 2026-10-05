//! Interfaces to the outside world, implemented by the infrastructure crate.

use std::path::{Path, PathBuf};

use floetask_domain::Date;

use crate::colors::ColorOverrides;
use crate::error::AppError;
use crate::saved_filters::SavedFilter;
use crate::settings::{Settings, WatcherOptions};
use crate::state::AppState;

pub trait FileSystem: Send + Sync {
    /// Reads a whole text file.
    fn read(&self, path: &Path) -> Result<String, AppError>;

    /// Replaces a file's content. With `safe`, the write goes through a
    /// verified temporary file and a backup so a crash cannot lose data.
    fn write(&self, path: &Path, content: &str, safe: bool) -> Result<(), AppError>;

    fn exists(&self, path: &Path) -> bool;

    /// Deletes a file. Only used to finish moving a note to another folder.
    fn remove(&self, path: &Path) -> Result<(), AppError>;
}

pub trait Clock: Send + Sync {
    fn today(&self) -> Date;
}

/// User settings (the config file).
pub trait SettingsStore: Send + Sync {
    fn load(&self) -> Result<Settings, AppError>;
    fn save(&self, settings: &Settings) -> Result<(), AppError>;
    /// Where settings live, shown in the settings dialog.
    fn location(&self) -> PathBuf;
}

/// "Where I left off": registered files, layout, view toggles.
pub trait StateStore: Send + Sync {
    fn load(&self) -> AppState;
    fn save(&self, state: &AppState) -> Result<(), AppError>;
}

/// Saved search filters: user data worth keeping.
pub trait SavedFilterStore: Send + Sync {
    fn load(&self) -> Result<Vec<SavedFilter>, AppError>;
    fn save(&self, filters: &[SavedFilter]) -> Result<(), AppError>;
    fn location(&self) -> PathBuf;
}

/// Remembers which notifications were already shown.
pub trait NotificationLog: Send + Sync {
    fn contains(&self, key: &str) -> bool;
    fn insert(&self, key: &str) -> Result<(), AppError>;
}

pub trait Notifier: Send + Sync {
    fn notify(&self, title: &str, body: &str) -> Result<(), AppError>;
}

/// Keeps a watch alive; dropping it stops watching.
pub struct WatchGuard(pub Box<dyn Send>);

pub trait FileWatcher: Send + Sync {
    /// Calls `on_change` with the path of every watched file that changed,
    /// after the change settled for the configured debounce time.
    fn watch(
        &self,
        paths: &[PathBuf],
        options: &WatcherOptions,
        on_change: Box<dyn Fn(PathBuf) + Send + Sync>,
    ) -> Result<WatchGuard, AppError>;
}

/// Hands things over to the operating system.
pub trait Desktop: Send + Sync {
    /// Opens a link with the OS handler (`https://`, `file://`, custom schemes).
    fn open_uri(&self, uri: &str) -> Result<(), AppError>;
    /// Shows a file in the OS file manager.
    fn reveal(&self, path: &Path) -> Result<(), AppError>;
    /// Opens a file in the app the OS associates with its type.
    fn open_file(&self, path: &Path) -> Result<(), AppError>;
}

/// The colour file overriding theme colours.
pub trait ColorStore: Send + Sync {
    fn load(&self) -> Result<ColorOverrides, AppError>;
    fn location(&self) -> PathBuf;
}

/// A newer release than the running one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AvailableUpdate {
    pub version: String,
    /// Release notes, when the release has any.
    pub notes: Option<String>,
}

/// Finds and installs new releases of floetask.
pub trait Updater: Send + Sync {
    /// Whether this build knows where releases are published and how to
    /// verify them. Local builds usually do not.
    fn is_configured(&self) -> bool;
    /// Asks the release server for a newer version.
    fn check(&self) -> Result<Option<AvailableUpdate>, AppError>;
    /// Downloads the newest release, verifies its signature and starts its
    /// installer. On success the installer replaces the running app, which
    /// may exit before this returns.
    fn install_latest(&self) -> Result<(), AppError>;
}
