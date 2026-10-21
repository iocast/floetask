//! Checking for, installing and rolling back releases (Settings → About).

use iced::Task;

use floetask_application::AppError;
use floetask_application::ports::PublishedRelease;
use floetask_application::updates::offered;

use super::UpdateState;
use crate::app::{Floetask, Message};
use crate::settings;

impl Floetask {
    pub(super) fn update_updates(&mut self, message: settings::Message) -> Task<Message> {
        match message {
            settings::Message::CheckForUpdates => self.check_for_updates(true),
            settings::Message::UpdateChecked(manual, result) => self.update_checked(manual, result),
            settings::Message::RollbackChoice(version) => {
                self.rollback_to = Some(version);
                Task::none()
            }
            settings::Message::InstallRelease(version) => {
                self.update = UpdateState::Installing(version.clone());
                let updater = self.services.ports.updater.clone();
                Task::perform(async move { updater.install(&version) }, |result| {
                    Message::Settings(settings::Message::UpdateInstalled(result))
                })
            }
            settings::Message::UpdateInstalled(result) => {
                // On success the installer takes over and the app usually has
                // exited already; on macOS and Linux it is replaced in place.
                self.update = match result {
                    Ok(()) => UpdateState::Idle,
                    Err(error) => UpdateState::Failed(crate::i18n::error_text(&error)),
                };
                Task::none()
            }
            _ => Task::none(),
        }
    }

    /// Lists the published releases. The automatic check on start stays
    /// silent; an update it finds shows as a button in the title bar.
    pub(crate) fn check_for_updates(&mut self, manual: bool) -> Task<Message> {
        let updater = self.services.ports.updater.clone();
        if !updater.is_configured() {
            return Task::none();
        }
        if manual {
            self.update = UpdateState::Checking;
        }
        Task::perform(async move { updater.releases() }, move |result| {
            Message::Settings(settings::Message::UpdateChecked(manual, result))
        })
    }

    fn update_checked(&mut self, manual: bool, result: Result<Vec<PublishedRelease>, AppError>) -> Task<Message> {
        let releases = match result {
            Ok(releases) => releases,
            Err(error) => {
                // A failed check on start is not worth interrupting for.
                if manual {
                    self.update = UpdateState::Failed(crate::i18n::error_text(&error));
                }
                return Task::none();
            }
        };
        self.published = releases;
        if self
            .rollback_to
            .as_ref()
            .is_some_and(|version| !self.published.iter().any(|release| &release.version == version))
        {
            self.rollback_to = None;
        }
        match offered(&self.published, self.settings.update_channel).cloned() {
            // The title bar shows an update button while one is available.
            Some(release) => self.update = UpdateState::Available(release),
            None => {
                if manual {
                    self.update = UpdateState::UpToDate;
                }
            }
        }
        Task::none()
    }
}
