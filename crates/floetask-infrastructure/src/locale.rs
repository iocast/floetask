use floetask_application::ports::SystemLocale;

/// The OS interface language: the user's language list on Windows and
/// macOS, `LC_ALL` / `LC_MESSAGES` / `LANG` on Linux.
pub struct OsLocale;

impl SystemLocale for OsLocale {
    fn language(&self) -> Option<String> {
        sys_locale::get_locale()
    }
}
