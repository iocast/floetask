//! How Windows names floetask in notifications and the taskbar.
//!
//! A Windows notification shows the name and icon of the application user
//! model ID (AUMID) it is sent under. Without one, notify-rust sends as
//! PowerShell. floetask therefore registers its own AUMID for the current
//! user (`HKCU\Software\Classes\AppUserModelId\<id>` with a display name and
//! an icon), sends notifications under it and runs under it, so the taskbar
//! groups its windows the same way. Other systems name the app from the
//! notification itself and need none of this.

use std::io;
use std::path::Path;

/// The application user model ID; the same as the installers' identifier.
pub const APP_ID: &str = "com.iocast.floetask";

/// Shown as the sender of notifications.
const DISPLAY_NAME: &str = "floetask";

#[cfg_attr(not(windows), allow(dead_code))]
const ICON_PNG: &[u8] = include_bytes!("../../../assets/icon-256.png");

/// Registers floetask's name and icon for notifications and adopts the ID
/// for this process. The icon is written to `cache_dir`, since Windows reads
/// it from a file. Call once at start-up; it is cheap and idempotent.
pub fn register(cache_dir: &Path) -> io::Result<()> {
    #[cfg(windows)]
    {
        let icon = cache_dir.join("floetask.png");
        if std::fs::read(&icon).ok().as_deref() != Some(ICON_PNG) {
            std::fs::create_dir_all(cache_dir)?;
            std::fs::write(&icon, ICON_PNG)?;
        }
        windows::register(&icon.to_string_lossy())
    }
    #[cfg(not(windows))]
    {
        let _ = cache_dir;
        Ok(())
    }
}

#[cfg(windows)]
mod windows {
    use std::io;

    use windows_sys::Win32::System::Registry::{HKEY_CURRENT_USER, REG_SZ, RegSetKeyValueW};
    use windows_sys::Win32::UI::Shell::SetCurrentProcessExplicitAppUserModelID;

    use super::{APP_ID, DISPLAY_NAME};

    fn wide(text: &str) -> Vec<u16> {
        text.encode_utf16().chain(std::iter::once(0)).collect()
    }

    pub(super) fn register(icon_path: &str) -> io::Result<()> {
        let key = wide(&format!(r"Software\Classes\AppUserModelId\{APP_ID}"));
        for (name, value) in [("DisplayName", DISPLAY_NAME), ("IconUri", icon_path)] {
            let name = wide(name);
            let value = wide(value);
            // SAFETY: every pointer is a NUL-terminated UTF-16 buffer that
            // outlives the call; the data length is in bytes, NUL included.
            let status = unsafe {
                RegSetKeyValueW(
                    HKEY_CURRENT_USER,
                    key.as_ptr(),
                    name.as_ptr(),
                    REG_SZ,
                    value.as_ptr().cast(),
                    (value.len() * 2) as u32,
                )
            };
            if status != 0 {
                return Err(io::Error::from_raw_os_error(status as i32));
            }
        }
        let id = wide(APP_ID);
        // SAFETY: `id` is a NUL-terminated UTF-16 buffer that outlives the call.
        let result = unsafe { SetCurrentProcessExplicitAppUserModelID(id.as_ptr()) };
        if result < 0 {
            return Err(io::Error::other(format!("could not set the app ID ({result:#x})")));
        }
        Ok(())
    }
}
