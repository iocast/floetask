//! Checking for and installing new releases (Settings → About).

use iced::Task;

use floetask_application::AppError;
use floetask_application::ports::AvailableUpdate;

use super::{Floetask, Message};
use crate::i18n::trf;

/// Where the update check stands, shown on the About page.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum UpdateState {
    #[default]
    Idle,
    Checking,
    UpToDate,
    Available(AvailableUpdate),
    Installing,
    Failed(String),
}

impl Floetask {
    pub(super) fn update_updates(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::CheckForUpdates => self.check_for_updates(true),
            Message::UpdateChecked(manual, result) => self.update_checked(manual, result),
            Message::InstallUpdate => {
                self.update = UpdateState::Installing;
                let updater = self.services.ports.updater.clone();
                Task::perform(async move { updater.install_latest() }, Message::UpdateInstalled)
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

    /// Asks for a newer release. The automatic check on start stays silent
    /// unless it finds one.
    pub(super) fn check_for_updates(&mut self, manual: bool) -> Task<Message> {
        let updater = self.services.ports.updater.clone();
        if !updater.is_configured() {
            return Task::none();
        }
        if manual {
            self.update = UpdateState::Checking;
        }
        Task::perform(async move { updater.check() }, move |result| {
            Message::UpdateChecked(manual, result)
        })
    }

    fn update_checked(&mut self, manual: bool, result: Result<Option<AvailableUpdate>, AppError>) -> Task<Message> {
        match result {
            Ok(Some(update)) => {
                // A manual check shows the result on the About page itself.
                let notice = if manual {
                    Task::none()
                } else {
                    self.toast(trf("update_available_toast", &[&update.version]))
                };
                self.update = UpdateState::Available(update);
                notice
            }
            Ok(None) if manual => {
                self.update = UpdateState::UpToDate;
                Task::none()
            }
            Err(error) if manual => {
                self.update = UpdateState::Failed(error.to_string());
                Task::none()
            }
            // A failed or empty check on start is not worth interrupting for.
            _ => Task::none(),
        }
    }
}
