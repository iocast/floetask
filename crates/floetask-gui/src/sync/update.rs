//! Sharing, joining and syncing linked files, and settling conflicts.

use std::path::{Path, PathBuf};

use iced::Task;

use floetask_application::{AppError, NoteService, ShareOutcome, StoreLocation, SyncLink, SyncReport, SyncStatus};

use super::{ConflictsDialog, JoinDialog, ShareDialog};
use crate::app::{Dialog, Floetask, Message};
use crate::i18n::{self, tr, trf};
use crate::sync;

impl Floetask {
    pub(crate) fn update_sync(&mut self, message: sync::Message) -> Task<Message> {
        match message {
            sync::Message::Share(index) => {
                self.file_menu = None;
                if let Some(entry) = self.state.files.get(index) {
                    self.dialog = Some(Dialog::Share(ShareDialog::new(entry.path.clone())));
                }
                Task::none()
            }
            sync::Message::ShareChooseFolder => Task::perform(pick_folder(), |folder| {
                Message::Sync(sync::Message::ShareFolder(folder))
            }),
            sync::Message::ShareChooseFile => {
                Task::perform(pick_shared_file(), |file| Message::Sync(sync::Message::ShareFile(file)))
            }
            sync::Message::ShareFolder(Some(folder)) => self.edit_share(|dialog| {
                dialog.name = dialog.suggested_name();
                dialog.folder = Some(folder);
            }),
            sync::Message::ShareFile(Some(file)) => {
                let (Some(folder), Some(_)) = (file.parent(), file.file_name()) else {
                    return Task::none();
                };
                let (folder, name) = (folder.to_path_buf(), file_name(&file));
                self.edit_share(|dialog| {
                    dialog.folder = Some(folder);
                    dialog.name = name;
                })
            }
            sync::Message::ShareName(name) => self.edit_share(|dialog| dialog.name = name),
            sync::Message::ShareExists(folder, name, exists) => {
                if let Some(Dialog::Share(dialog)) = &mut self.dialog
                    && dialog.folder.as_ref() == Some(&folder)
                    && dialog.name.trim() == name
                {
                    dialog.exists = exists;
                }
                Task::none()
            }
            sync::Message::ShareFolder(None)
            | sync::Message::ShareFile(None)
            | sync::Message::JoinShared(None)
            | sync::Message::JoinLocal(None) => Task::none(),
            sync::Message::Shared(dialog, result) => self.shared(dialog, result),
            sync::Message::Join => {
                self.file_menu = None;
                self.dialog = Some(Dialog::Join(JoinDialog::default()));
                Task::none()
            }
            sync::Message::JoinChooseShared => Task::perform(pick_shared_copy(), |shared| {
                Message::Sync(sync::Message::JoinShared(shared))
            }),
            sync::Message::JoinShared(Some(shared)) => self.join_shared(shared),
            sync::Message::JoinChooseNew => {
                let Some(Dialog::Join(dialog)) = &self.dialog else {
                    return Task::none();
                };
                let name = dialog.shared.as_deref().map(file_name).unwrap_or_default();
                let folder = dialog.local.as_deref().and_then(Path::parent).map(Path::to_path_buf);
                Task::perform(pick_local_copy(name, folder), |local| {
                    Message::Sync(sync::Message::JoinLocal(local))
                })
            }
            sync::Message::JoinChooseExisting => Task::perform(pick_existing_local(), |local| {
                Message::Sync(sync::Message::JoinLocal(local))
            }),
            sync::Message::JoinLocal(Some(local)) => self.join_local(local, false),
            sync::Message::JoinLocalExists(local, exists) => {
                if let Some(Dialog::Join(dialog)) = &mut self.dialog
                    && dialog.local.as_ref() == Some(&local)
                {
                    dialog.local_exists = Some(exists);
                }
                Task::none()
            }
            sync::Message::JoinBack => {
                if let Some(Dialog::Join(dialog)) = &mut self.dialog
                    && let Some(step) = dialog.step.previous()
                {
                    dialog.step = step;
                }
                Task::none()
            }
            sync::Message::Joined(link, result) => match result {
                Ok(report) => {
                    let local = link.local.clone();
                    self.sync.set_link(link);
                    let open = self.open_path(local.clone());
                    let toast = self.toast(trf("sync_joined", &[("file", file_name(&local).into())]));
                    let status = self.synced(local, Ok(report));
                    Task::batch([open, toast, status])
                }
                Err(error) => self.report(&error),
            },
            sync::Message::ShareAgain(index) => {
                self.file_menu = None;
                let Some(link) = self.link_at(index) else {
                    return Task::none();
                };
                let StoreLocation::Folder { path } = &link.store;
                self.share(ShareDialog {
                    file: link.local.clone(),
                    folder: Some(path.clone()),
                    name: link.name.clone(),
                    exists: false,
                })
            }
            sync::Message::SyncNow(index) => {
                self.file_menu = None;
                match self.state.files.get(index).map(|entry| entry.path.clone()) {
                    Some(path) => self.request_sync(&path),
                    None => Task::none(),
                }
            }
            sync::Message::Interval => self.sync_all(),
            sync::Message::Synced(path, result) => self.synced(path, result),
            sync::Message::StopSharing(index) => {
                self.file_menu = None;
                let Some(link) = self.link_at(index) else {
                    return Task::none();
                };
                let service = self.services.sync.clone();
                let local = link.local.clone();
                Task::perform(
                    {
                        let local = local.clone();
                        async move { service.unlink(&local) }
                    },
                    move |result| Message::Sync(sync::Message::Unlinked(local.clone(), result)),
                )
            }
            sync::Message::Unlinked(local, result) => match result {
                Ok(()) => {
                    self.sync.links.retain(|link| link.local != local);
                    self.sync.files.remove(&local);
                    self.watch_generation += 1;
                    self.toast(trf("sync_stopped", &[("file", file_name(&local).into())]))
                }
                Err(error) => self.report(&error),
            },
            sync::Message::ShowConflicts(local) => {
                let service = self.services.sync.clone();
                Task::perform(
                    {
                        let local = local.clone();
                        async move { service.conflicts(&local) }
                    },
                    move |result| Message::Sync(sync::Message::ConflictsLoaded(local.clone(), result)),
                )
            }
            sync::Message::ConflictsLoaded(local, result) => match result {
                Ok(items) if items.is_empty() => {
                    let toast = self.toast(tr("sync_no_conflicts"));
                    Task::batch([toast, self.request_sync(&local)])
                }
                Ok(items) => {
                    self.file_menu = None;
                    self.dialog = Some(Dialog::Conflicts(ConflictsDialog::new(local, items)));
                    Task::none()
                }
                Err(error) => self.report(&error),
            },
            sync::Message::Choose(item, conflict, choice) => {
                if let Some(Dialog::Conflicts(dialog)) = &mut self.dialog
                    && let Some(slot) = dialog
                        .choices
                        .get_mut(item)
                        .and_then(|choices| choices.get_mut(conflict))
                {
                    *slot = choice;
                }
                Task::none()
            }
            sync::Message::Resolved(local, result) => match result {
                Err(AppError::Changed(_)) => {
                    let toast = self.toast_error(tr("sync_changed_meanwhile"));
                    Task::batch([toast, self.request_sync(&local)])
                }
                result => self.synced(local, result),
            },
        }
    }

