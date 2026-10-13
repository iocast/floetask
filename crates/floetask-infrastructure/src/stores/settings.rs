//! `config.toml`: user settings.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use floetask_application::ports::SettingsStore;
use floetask_application::{AppError, MonthOverflow, Settings, ThemePreference, UpdateChannel, WatcherOptions};
use floetask_domain::board::BoardColumns;
use floetask_domain::{StatusSet, WeekStart};

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
        Ok(file.map(Settings::from).unwrap_or_default().normalized())
    }

    fn save(&self, settings: &Settings) -> Result<(), AppError> {
        write_toml(&self.path, &SettingsFile::from(settings))
    }

    fn location(&self) -> PathBuf {
        self.path.clone()
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(default)]
struct SettingsFile {
    schema_version: u32,
    append_creation_date: bool,
    convert_relative_dates: bool,
    human_friendly_dates: bool,
    safe_writes: bool,
    bulk_creation: bool,
    disable_animations: bool,
    compact: bool,
    notifications: bool,
    check_for_updates: bool,
    /// `stable` or `preview` (release candidates too).
    update_channel: String,
    notification_threshold_days: u8,
    zoom_percent: u16,
    /// `system`, `light` or `dark`.
    theme: String,
    /// `monday`, `saturday` or `sunday`.
    week_start: String,
    /// `more` or `scroll`: what a full calendar month cell does.
    month_overflow: String,
    language: String,
    exclude_lines_with_prefix: Vec<String>,
    watcher: WatcherFile,
    statuses: StatusesFile,
    /// `[[boards]]`: board columns of one todo file each.
    boards: Vec<BoardFile>,
}

#[derive(Debug, Serialize, Deserialize)]
struct BoardFile {
    file: PathBuf,
    /// Column keys, with a display name for statuses only this board knows.
    columns: Vec<StatusEntry>,
}

impl BoardFile {
    fn new(file: &Path, columns: &BoardColumns) -> Self {
        Self {
            file: file.to_path_buf(),
            columns: columns
                .keys()
                .into_iter()
                .map(|key| StatusEntry::new(key.clone(), columns.label(&key)))
                .collect(),
        }
    }

    fn columns(&self) -> BoardColumns {
        let mut columns = BoardColumns::from_keys(self.columns.iter().map(StatusEntry::key));
        for entry in &self.columns {
            if let StatusEntry::Labelled { key, label } = entry {
                columns.set_label(key, label);
            }
        }
        columns
    }
}

/// `[statuses]`: the user's workflow statuses for the `status:` extension.
#[derive(Debug, Serialize, Deserialize)]
#[serde(default)]
struct StatusesFile {
    /// Display and sort order; custom statuses go here too.
    order: Vec<StatusEntry>,
    /// Statuses left out of the default list, like `someday`.
    hidden: Vec<String>,
}

/// One entry of `order`: a plain key such as `"doing"`, or a table with a
/// display name, `{ key = "in-review", label = "In review" }`. Plain keys
/// keep older files valid and leave built-in statuses translated.
#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
enum StatusEntry {
    Key(String),
    Labelled { key: String, label: String },
}

impl StatusEntry {
    fn new(key: String, label: Option<&str>) -> Self {
        match label {
            Some(label) => Self::Labelled {
                key,
                label: label.to_owned(),
            },
            None => Self::Key(key),
        }
    }

    fn key(&self) -> &str {
        match self {
            Self::Key(key) | Self::Labelled { key, .. } => key,
        }
    }
}

impl From<StatusesFile> for StatusSet {
    fn from(file: StatusesFile) -> Self {
        let keys = file.order.iter().map(|entry| entry.key().to_owned()).collect();
        let mut set = StatusSet::new(keys, file.hidden);
        for entry in &file.order {
            if let StatusEntry::Labelled { key, label } = entry {
                set.set_label(key, label);
            }
        }
        set
    }
}

impl Default for StatusesFile {
    fn default() -> Self {
        Self::from(&StatusSet::default())
    }
}

