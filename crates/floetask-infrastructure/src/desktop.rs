//! OS integration: notifications, opening links, revealing files.

use std::path::Path;

use floetask_application::AppError;
use floetask_application::ports::{Desktop, Notifier};

pub struct DesktopNotifier;

impl Notifier for DesktopNotifier {
    fn notify(&self, title: &str, body: &str) -> Result<(), AppError> {
        notify_rust::Notification::new()
            .appname("floetask")
            .summary(title)
            .body(body)
            .show()
            .map(drop)
            .map_err(|e| AppError::Other(format!("notification failed: {e}")))
    }
}

pub struct OsDesktop;

impl Desktop for OsDesktop {
    fn open_uri(&self, uri: &str) -> Result<(), AppError> {
        open::that_detached(uri).map_err(|e| AppError::Other(format!("could not open {uri}: {e}")))
    }

    fn reveal(&self, path: &Path) -> Result<(), AppError> {
        let folder = path.parent().filter(|p| !p.as_os_str().is_empty()).unwrap_or(path);
        open::that_detached(folder).map_err(|e| AppError::io(folder, e))
    }

    fn open_file(&self, path: &Path) -> Result<(), AppError> {
        open::that_detached(path).map_err(|e| AppError::io(path, e))
    }
}
