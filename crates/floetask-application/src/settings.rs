//! User settings, stored in the config file.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use floetask_domain::board::{BoardColumns, Column};
use floetask_domain::{StatusSet, WeekStart};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ThemePreference {
    #[default]
    System,
    Light,
    Dark,
}

/// How todo files are watched for external changes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WatcherOptions {
    /// Wait until a file has been stable this long before reloading, so sync
    /// tools do not cause flicker.
    pub debounce_ms: u64,
    /// Poll instead of using OS events (network drives, some sync tools).
    pub polling: bool,
    pub poll_interval_ms: u64,
}

impl Default for WatcherOptions {
    fn default() -> Self {
        Self {
            debounce_ms: 100,
            polling: false,
            poll_interval_ms: 1000,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Settings {
    pub append_creation_date: bool,
    pub convert_relative_dates: bool,
    pub human_friendly_dates: bool,
    pub safe_writes: bool,
    pub bulk_creation: bool,
    pub disable_animations: bool,
    pub compact: bool,
    pub notifications: bool,
    /// Notify about todos due within this many days (0 = today only).
    pub notification_threshold_days: u8,
    pub zoom_percent: u16,
    pub theme: ThemePreference,
    pub week_start: WeekStart,
    /// `system` or a language code such as `en`.
    pub language: String,
    /// Lines starting with one of these are ignored and preserved.
    pub exclude_lines_with_prefix: Vec<String>,
    pub watcher: WatcherOptions,
    /// Workflow statuses for `status:`: display order and hidden ones.
    pub statuses: StatusSet,
    /// Board columns per todo file. Files without an entry use the default
    /// columns.
    pub boards: BTreeMap<PathBuf, BoardColumns>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            append_creation_date: false,
            convert_relative_dates: true,
            human_friendly_dates: false,
            safe_writes: true,
            bulk_creation: false,
            disable_animations: false,
            compact: false,
            notifications: true,
            notification_threshold_days: 2,
            zoom_percent: 100,
            theme: ThemePreference::System,
            week_start: WeekStart::Monday,
            language: "system".to_owned(),
            exclude_lines_with_prefix: Vec::new(),
            watcher: WatcherOptions::default(),
            statuses: StatusSet::default(),
            boards: BTreeMap::new(),
        }
    }
}

impl Settings {
    pub const ZOOM_RANGE: std::ops::RangeInclusive<u16> = 50..=150;
    pub const NOTIFICATION_THRESHOLD_MAX: u8 = 10;

    /// The board columns of a todo file.
    pub fn board_columns(&self, file: &Path) -> BoardColumns {
        self.boards.get(file).cloned().unwrap_or_default()
    }

    /// Sets a file's board columns; the default columns remove the entry so
    /// the config file only lists real customisations. A column for a new
    /// status adds that status to the status list, so pickers offer it.
    pub fn set_board_columns(&mut self, file: &Path, columns: BoardColumns) {
        self.register_column_statuses(&columns);
        if columns == BoardColumns::default() {
            self.boards.remove(file);
        } else {
            self.boards.insert(file.to_path_buf(), columns);
        }
    }

    fn register_column_statuses(&mut self, columns: &BoardColumns) {
        for column in columns.columns() {
            if let Column::Status(status) = column {
                self.statuses.add(status);
            }
        }
    }

    /// Clamps values a hand-edited config file may have out of range, and
    /// adds statuses that only appear as board columns to the status list.
    pub fn normalized(mut self) -> Self {
        for columns in self.boards.values().cloned().collect::<Vec<_>>() {
            self.register_column_statuses(&columns);
        }
        self.zoom_percent = self
            .zoom_percent
            .clamp(*Self::ZOOM_RANGE.start(), *Self::ZOOM_RANGE.end());
        self.zoom_percent -= self.zoom_percent % 10;
        self.notification_threshold_days = self.notification_threshold_days.min(Self::NOTIFICATION_THRESHOLD_MAX);
        self
    }
}