impl From<&StatusSet> for StatusesFile {
    fn from(statuses: &StatusSet) -> Self {
        Self {
            order: statuses
                .names()
                .iter()
                .map(|key| StatusEntry::new(key.clone(), statuses.label(key)))
                .collect(),
            hidden: statuses.hidden().to_vec(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(default)]
struct WatcherFile {
    debounce_ms: u64,
    polling: bool,
    poll_interval_ms: u64,
}

impl Default for SettingsFile {
    fn default() -> Self {
        Self::from(&Settings::default())
    }
}

impl Default for WatcherFile {
    fn default() -> Self {
        let defaults = WatcherOptions::default();
        Self {
            debounce_ms: defaults.debounce_ms,
            polling: defaults.polling,
            poll_interval_ms: defaults.poll_interval_ms,
        }
    }
}

impl From<&Settings> for SettingsFile {
    fn from(s: &Settings) -> Self {
        Self {
            schema_version: SCHEMA_VERSION,
            append_creation_date: s.append_creation_date,
            convert_relative_dates: s.convert_relative_dates,
            human_friendly_dates: s.human_friendly_dates,
            safe_writes: s.safe_writes,
            bulk_creation: s.bulk_creation,
            disable_animations: s.disable_animations,
            compact: s.compact,
            notifications: s.notifications,
            check_for_updates: s.check_for_updates,
            update_channel: match s.update_channel {
                UpdateChannel::Stable => "stable",
                UpdateChannel::Preview => "preview",
            }
            .to_owned(),
            notification_threshold_days: s.notification_threshold_days,
            zoom_percent: s.zoom_percent,
            theme: match s.theme {
                ThemePreference::System => "system",
                ThemePreference::Light => "light",
                ThemePreference::Dark => "dark",
            }
            .to_owned(),
            week_start: match s.week_start {
                WeekStart::Monday => "monday",
                WeekStart::Saturday => "saturday",
                WeekStart::Sunday => "sunday",
            }
            .to_owned(),
            month_overflow: match s.month_overflow {
                MonthOverflow::More => "more",
                MonthOverflow::Scroll => "scroll",
            }
            .to_owned(),
            language: s.language.clone(),
            exclude_lines_with_prefix: s.exclude_lines_with_prefix.clone(),
            watcher: WatcherFile {
                debounce_ms: s.watcher.debounce_ms,
                polling: s.watcher.polling,
                poll_interval_ms: s.watcher.poll_interval_ms,
            },
            statuses: StatusesFile::from(&s.statuses),
            boards: s
                .boards
                .iter()
                .map(|(file, columns)| BoardFile::new(file, columns))
                .collect(),
        }
    }
}

impl From<SettingsFile> for Settings {
    fn from(f: SettingsFile) -> Self {
        Self {
            append_creation_date: f.append_creation_date,
            convert_relative_dates: f.convert_relative_dates,
            human_friendly_dates: f.human_friendly_dates,
            safe_writes: f.safe_writes,
            bulk_creation: f.bulk_creation,
            disable_animations: f.disable_animations,
            compact: f.compact,
            notifications: f.notifications,
            check_for_updates: f.check_for_updates,
            update_channel: match f.update_channel.as_str() {
                "preview" => UpdateChannel::Preview,
                _ => UpdateChannel::Stable,
            },
            notification_threshold_days: f.notification_threshold_days,
            zoom_percent: f.zoom_percent,
            theme: match f.theme.as_str() {
                "light" => ThemePreference::Light,
                "dark" => ThemePreference::Dark,
                _ => ThemePreference::System,
            },
            week_start: match f.week_start.as_str() {
                "saturday" => WeekStart::Saturday,
                "sunday" => WeekStart::Sunday,
                _ => WeekStart::Monday,
            },
            month_overflow: match f.month_overflow.as_str() {
                "scroll" => MonthOverflow::Scroll,
                _ => MonthOverflow::More,
            },
            language: f.language,
            exclude_lines_with_prefix: f.exclude_lines_with_prefix,
            watcher: WatcherOptions {
                debounce_ms: f.watcher.debounce_ms,
                polling: f.watcher.polling,
                poll_interval_ms: f.watcher.poll_interval_ms,
            },
            statuses: StatusSet::from(f.statuses),
            boards: f
                .boards
                .into_iter()
                .map(|board| {
                    let columns = board.columns();
                    (board.file, columns)
                })
                .collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
