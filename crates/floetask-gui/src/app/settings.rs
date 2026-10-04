//! The settings dialog and view toggles that change settings.

use iced::Task;

use floetask_application::ThemePreference;

use super::{Dialog, Floetask, Message, SettingChange};

impl Floetask {
    pub(super) fn update_settings(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::OpenSettings => {
                self.dialog = Some(Dialog::Settings);
                Task::none()
            }
            Message::Setting(change) => {
                self.apply_setting(change);
                self.save_settings()
            }
            Message::ToggleTheme => {
                let dark = self.colors().dark;
                self.settings.theme = if dark {
                    ThemePreference::Light
                } else {
                    ThemePreference::Dark
                };
                self.save_settings()
            }
            Message::ToggleTabs => {
                self.state.tabs_visible = !self.state.tabs_visible;
                self.persist_state()
            }
            _ => Task::none(),
        }
    }

    fn apply_setting(&mut self, change: SettingChange) {
        let s = &mut self.settings;
        let mut affects_list = false;
        match change {
            SettingChange::AppendCreationDate(on) => s.append_creation_date = on,
            SettingChange::ConvertRelativeDates(on) => s.convert_relative_dates = on,
            SettingChange::HumanFriendlyDates(on) => {
                s.human_friendly_dates = on;
                affects_list = true;
            }
            SettingChange::SafeWrites(on) => s.safe_writes = on,
            SettingChange::BulkCreation(on) => s.bulk_creation = on,
            SettingChange::DisableAnimations(on) => s.disable_animations = on,
            SettingChange::Compact(on) => s.compact = on,
            SettingChange::Notifications(on) => s.notifications = on,
            SettingChange::NotificationThreshold(days) => s.notification_threshold_days = days,
            SettingChange::Zoom(percent) => s.zoom_percent = percent,
            SettingChange::Theme(theme) => s.theme = theme,
            SettingChange::WeekStart(week_start) => {
                s.week_start = week_start;
                affects_list = true;
            }
            SettingChange::Language(language) => s.language = language,
        }
        self.settings = std::mem::take(&mut self.settings).normalized();
        if affects_list {
            self.refresh();
        }
    }

    fn save_settings(&self) -> Task<Message> {
        let store = self.services.ports.settings.clone();
        let settings = self.settings.clone();
        Task::perform(async move { store.save(&settings) }, Message::from_result)
    }
}
