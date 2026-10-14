//! The settings dialog, the switches elsewhere that change settings, and
//! the release check on its About page.

mod update;
mod updates;
mod view;

use floetask_application::ports::PublishedRelease;
use floetask_application::{AppError, MonthOverflow, ThemePreference, UpdateChannel};
use floetask_domain::WeekStart;

pub(crate) use view::view;

#[derive(Debug, Clone)]
pub enum Message {
    Open,
    Change(SettingChange),
    ToggleTheme,
    Section(SettingsSection),
    StatusInput(String),
    CheckForUpdates,
    /// Whether the user asked, and the published releases.
    UpdateChecked(bool, Result<Vec<PublishedRelease>, AppError>),
    /// Installs this version: the offered update, or an older one (a rollback).
    InstallRelease(String),
    /// The earlier version picked to roll back to.
    RollbackChoice(String),
    UpdateInstalled(Result<(), AppError>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SettingChange {
    AppendCreationDate(bool),
    ConvertRelativeDates(bool),
    HumanFriendlyDates(bool),
    SafeWrites(bool),
    BulkCreation(bool),
    DisableAnimations(bool),
    Compact(bool),
    Notifications(bool),
    CheckForUpdates(bool),
    UpdateChannel(UpdateChannel),
    NotificationThreshold(u8),
    Zoom(u16),
    Theme(ThemePreference),
    WeekStart(WeekStart),
    MonthOverflow(MonthOverflow),
    /// Adds a custom status from the name the user typed, such as
    /// "In review"; the key (`in-review`) is derived from it.
    StatusAdd(String),
    StatusRemove(String),
    StatusMove(usize, isize),
    StatusHidden(String, bool),
    Language(String),
}

/// A page of the settings dialog, chosen in its sidebar.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SettingsSection {
    #[default]
    Todos,
    Dates,
    Statuses,
    Appearance,
    Notifications,
    Files,
    About,
}

impl SettingsSection {
    /// Sidebar order.
    pub const ALL: [SettingsSection; 7] = [
        SettingsSection::Todos,
        SettingsSection::Dates,
        SettingsSection::Statuses,
        SettingsSection::Appearance,
        SettingsSection::Notifications,
        SettingsSection::Files,
        SettingsSection::About,
    ];

    /// Translation key of the section title.
    pub fn key(self) -> &'static str {
        match self {
            SettingsSection::Todos => "section_todos",
            SettingsSection::Dates => "section_dates",
            SettingsSection::Statuses => "section_statuses",
            SettingsSection::Appearance => "section_appearance",
            SettingsSection::Notifications => "section_notifications",
            SettingsSection::Files => "section_files",
            SettingsSection::About => "section_about",
        }
    }
}

/// The settings dialog's own state; the settings live in `Floetask`.
#[derive(Debug, Default)]
pub struct SettingsDialog {
    pub section: SettingsSection,
    /// The display name of a new status being typed on the Statuses page.
    pub status_input: String,
}

/// Where the update check stands, shown on the About page.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum UpdateState {
    #[default]
    Idle,
    Checking,
    UpToDate,
    Available(PublishedRelease),
    /// Downloading and installing this version.
    Installing(String),
    Failed(String),
}
