//! Checking for, installing and rolling back releases (Settings → About).

use iced::Task;

use floetask_application::AppError;
use floetask_application::ports::PublishedRelease;
use floetask_application::updates::offered;

use super::{Floetask, Message};
use crate::i18n::trf;

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

impl Floetask {
    pub(super) fn update_updates(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::CheckForUpdates => self.check_for_updates(true),
            Message::UpdateChecked(manual, result) => self.update_checked(manual, result),
            Message::RollbackChoice(version) => {
                self.rollback_to = Some(version);
                Task::none()
            }
            Message::InstallRelease(version) => {
                self.update = UpdateState::Installing(version.clone());
                let updater = self.services.ports.updater.clone();
                Task::perform(async move { updater.install(&version) }, Message::UpdateInstalled)
            }
            Message::UpdateInstalled(result) => {
                // On success the installer takes over and the app usually has
                // exited already; on macOS and Linux it is replaced in place.
                self.update = match result {
                    Ok(()) => UpdateState::Idle,
                    Err(error) => UpdateState::Failed(error.to_string()),
                };
                Task::none()
            }
            _ => Task::none(),
        }
    }

    /// Lists the published releases. The automatic check on start stays
    /// silent unless it finds an update.
    pub(super) fn check_for_updates(&mut self, manual: bool) -> Task<Message> {
        let updater = self.services.ports.updater.clone();
        if !updater.is_configured() {
            return Task::none();
        }
        if manual {
            self.update = UpdateState::Checking;
        }
        Task::perform(async move { updater.releases() }, move |result| {
            Message::UpdateChecked(manual, result)
        })
    }

    fn update_checked(&mut self, manual: bool, result: Result<Vec<PublishedRelease>, AppError>) -> Task<Message> {
        let releases = match result {
            Ok(releases) => releases,
            Err(error) => {
                // A failed check on start is not worth interrupting for.
                if manual {
                    self.update = UpdateState::Failed(error.to_string());
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
            Some(release) => {
                // A manual check shows the result on the About page itself.
                let notice = if manual {
                    Task::none()
                } else {
                    self.toast(trf("update_available_toast", &[&release.version]))
                };
                self.update = UpdateState::Available(release);
                notice
            }
            None => {
                if manual {
                    self.update = UpdateState::UpToDate;
                }
                Task::none()
            }
        }
    }
}
