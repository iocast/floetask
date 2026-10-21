//! Opening, watching, switching and archiving todo files.

use std::path::{Path, PathBuf};

use iced::Task;

use floetask_application::{AppError, FileOptions, TodoFileService};
use floetask_domain::TodoDocument;

use crate::app::{Dialog, Floetask, Message, Pending, absolute};
use crate::i18n::{self, tr, trf};
use crate::{files, search};

impl Floetask {
    pub(crate) fn update_files(&mut self, message: files::Message) -> Task<Message> {
        match message {
            files::Message::Loaded(path, result) => self.file_loaded(path, result),
            files::Message::Changed(path) => self.file_changed(path),
            files::Message::Saved(path, result) => match result {
                Ok(document) => self.document_saved(path, document),
                Err(error) => self.save_failed(path, &error),
            },
            files::Message::SavedCount(path, result, key) => match result {
                Ok((document, count)) => {
                    let reload = self.document_saved(path, document);
                    Task::batch([reload, self.toast(trf(key, &[("count", count.into())]))])
                }
                Err(error) => self.save_failed(path, &error),
            },
            files::Message::OpenDialog => {
                Task::perform(pick_file(), |path| Message::Files(files::Message::Picked(path)))
            }
            files::Message::CreateDialog => Task::perform(save_file("todo.txt"), |path| {
                Message::Files(files::Message::Created(path))
            }),
            files::Message::Picked(Some(path)) => self.open_path(path),
            files::Message::Created(Some(path)) => self.create_file(path),
            files::Message::Picked(None) | files::Message::Created(None) => Task::none(),
            files::Message::Select(index) => self.select_file(index),
            files::Message::SelectOverview => {
                self.state.overview = true;
                self.file_menu = None;
                self.after_file_switch();
                self.persist_state()
            }
            files::Message::ToggleDrawer => {
                self.file_menu = None;
                self.toggle_panel(crate::layout::Panel::Files)
            }
            files::Message::OpenDoneFile(index) => {
                self.file_menu = None;
                match self.state.files.get(index).and_then(|entry| entry.done_path.clone()) {
                    Some(done) => self.open_path(done),
                    None => self.toast(tr("done_file_needed")),
                }
            }
            files::Message::Menu(index) => {
                self.file_menu = index;
                Task::none()
            }
            files::Message::ChangeDoneFile(index) => {
                self.file_menu = None;
                Task::perform(save_file("done.txt"), move |path| {
                    Message::Files(files::Message::DoneFilePicked(index, path))
                })
            }
            files::Message::DoneFilePicked(index, Some(path)) => {
                if let Some(entry) = self.state.files.get_mut(index) {
                    entry.done_path = Some(path);
                }
                self.persist_state()
            }
            files::Message::DoneFilePicked(_, None) => Task::none(),
            files::Message::Reveal(index) => {
                let path = self.state.files.get(index).map(|entry| entry.path.clone());
                self.reveal(path)
            }
            files::Message::AskRemove(index) => {
                self.file_menu = None;
                if let Some(entry) = self.state.files.get(index) {
                    self.dialog = Some(Dialog::Confirm {
                        message: trf("remove_file_confirm", &[("file", entry.file_name().into())]),
                        detail: Some(entry.path.display().to_string()),
                        pending: Pending::RemoveFile(index),
                    });
                }
                Task::none()
            }
            files::Message::ArchiveCompleted => self.archive_completed(),
            files::Message::SortMenu(open) => {
                self.file_sort_menu = open;
                Task::none()
            }
            files::Message::Sort(sort) => {
                self.file_sort_menu = false;
                self.state.file_sort = sort;
                self.persist_state()
            }
            files::Message::Press(index) => {
                self.file_menu = None;
                self.file_drag = Some(files::FileDrag {
                    from: index,
                    moved: false,
                });
                self.file_hover = Some(index);
                Task::none()
            }
            files::Message::Hover(index, entered) => {
                if entered {
                    self.file_hover = Some(index);
                } else if self.file_hover == Some(index) {
                    // Leaving one file and entering the next can arrive in
                    // either order, so only clear our own.
                    self.file_hover = None;
                }
                Task::none()
            }
            files::Message::EntryLeft => {
                if let Some(drag) = &mut self.file_drag {
                    drag.moved = true;
                }
                Task::none()
            }
            files::Message::Release => self.drop_file(),
            files::Message::ListLeft => {
                self.file_drag = None;
                self.file_hover = None;
                Task::none()
            }
        }
    }

    /// Ends a drag. Released on another file, the dragged file takes its
    /// place in a custom order; released on the file itself without leaving
    /// it, it was a click and selects the file.
    fn drop_file(&mut self) -> Task<Message> {
        let Some(drag) = self.file_drag.take() else {
            return Task::none();
        };
        match self.file_hover {
            Some(target) if target != drag.from => {
                self.state.move_file(drag.from, target);
                self.persist_state()
            }
            Some(_) if !drag.moved => self.select_file(drag.from),
            _ => Task::none(),
        }
    }

