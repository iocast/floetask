//! Opening, watching, switching and archiving todo files.

use std::path::{Path, PathBuf};

use iced::Task;

use floetask_application::{AppError, FileOptions, TodoFileService};
use floetask_domain::TodoDocument;

use super::{Dialog, Floetask, Message, Pending, absolute};
use crate::i18n::{tr, trf};

impl Floetask {
    pub(super) fn update_files(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::FileLoaded(path, result) => self.file_loaded(path, result),
            Message::FileChanged(path) => self.file_changed(path),
            Message::Saved(path, result) => match result {
                Ok(document) => self.document_saved(path, document),
                Err(error) => self.save_failed(path, &error),
            },
            Message::SavedCount(path, result, key) => match result {
                Ok((document, count)) => {
                    let reload = self.document_saved(path, document);
                    Task::batch([reload, self.toast(trf(key, &[&count]))])
                }
                Err(error) => self.save_failed(path, &error),
            },
            Message::OpenFileDialog => Task::perform(pick_file(), Message::FilePicked),
            Message::CreateFileDialog => Task::perform(save_file("todo.txt"), Message::FileCreated),
            Message::FilePicked(Some(path)) => self.open_path(path),
            Message::FileCreated(Some(path)) => self.create_file(path),
            Message::FilePicked(None) | Message::FileCreated(None) => Task::none(),
            Message::SelectFile(index) => self.select_file(index),
            Message::TabMenu(index) => {
                self.tab_menu = index;
                Task::none()
            }
            Message::ChangeDoneFile(index) => {
                self.tab_menu = None;
                Task::perform(save_file("done.txt"), move |path| Message::DoneFilePicked(index, path))
            }
            Message::DoneFilePicked(index, Some(path)) => {
                if let Some(entry) = self.state.files.get_mut(index) {
                    entry.done_path = Some(path);
                }
                self.persist_state()
            }
            Message::DoneFilePicked(_, None) => Task::none(),
            Message::RevealTodoFile(index) => {
                let path = self.state.files.get(index).map(|entry| entry.path.clone());
                self.reveal(path)
            }
            Message::RevealDoneFile(index) => {
                let path = self.state.files.get(index).and_then(|entry| entry.done_path.clone());
                self.reveal(path)
            }
            Message::AskRemoveFile(index) => {
                self.tab_menu = None;
                if let Some(entry) = self.state.files.get(index) {
                    self.dialog = Some(Dialog::Confirm {
                        message: trf("remove_file_confirm", &[&entry.file_name()]),
                        detail: Some(entry.path.display().to_string()),
                        pending: Pending::RemoveFile(index),
                    });
                }
                Task::none()
            }
            Message::ArchiveCompleted => self.archive_completed(),
            Message::SavedFiltersLoaded(result) => match result {
                Ok(filters) => {
                    self.saved_filters = filters;
                    Task::none()
                }
                Err(error) => self.report(&error),
            },
            _ => Task::none(),
        }
    }

    /// Loads every registered file, for the tabs and for notifications.
    pub(super) fn load_all_files(&self) -> Task<Message> {
        Task::batch(self.state.files.iter().map(|entry| self.load(entry.path.clone())))
    }

    fn load(&self, path: PathBuf) -> Task<Message> {
        let service = self.services.todo_files.clone();
        let options = self.file_options();
        Task::perform(
            async move { service.load(&path, &options).map(|doc| (path, doc)) },
            |result| match result {
                Ok((path, document)) => Message::FileLoaded(path, Ok(document)),
                Err(error) => {
                    let path = match &error {
                        AppError::Io { path, .. } => path.clone(),
                        _ => PathBuf::new(),
                    };
                    Message::FileLoaded(path, Err(error))
                }
            },
        )
    }

    fn file_loaded(&mut self, path: PathBuf, result: Result<TodoDocument, AppError>) -> Task<Message> {
        match result {
            Ok(document) => {
                let unchanged = self.documents.get(&path) == Some(&document);
                self.documents.insert(path.clone(), document);
                if !unchanged && self.active_path().as_ref() == Some(&path) {
                    self.refresh();
                }
                Task::none()
            }
            Err(error) => {
                let name = path
                    .file_name()
                    .map(|n| n.to_string_lossy().into_owned())
                    .unwrap_or_default();
                self.toast_error(format!("{}: {error}", trf("file_load_failed", &[&name])))
            }
        }
    }

    fn file_changed(&mut self, path: PathBuf) -> Task<Message> {
        if path == self.services.ports.saved_filters.location() {
            let store = self.services.ports.saved_filters.clone();
            return Task::perform(async move { store.load() }, Message::SavedFiltersLoaded);
        }
        if self.state.files.iter().any(|entry| entry.path == path) {
            self.load(path)
        } else {
            Task::none()
        }
    }

    fn document_saved(&mut self, path: PathBuf, document: TodoDocument) -> Task<Message> {
        self.documents.insert(path.clone(), document);
        if self.active_path().as_ref() == Some(&path) {
            self.refresh();
        }
        Task::none()
    }

    /// A failed change usually means the file changed underneath: show the
    /// error and reload so the user sees the current content.
    fn save_failed(&mut self, path: PathBuf, error: &AppError) -> Task<Message> {
        let report = self.report(error);
        if path.as_os_str().is_empty() {
            report
        } else {
            Task::batch([report, self.load(path)])
        }
    }

    /// Registers a file (if new), makes it active and loads it.
    pub(crate) fn open_path(&mut self, path: PathBuf) -> Task<Message> {
        let path = absolute(&path);
        self.state.open_file(path.clone());
        self.after_file_switch();
        Task::batch([self.load(path), self.persist_state()])
    }

    fn create_file(&mut self, path: PathBuf) -> Task<Message> {
        let service = self.services.todo_files.clone();
        let options = self.file_options();
        let created = path.clone();
        Task::perform(
            async move { service.open_or_create(&created, &options).map(drop) },
            Message::from_result,
        )
        .chain(Task::done(Message::FilePicked(Some(path))))
    }

    fn select_file(&mut self, index: usize) -> Task<Message> {
        if index >= self.state.files.len() {
            return Task::none();
        }
        self.state.active_file = index;
        self.tab_menu = None;
        self.after_file_switch();
        self.persist_state()
    }

    pub(super) fn remove_file(&mut self, index: usize) -> Task<Message> {
        if let Some(entry) = self.state.files.get(index) {
            self.documents.remove(&entry.path.clone());
        }
        self.state.remove_file(index);
        self.after_file_switch();
        self.persist_state()
    }

    fn after_file_switch(&mut self) {
        self.selected = None;
        self.row_menu = None;
        self.watch_generation += 1;
        self.refresh();
    }

    fn archive_completed(&mut self) -> Task<Message> {
        let Some(entry) = self.state.active_entry().cloned() else {
            return Task::none();
        };
        let Some(done_path) = entry.done_path.clone() else {
            let index = self.state.active_file;
            let notice = self.toast(tr("done_file_needed"));
            return Task::batch([notice, Task::done(Message::ChangeDoneFile(index))]);
        };
        let service = self.services.todo_files.clone();
        let options = self.file_options();
        let path = entry.path.clone();
        Task::perform(
            async move { service.archive_completed(&entry.path, &done_path, &options) },
            move |result| Message::SavedCount(path.clone(), result, "archived"),
        )
    }

    fn reveal(&mut self, path: Option<PathBuf>) -> Task<Message> {
        self.tab_menu = None;
        let Some(path) = path else {
            return self.toast(tr("done_file_needed"));
        };
        let desktop = self.services.ports.desktop.clone();
        Task::perform(async move { desktop.reveal(&path) }, Message::from_result)
    }

    /// Runs a change on the active file in the background and shows the
    /// resulting document.
    pub(crate) fn change_active_file(
        &self,
        change: impl FnOnce(&TodoFileService, &Path, &FileOptions) -> Result<TodoDocument, AppError> + Send + 'static,
    ) -> Task<Message> {
        let Some(path) = self.active_path() else {
            return Task::none();
        };
        let service = self.services.todo_files.clone();
        let options = self.file_options();
        let target = path.clone();
        Task::perform(async move { change(&service, &target, &options) }, move |result| {
            Message::Saved(path.clone(), result)
        })
    }
}

async fn pick_file() -> Option<PathBuf> {
    rfd::AsyncFileDialog::new()
        .set_title(tr("open_file"))
        .add_filter(tr("file_dialog_title"), &["txt"])
        .add_filter("*", &["*"])
        .pick_file()
        .await
        .map(|handle| handle.path().to_path_buf())
}

async fn save_file(default_name: &'static str) -> Option<PathBuf> {
    rfd::AsyncFileDialog::new()
        .set_title(tr("create_file"))
        .set_file_name(default_name)
        .add_filter(tr("file_dialog_title"), &["txt"])
        .save_file()
        .await
        .map(|handle| handle.path().to_path_buf())
}