    /// Changes the share dialog, then looks up whether its shared copy
    /// exists, which turns sharing into joining.
    fn edit_share(&mut self, change: impl FnOnce(&mut ShareDialog)) -> Task<Message> {
        let Some(Dialog::Share(dialog)) = &mut self.dialog else {
            return Task::none();
        };
        change(dialog);
        dialog.exists = false;
        let Some(folder) = dialog.folder.clone().filter(|_| dialog.is_ready()) else {
            return Task::none();
        };
        let name = dialog.name.trim().to_owned();
        let service = self.services.sync.clone();
        Task::perform(
            {
                let (folder, name) = (folder.clone(), name.clone());
                async move { service.exists(&StoreLocation::Folder { path: folder }, &name) }
            },
            move |exists| {
                Message::Sync(sync::Message::ShareExists(
                    folder.clone(),
                    name.clone(),
                    exists.unwrap_or(false),
                ))
            },
        )
    }

    /// Takes the picked shared copy and, unless the user chose their own
    /// copy already, suggests one: the shared copy's name next to the
    /// active file.
    fn join_shared(&mut self, shared: PathBuf) -> Task<Message> {
        let suggestion = self
            .active_path()
            .and_then(|active| Some(active.parent()?.join(shared.file_name()?)))
            .filter(|suggestion| *suggestion != shared);
        let Some(Dialog::Join(dialog)) = &mut self.dialog else {
            return Task::none();
        };
        dialog.shared = Some(shared);
        if dialog.local.is_some() && !dialog.suggested {
            return Task::none();
        }
        match suggestion {
            Some(local) => self.join_local(local, true),
            None => {
                dialog.local = None;
                dialog.local_exists = None;
                Task::none()
            }
        }
    }

    /// Sets the join wizard's local copy, then looks up whether it exists,
    /// which decides whether joining creates it or merges it.
    fn join_local(&mut self, local: PathBuf, suggested: bool) -> Task<Message> {
        let Some(Dialog::Join(dialog)) = &mut self.dialog else {
            return Task::none();
        };
        dialog.local = Some(local.clone());
        dialog.suggested = suggested;
        dialog.local_exists = None;
        let files = self.services.ports.files.clone();
        Task::perform(
            {
                let local = local.clone();
                async move { files.exists(&local) }
            },
            move |exists| Message::Sync(sync::Message::JoinLocalExists(local.clone(), exists)),
        )
    }

