//! `config.toml`: user settings.
//!
//! `Settings` derives serde itself; this store adds the schema version and
//! owns reading and writing the file.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use floetask_application::ports::SettingsStore;
use floetask_application::{AppError, Settings};

use super::{read_toml, write_toml};

/// Bump when the file format changes incompatibly, and migrate in `load`.
const SCHEMA_VERSION: u32 = 1;

pub struct TomlSettingsStore {
    path: PathBuf,
}

impl TomlSettingsStore {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }
}

impl SettingsStore for TomlSettingsStore {
    fn load(&self) -> Result<Settings, AppError> {
        let file: Option<SettingsFile> = read_toml(&self.path)?;
        Ok(file.map(|file| file.settings).unwrap_or_default().normalized())
    }

    fn save(&self, settings: &Settings) -> Result<(), AppError> {
        let file = SettingsFile {
            schema_version: SCHEMA_VERSION,
            settings: settings.clone(),
        };
        write_toml(&self.path, &file)
    }

    fn location(&self) -> PathBuf {
        self.path.clone()
    }
}

#[derive(Serialize, Deserialize)]
struct SettingsFile {
    #[serde(default)]
    schema_version: u32,
    #[serde(flatten)]
    settings: Settings,
}

#[cfg(test)]
mod tests {
    use floetask_application::{MonthOverflow, ThemePreference, UpdateChannel};
    use floetask_domain::board::BoardColumns;
    use floetask_domain::{StatusSet, WeekStart};

    use super::*;