    /// Loads every registered file, for the tabs and for notifications.
    pub(crate) fn load_all_files(&self) -> Task<Message> {
        Task::batch(self.state.files.iter().map(|entry| self.load(entry.path.clone())))
    }

    fn load(&self, path: PathBuf) -> Task<Message> {
        let service = self.services.todo_files.clone();
        let options = self.file_options();
        Task::perform(
            async move { service.load(&path, &options).map(|doc| (path, doc)) },
            |result| match result {
                Ok((path, document)) => Message::Files(files::Message::Loaded(path, Ok(document))),
                Err(error) => {
                    let path = match &error {
                        AppError::Io { path, .. } => path.clone(),
                        _ => PathBuf::new(),
                    };
                    Message::Files(files::Message::Loaded(path, Err(error)))
                }
            },
        )
    }

    fn file_loaded(&mut self, path: PathBuf, result: Result<TodoDocument, AppError>) -> Task<Message> {
        match result {
            Ok(document) => {
                let unchanged = self.documents.get(&path) == Some(&document);
                self.documents.insert(path.clone(), document);
                if !unchanged && self.is_shown(&path) {
                    self.refresh();
                }
                Task::none()
            }
            Err(error) => {
                let name = path
                    .file_name()
                    .map(|n| n.to_string_lossy().into_owned())
                    .unwrap_or_default();
                self.toast_error(format!(
                    "{}: {}",
                    trf("file_load_failed", &[("file", name.into())]),
                    i18n::error_text(&error)
                ))
            }
        }
    }

    fn file_changed(&mut self, path: PathBuf) -> Task<Message> {
        if path == self.services.ports.saved_filters.location() {
            let store = self.services.ports.saved_filters.clone();
            return Task::perform(async move { store.load() }, |result| {
                Message::Search(search::Message::SavedFiltersLoaded(result))
            });
        }
        if self.state.files.iter().any(|entry| entry.path == path) {
            self.load(path)
        } else {
            Task::none()
        }
    }

    /// Whether the file at `path` is on screen: the active file, or one of
    /// the overview's.
    fn is_shown(&self, path: &Path) -> bool {
        self.shown_paths().iter().any(|shown| shown == path)
    }

    fn document_saved(&mut self, path: PathBuf, document: TodoDocument) -> Task<Message> {
        self.documents.insert(path.clone(), document);
        if self.is_shown(&path) {
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
        .chain(Task::done(Message::Files(files::Message::Picked(Some(path)))))
    }

    fn select_file(&mut self, index: usize) -> Task<Message> {
        if index >= self.state.files.len() {
            return Task::none();
        }
        self.state.active_file = index;
        self.state.overview = false;
        self.file_menu = None;
        self.after_file_switch();
        self.persist_state()
    }

    pub(crate) fn remove_file(&mut self, index: usize) -> Task<Message> {
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

    /// Moves the active file's completed todos to its done file. The
    /// overview has no active file to archive.
    fn archive_completed(&mut self) -> Task<Message> {
        let Some(entry) = self.state.active_entry().cloned() else {
            return Task::none();
        };
        if self.state.shows_overview() {
            return Task::none();
        }
        let Some(done_path) = entry.done_path.clone() else {
            let index = self.state.active_file;
            let notice = self.toast(tr("done_file_needed"));
            return Task::batch([
                notice,
                Task::done(Message::Files(files::Message::ChangeDoneFile(index))),
            ]);
        };
        let service = self.services.todo_files.clone();
        let options = self.file_options();
        let path = entry.path.clone();
        Task::perform(
            async move { service.archive_completed(&entry.path, &done_path, &options) },
            move |result| Message::Files(files::Message::SavedCount(path.clone(), result, "archived")),
        )
    }

    fn reveal(&mut self, path: Option<PathBuf>) -> Task<Message> {
        self.file_menu = None;
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
        match self.active_path() {
            Some(path) => self.change_file(path, change),
            None => Task::none(),
        }
    }

    /// Runs a change on the file at `path` in the background and shows the
    /// resulting document. Changes to a todo go to the file it is in, which
    /// in the overview need not be the active one.
    pub(crate) fn change_file(
        &self,
        path: PathBuf,
        change: impl FnOnce(&TodoFileService, &Path, &FileOptions) -> Result<TodoDocument, AppError> + Send + 'static,
    ) -> Task<Message> {
        let service = self.services.todo_files.clone();
        let options = self.file_options();
        let target = path.clone();
        Task::perform(async move { change(&service, &target, &options) }, move |result| {
            Message::Files(files::Message::Saved(path.clone(), result))
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
