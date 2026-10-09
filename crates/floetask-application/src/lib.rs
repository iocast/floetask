//! Application layer of floetask.
//!
//! Use cases that combine domain rules with the outside world. The outside
//! world is reached only through the traits in [`ports`]; the infrastructure
//! crate implements them and the app crate wires everything together.
//!
//! Every service is cheap to clone and thread-safe, so the GUI can run use
//! cases on background tasks.

pub mod colors;
pub mod error;
pub mod notes;
pub mod notifications;
pub mod ports;
pub mod saved_filters;
pub mod settings;
pub mod state;
pub mod todo_files;

pub use colors::{ColorOverrides, PaletteOverrides};
pub use error::AppError;
pub use notes::{NoteService, NoteStatus};
pub use notifications::{DueNotification, NotificationService};
pub use saved_filters::SavedFilter;
pub use settings::{MonthOverflow, Settings, ThemePreference, WatcherOptions};
pub use state::{AppState, DrawerTab, MainView, TodoFileEntry, WindowState};
pub use todo_files::{FileOptions, InputOptions, TodoFileService, TodoRef};

use std::sync::Arc;

/// Every port the application needs, bundled for the composition root.
#[derive(Clone)]
pub struct Ports {
    pub files: Arc<dyn ports::FileSystem>,
    pub clock: Arc<dyn ports::Clock>,
    pub settings: Arc<dyn ports::SettingsStore>,
    pub state: Arc<dyn ports::StateStore>,
    pub saved_filters: Arc<dyn ports::SavedFilterStore>,
    pub notification_log: Arc<dyn ports::NotificationLog>,
    pub notifier: Arc<dyn ports::Notifier>,
    pub watcher: Arc<dyn ports::FileWatcher>,
    pub desktop: Arc<dyn ports::Desktop>,
    pub colors: Arc<dyn ports::ColorStore>,
    pub updater: Arc<dyn ports::Updater>,
}

/// The services the GUI uses. Built once from [`Ports`].
#[derive(Clone)]
pub struct Services {
    pub todo_files: TodoFileService,
    pub notes: NoteService,
    pub notifications: NotificationService,
    pub ports: Ports,
}

impl Services {
    pub fn new(ports: Ports) -> Self {
        Self {
            todo_files: TodoFileService::new(ports.files.clone(), ports.clock.clone()),
            notes: NoteService::new(ports.files.clone(), ports.desktop.clone()),
            notifications: NotificationService::new(
                ports.notifier.clone(),
                ports.notification_log.clone(),
                ports.clock.clone(),
            ),
            ports,
        }
    }
}