    #[test]
    fn reads_the_format_older_versions_wrote() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");
        std::fs::write(
            &path,
            r###"schema_version = 1
append_creation_date = true
convert_relative_dates = true
human_friendly_dates = false
safe_writes = true
bulk_creation = false
disable_animations = false
compact = true
notifications = true
check_for_updates = true
update_channel = "preview"
notification_threshold_days = 3
zoom_percent = 110
theme = "dark"
week_start = "sunday"
month_overflow = "scroll"
language = "de"
exclude_lines_with_prefix = ["##"]

[watcher]
debounce_ms = 250
polling = false
poll_interval_ms = 1000

[statuses]
order = ["doing", { key = "in-review", label = "In review" }, "todo", "waiting", "someday"]
hidden = ["someday"]

[[boards]]
file = "/home/me/todo.txt"
columns = ["doing", "done"]
"###,
        )
        .unwrap();
        let settings = TomlSettingsStore::new(path).load().unwrap();
        assert!(settings.append_creation_date && settings.compact);
        assert_eq!(settings.update_channel, UpdateChannel::Preview);
        assert_eq!(settings.notification_threshold_days, 3);
        assert_eq!(settings.zoom_percent, 110);
        assert_eq!(settings.theme, ThemePreference::Dark);
        assert_eq!(settings.week_start, WeekStart::Sunday);
        assert_eq!(settings.month_overflow, MonthOverflow::Scroll);
        assert_eq!(settings.language, "de");
        assert_eq!(settings.exclude_lines_with_prefix, ["##"]);
        assert_eq!(settings.watcher.debounce_ms, 250);
        assert_eq!(settings.statuses.label("in-review"), Some("In review"));
        assert_eq!(
            settings.board_columns(std::path::Path::new("/home/me/todo.txt")).keys(),
            ["doing", "done"]
        );
    }

    #[test]
    fn unknown_names_fall_back_to_defaults() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");
        std::fs::write(
            &path,
            "theme = \"blue\"\nweek_start = \"friday\"\nmonth_overflow = \"x\"\nupdate_channel = \"nightly\"\n",
        )
        .unwrap();
        let settings = TomlSettingsStore::new(path).load().unwrap();
        assert_eq!(settings.theme, ThemePreference::System);
        assert_eq!(settings.week_start, WeekStart::Monday);
        assert_eq!(settings.month_overflow, MonthOverflow::More);
        assert_eq!(settings.update_channel, UpdateChannel::Stable);
    }

    #[test]
    fn writes_the_schema_version() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");
        TomlSettingsStore::new(path.clone()).save(&Settings::default()).unwrap();
        let written = std::fs::read_to_string(&path).unwrap();
        assert!(written.starts_with("schema_version = 1\n"), "{written}");
    }

    #[test]
    fn missing_file_gives_defaults_and_round_trips() {
        let dir = tempfile::tempdir().unwrap();
        let store = TomlSettingsStore::new(dir.path().join("config.toml"));
        assert_eq!(store.load().unwrap(), Settings::default());

        let settings = Settings {
            compact: true,
            theme: ThemePreference::Dark,
            week_start: WeekStart::Sunday,
            month_overflow: MonthOverflow::Scroll,
            update_channel: UpdateChannel::Preview,
            exclude_lines_with_prefix: vec!["##".into()],
            ..Settings::default()
        };
        store.save(&settings).unwrap();
        assert_eq!(store.load().unwrap(), settings);
    }

    #[test]
    fn partial_file_fills_defaults_and_clamps() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");
        std::fs::write(&path, "zoom_percent = 300\n[watcher]\npolling = true\n").unwrap();
        let settings = TomlSettingsStore::new(path).load().unwrap();
        assert_eq!(settings.zoom_percent, 150);
        assert!(settings.watcher.polling);
        assert_eq!(settings.watcher.debounce_ms, 100);
        assert!(settings.safe_writes);
        assert_eq!(settings.statuses, StatusSet::default());
    }

    #[test]
    fn reads_custom_statuses() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");
        std::fs::write(
            &path,
            "[statuses]\norder = [\"doing\", \"in-review\", \"todo\"]\nhidden = [\"someday\", \"delegated\"]\n",
        )
        .unwrap();
        let store = TomlSettingsStore::new(path);
        let settings = store.load().unwrap();
        assert_eq!(
            settings.statuses.names(),
            ["doing", "in-review", "todo", "waiting", "someday"]
        );
        assert!(settings.statuses.is_hidden("delegated"));
        store.save(&settings).unwrap();
        assert_eq!(store.load().unwrap(), settings);
    }

    #[test]
    fn statuses_mix_plain_keys_and_labels() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");
        std::fs::write(
            &path,
            "[statuses]
order = [
  \"doing\",
  { key = \"in-review\", label = \"In review\" },
  \"todo\",
]
",
        )
        .unwrap();
        let store = TomlSettingsStore::new(path.clone());
        let settings = store.load().unwrap();
        assert_eq!(
            settings.statuses.names(),
            ["doing", "in-review", "todo", "waiting", "someday"]
        );
        assert_eq!(settings.statuses.label("in-review"), Some("In review"));
        assert_eq!(settings.statuses.label("doing"), None);
        store.save(&settings).unwrap();
        let written = std::fs::read_to_string(&path).unwrap();
        assert!(
            written.contains("\"doing\""),
            "plain keys stay plain:
{written}"
        );
        assert_eq!(store.load().unwrap(), settings);
    }

    #[test]
    fn board_columns_round_trip_per_file() {
        let dir = tempfile::tempdir().unwrap();
        let store = TomlSettingsStore::new(dir.path().join("config.toml"));
        let mut settings = Settings::default();
        let file = dir.path().join("todo.txt");
        let mut columns = BoardColumns::from_keys(["doing", "in-review", "done"]);
        columns.set_label("in-review", "In review");
        settings.set_board_columns(&file, columns);
        store.save(&settings).unwrap();
        let loaded = store.load().unwrap();
        assert_eq!(loaded.board_columns(&file).keys(), ["doing", "in-review", "done"]);
        assert_eq!(loaded.board_columns(&file).label("in-review"), Some("In review"));
        assert_eq!(
            loaded.board_columns(&dir.path().join("other.txt")),
            BoardColumns::default()
        );
    }
}
