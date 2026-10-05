//! Infrastructure layer of floetask: implementations of the application
//! ports on top of the real file system, TOML files and the OS.
//!
//! File locations follow the XDG Base Directory split on every platform,
//! see [`paths::AppPaths`].

pub mod clock;
pub mod colors;
pub mod desktop;
pub mod fs;
pub mod paths;
pub mod stores;
pub mod updater;
pub mod watcher;

use std::sync::Arc;

use floetask_application::Ports;

pub use colors::TomlColorStore;
pub use paths::AppPaths;

/// Builds every port backed by real infrastructure.
pub fn ports(paths: &AppPaths) -> Ports {
    Ports {
        files: Arc::new(fs::LocalFileSystem),
        clock: Arc::new(clock::SystemClock),
        settings: Arc::new(stores::settings::TomlSettingsStore::new(paths.config_file.clone())),
        state: Arc::new(stores::state::TomlStateStore::new(paths.state_file())),
        saved_filters: Arc::new(stores::filters::TomlSavedFilterStore::new(paths.filters_file())),
        notification_log: Arc::new(stores::notified::TomlNotificationLog::new(paths.notified_file())),
        notifier: Arc::new(desktop::DesktopNotifier),
        watcher: Arc::new(watcher::NotifyFileWatcher),
        desktop: Arc::new(desktop::OsDesktop),
        colors: Arc::new(TomlColorStore::new(paths.colors_file())),
        updater: Arc::new(updater::ReleaseUpdater::from_build_env()),
    }
}