    /// Goes on to the join wizard's next step, or joins on the last one.
    pub(crate) fn join_next(&mut self, mut dialog: JoinDialog) -> Task<Message> {
        if !dialog.can_continue() {
            self.dialog = Some(Dialog::Join(dialog));
            return Task::none();
        }
        match (dialog.step.next(), dialog.link()) {
            (Some(step), _) => {
                dialog.step = step;
                self.dialog = Some(Dialog::Join(dialog));
                Task::none()
            }
            (None, Some(link)) => self.join(link),
            (None, None) => Task::none(),
        }
    }

    /// Shares the file of the confirmed share dialog, or joins its shared
    /// copy when that exists already.
    pub(crate) fn share(&mut self, dialog: ShareDialog) -> Task<Message> {
        let Some(folder) = dialog.folder.clone().filter(|_| dialog.is_ready()) else {
            self.dialog = Some(Dialog::Share(dialog));
            return Task::none();
        };
        let store = StoreLocation::Folder { path: folder };
        if dialog.exists {
            return self.join(SyncLink {
                local: dialog.file,
                store,
                name: dialog.name.trim().to_owned(),
            });
        }
        let service = self.services.sync.clone();
        let (file, name) = (dialog.file.clone(), dialog.name.trim().to_owned());
        Task::perform(async move { service.share(&file, store, &name) }, move |result| {
            Message::Sync(sync::Message::Shared(dialog.clone(), result))
        })
    }

    /// Applies the conflict dialog's choices.
    pub(crate) fn resolve_conflicts(&mut self, dialog: ConflictsDialog) -> Task<Message> {
        let service = self.services.sync.clone();
        let skip = self.notes_in_editors();
        let choices: Vec<_> = dialog
            .items
            .iter()
            .map(|item| item.item.clone())
            .zip(dialog.choices)
            .collect();
        let local = dialog.file;
        Task::perform(
            {
                let local = local.clone();
                async move { service.resolve(&local, &choices, &skip) }
            },
            move |result| Message::Sync(sync::Message::Resolved(local.clone(), result)),
        )
    }

    /// Syncs a linked file in the background. A sync asked for while one
    /// runs starts again when that one finished; files that are not linked
    /// are left alone.
    pub(crate) fn request_sync(&mut self, path: &Path) -> Task<Message> {
        if self.sync.link(path).is_none() {
            return Task::none();
        }
        let entry = self.sync.files.entry(path.to_path_buf()).or_default();
        if entry.running {
            entry.again = true;
            return Task::none();
        }
        entry.running = true;
        let service = self.services.sync.clone();
        let skip = self.notes_in_editors();
        let path = path.to_path_buf();
        Task::perform(
            {
                let path = path.clone();
                async move { service.sync(&path, &skip) }
            },
            move |result| Message::Sync(sync::Message::Synced(path.clone(), result)),
        )
    }

    /// Syncs every linked file: on start and every `sync.interval`.
    pub(crate) fn sync_all(&mut self) -> Task<Message> {
        let paths: Vec<PathBuf> = self.sync.links.iter().map(|link| link.local.clone()).collect();
        Task::batch(paths.iter().map(|path| self.request_sync(path)).collect::<Vec<_>>())
    }

    /// Syncs the linked files whose shared copies are in `changed`, a
    /// watched folder.
    pub(crate) fn sync_folder(&mut self, changed: &Path) -> Task<Message> {
        let service = &self.services.sync;
        let paths: Vec<PathBuf> = self
            .sync
            .links
            .iter()
            .filter(|link| {
                service
                    .watch_paths(std::slice::from_ref(link))
                    .iter()
                    .any(|path| path == changed)
            })
            .map(|link| link.local.clone())
            .collect();
        Task::batch(paths.iter().map(|path| self.request_sync(path)).collect::<Vec<_>>())
    }

    /// The notes open in the note window or the edit dialog: a sync leaves
    /// them alone until they are closed, so an editor never writes over a
    /// merge.
    fn notes_in_editors(&self) -> Vec<PathBuf> {
        let window = self.note_window.as_ref().map(|window| window.path());
        let dialog = match &self.dialog {
            Some(Dialog::Editor(editor)) => match editor.note_target() {
                Some(Ok(link)) => Some(NoteService::link_path(&editor.file, &link)),
                _ => None,
            },
            _ => None,
        };
        window.into_iter().chain(dialog).collect()
    }

    fn link_at(&self, index: usize) -> Option<SyncLink> {
        let entry = self.state.files.get(index)?;
        self.sync.link(&entry.path).cloned()
    }

