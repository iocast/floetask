//! Interfaces to the outside world, implemented by the infrastructure crate.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use floetask_domain::Date;

use crate::colors::ColorOverrides;
use crate::error::AppError;
use crate::saved_filters::SavedFilter;
use crate::settings::{Settings, WatcherOptions};
use crate::state::AppState;
use crate::sync::{StoreLocation, SyncLink};

pub trait FileSystem: Send + Sync {
    /// Reads a whole text file.
    fn read(&self, path: &Path) -> Result<String, AppError>;

    /// Replaces a file's content. With `safe`, the write goes through a
    /// verified temporary file and a backup so a crash cannot lose data.
    fn write(&self, path: &Path, content: &str, safe: bool) -> Result<(), AppError>;

    /// Replaces a file's content only if it still holds `expected`, the
    /// content it was read with; otherwise fails with
    /// [`AppError::Changed`], so a sync and an edit never overwrite each
    /// other's change. The default checks just before writing; the real
    /// file system checks just before the final rename.
    fn write_if_unchanged(&self, path: &Path, content: &str, expected: &str, safe: bool) -> Result<(), AppError> {
        if self.read(path)? != expected {
            return Err(AppError::Changed(path.to_path_buf()));
        }
        self.write(path, content, safe)
    }

    fn exists(&self, path: &Path) -> bool;

    /// Deletes a file: a note moved to another folder, or one the user
    /// deleted along with its todo.
    fn remove(&self, path: &Path) -> Result<(), AppError>;
}

pub trait Clock: Send + Sync {
    fn today(&self) -> Date;
}

/// The language the operating system shows its own interface in.
pub trait SystemLocale: Send + Sync {
    /// A locale tag such as `de-CH`, or `None` when the OS names none.
    fn language(&self) -> Option<String>;
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

/// A published release of floetask that this build can install.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PublishedRelease {
    /// Without the `v`, e.g. `0.4.0` or `0.4.0-rc.1`.
    pub version: String,
    /// A release candidate, offered on the preview channel only.
    pub prerelease: bool,
    /// Release notes, when the release has any.
    pub notes: Option<String>,
    /// Newer than the running build. Older releases are what a rollback
    /// can go back to.
    pub newer: bool,
}

/// Finds and installs releases of floetask.
pub trait Updater: Send + Sync {
    /// Whether this build knows where releases are published and how to
    /// verify them. Local builds usually do not.
    fn is_configured(&self) -> bool;
    /// The published releases other than the running one, newest first.
    fn releases(&self) -> Result<Vec<PublishedRelease>, AppError>;
    /// Downloads exactly this version, newer or older than the running one,
    /// verifies its signature and starts its installer. On success the
    /// installer replaces the running app, which may exit before this returns.
    fn install(&self, version: &str) -> Result<(), AppError>;
}

/// The version of a shared copy as its store reports it: a content hash, an
/// ETag. Only compared for equality.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Version(pub String);

/// Where shared copies live: a synced folder, later a bucket or a service.
/// Names are `/`-separated paths inside the store, such as `acme.txt` or
/// `acme-notes/plan.md`.
pub trait RemoteStore: Send + Sync {
    /// A shared copy and its version, or `None` when it does not exist.
    /// Fails when the store cannot be reached.
    fn get(&self, name: &str) -> Result<Option<(String, Version)>, AppError>;

    /// Writes a shared copy only if it is still at `expected`, or, with
    /// `None`, only if it does not exist yet. Fails with
    /// [`AppError::Changed`] otherwise. Returns the new version.
    fn put(&self, name: &str, content: &str, expected: Option<&Version>) -> Result<Version, AppError>;

    /// Copies a sync tool made of `name` when two computers changed it at
    /// once (`acme-PC.txt`, `acme (conflicted copy …).txt`), by name.
    fn conflict_copies(&self, name: &str) -> Result<Vec<String>, AppError>;

    /// Deletes a file; only used for conflict copies once merged.
    fn remove(&self, name: &str) -> Result<(), AppError>;

    /// A local folder whose changes mean the store changed, for stores the
    /// file watcher can follow.
    fn watch_path(&self) -> Option<PathBuf>;
}

/// Opens the store a link points at.
pub trait RemoteStores: Send + Sync {
    fn open(&self, location: &StoreLocation) -> Arc<dyn RemoteStore>;
}

/// The linked files and the base copy of each, the content both sides had
/// at their last sync.
pub trait SyncLinkStore: Send + Sync {
    fn load(&self) -> Result<Vec<SyncLink>, AppError>;
    fn save(&self, links: &[SyncLink]) -> Result<(), AppError>;
    /// The base copy of the linked file `local` (`item` empty) or of one of
    /// its notes (`item` the note name). `None` when there is none yet.
    fn base(&self, local: &Path, item: &str) -> Result<Option<String>, AppError>;
    fn set_base(&self, local: &Path, item: &str, content: &str) -> Result<(), AppError>;
    /// Forgets every base copy of a linked file.
    fn remove_bases(&self, local: &Path) -> Result<(), AppError>;
}