    fn join(&mut self, link: SyncLink) -> Task<Message> {
        let service = self.services.sync.clone();
        Task::perform(
            {
                let link = link.clone();
                async move { service.join(&link.local, link.store.clone(), &link.name) }
            },
            move |result| Message::Sync(sync::Message::Joined(link.clone(), result)),
        )
    }

    fn shared(&mut self, dialog: ShareDialog, result: Result<ShareOutcome, AppError>) -> Task<Message> {
        match result {
            Ok(ShareOutcome::Shared) => {
                let local = dialog.file.clone();
                let Some(folder) = dialog.folder.clone() else {
                    return Task::none();
                };
                self.sync.set_link(SyncLink {
                    local: local.clone(),
                    store: StoreLocation::Folder { path: folder },
                    name: dialog.name.trim().to_owned(),
                });
                self.sync.files.remove(&local);
                self.watch_generation += 1;
                let toast = self.toast(trf("sync_shared", &[("file", file_name(&local).into())]));
                Task::batch([toast, self.request_sync(&local)])
            }
            Ok(ShareOutcome::Exists) => {
                self.dialog = Some(Dialog::Share(ShareDialog { exists: true, ..dialog }));
                Task::none()
            }
            Err(error) => self.report(&error),
        }
    }

    /// Shows how a sync went; reloads the file when the sync changed it.
    fn synced(&mut self, path: PathBuf, result: Result<SyncReport, AppError>) -> Task<Message> {
        let entry = self.sync.files.entry(path.clone()).or_default();
        entry.running = false;
        let again = std::mem::take(&mut entry.again);
        let previous = entry.status.clone();
        let mut tasks = Vec::new();
        match result {
            Ok(report) => {
                entry.status = Some(report.status.clone());
                let name = file_name(&path);
                if report.local_changed {
                    tasks.push(self.load(path.clone()));
                }
                let newly = |was: &Option<SyncStatus>, is: fn(&SyncStatus) -> bool| !was.as_ref().is_some_and(is);
                match &report.status {
                    SyncStatus::Conflict(count) if newly(&previous, |s| matches!(s, SyncStatus::Conflict(_))) => {
                        let text = trf("sync_conflicts", &[("file", name.into()), ("count", (*count).into())]);
                        tasks.push(self.toast_error(text));
                    }
                    SyncStatus::SharedMissing if newly(&previous, |s| *s == SyncStatus::SharedMissing) => {
                        tasks.push(self.toast_error(trf("sync_missing_toast", &[("file", name.into())])));
                    }
                    _ => {}
                }
                if !report.copies_left.is_empty() {
                    let names = report.copies_left.join(", ");
                    tasks.push(self.toast_error(trf("sync_copies_left", &[("names", names.into())])));
                }
            }
            Err(error) => {
                let text = format!(
                    "{}: {}",
                    trf("sync_failed", &[("file", file_name(&path).into())]),
                    i18n::error_text(&error)
                );
                tasks.push(self.toast_error(text));
            }
        }
        if again {
            tasks.push(self.request_sync(&path));
        }
        Task::batch(tasks)
    }
}

fn file_name(path: &Path) -> String {
    path.file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_default()
}

async fn pick_folder() -> Option<PathBuf> {
    rfd::AsyncFileDialog::new()
        .set_title(tr("share_pick_folder"))
        .pick_folder()
        .await
        .map(|handle| handle.path().to_path_buf())
}

async fn pick_shared_file() -> Option<PathBuf> {
    rfd::AsyncFileDialog::new()
        .set_title(tr("share_pick_file"))
        .add_filter(tr("file_dialog_title"), &["txt"])
        .add_filter("*", &["*"])
        .pick_file()
        .await
        .map(|handle| handle.path().to_path_buf())
}

async fn pick_shared_copy() -> Option<PathBuf> {
    rfd::AsyncFileDialog::new()
        .set_title(tr("join_pick_shared"))
        .add_filter(tr("file_dialog_title"), &["txt"])
        .add_filter("*", &["*"])
        .pick_file()
        .await
        .map(|handle| handle.path().to_path_buf())
}

async fn pick_local_copy(name: String, folder: Option<PathBuf>) -> Option<PathBuf> {
    let mut picker = rfd::AsyncFileDialog::new()
        .set_title(tr("join_pick_local"))
        .set_file_name(name)
        .add_filter(tr("file_dialog_title"), &["txt"]);
    if let Some(folder) = folder {
        picker = picker.set_directory(folder);
    }
    picker.save_file().await.map(|handle| handle.path().to_path_buf())
}

async fn pick_existing_local() -> Option<PathBuf> {
    rfd::AsyncFileDialog::new()
        .set_title(tr("join_pick_existing"))
        .add_filter(tr("file_dialog_title"), &["txt"])
        .add_filter("*", &["*"])
        .pick_file()
        .await
        .map(|handle| handle.path().to_path_buf())
}
