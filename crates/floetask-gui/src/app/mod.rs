//! The application state and the Elm-style loop around it.
//!
//! `update` is split by concern into the sibling modules; each handler only
//! calls application services and changes presentation state.

mod board;
mod calendar;
mod dialog;
mod drawer;
mod editing;
mod files;
mod list;
mod message;
mod search;
mod settings;
mod shortcuts;
mod subscriptions;
mod updates;
mod window_frame;

use std::collections::{BTreeSet, HashMap};
use std::path::{Path, PathBuf};
use std::time::Duration;

use iced::widget::markdown;
use iced::{Element, Task, Theme, window};

use floetask_application::TodoRef;
use floetask_application::{
    AppError, AppState, ColorOverrides, FileOptions, InputOptions, NoteStatus, SavedFilter, Services, Settings,
    ThemePreference,
};
use floetask_domain::listing::{
    AttributeSummary, Board, BoardLayout, DateContext, LaneId, TodoListing, build_board, build_listing, known_names,
    summarize_attributes,
};
use floetask_domain::search::Query;
use floetask_domain::{Date, TodoDocument};

use crate::i18n::tr;
use crate::theme::Colors;

pub use board::BoardDrag;
pub use dialog::{Calendar, DateKey, Dialog, Editor, Pending, SettingsDialog, SettingsSection};
#[cfg(test)]
pub(crate) use editing::NoteSave;
pub use message::{Message, SettingChange, ViewToggle};
pub use updates::UpdateState;

/// What the composition root hands to the GUI.
pub struct Startup {
    pub services: Services,
    /// A file passed on the command line, opened and made active.
    pub open: Option<PathBuf>,
}

/// A short message at the bottom of the window.
#[derive(Debug, Clone)]
pub struct Toast {
    pub id: u64,
    pub text: String,
    pub error: bool,
}

#[derive(Debug)]
pub struct SearchState {
    pub input: String,
    /// The last query that parsed, shown while an expression is incomplete.
    pub query: Query,
    pub saved_open: bool,
}

impl Default for SearchState {
    fn default() -> Self {
        Self {
            input: String::new(),
            query: Query::Empty,
            saved_open: false,
        }
    }
}

pub struct Floetask {
    pub(crate) services: Services,
    pub(crate) settings: Settings,
    pub(crate) state: AppState,
    pub(crate) color_overrides: ColorOverrides,
    pub(crate) system_dark: bool,
    pub(crate) today: Date,
    /// Every registered file, loaded. The active one is shown.
    pub(crate) documents: HashMap<PathBuf, TodoDocument>,
    pub(crate) listing: TodoListing,
    /// The active file on the status board.
    pub(crate) board: Board,
    /// The card being dragged on the board.
    pub(crate) drag: Option<BoardDrag>,
    /// Board column under the mouse.
    pub(crate) board_hover: Option<LaneId>,
    /// The day the calendar is centred on.
    pub(crate) calendar_anchor: Date,
    /// A todo being dragged on the calendar, with the day it came from.
    pub(crate) calendar_drag: Option<(TodoRef, Option<Date>)>,
    pub(crate) calendar_hover: Option<Date>,
    /// The panel listing todos without a due date is open.
    pub(crate) calendar_undated_open: bool,
    pub(crate) calendar_undated_hover: bool,
    pub(crate) summaries: Vec<AttributeSummary>,
    pub(crate) projects: BTreeSet<String>,
    pub(crate) contexts: BTreeSet<String>,
    pub(crate) markdown: HashMap<String, markdown::Content>,
    /// Whether each `note:` value in the active file points at a file.
    pub(crate) note_statuses: HashMap<String, NoteStatus>,
    pub(crate) search: SearchState,
    pub(crate) saved_filters: Vec<SavedFilter>,
    /// Index into the flattened visible list.
    pub(crate) selected: Option<usize>,
    /// Line whose action bar (copy, archive, delete) is open.
    pub(crate) row_menu: Option<usize>,
    pub(crate) file_menu: Option<usize>,
    pub(crate) dialog: Option<Dialog>,
    pub(crate) toasts: Vec<Toast>,
    next_toast_id: u64,
    /// Held modifier keys, so Ctrl+Enter in the search field can be told
    /// apart from Enter.
    pub(crate) modifiers: iced::keyboard::Modifiers,
    /// The main window, for dragging, resizing and the window buttons.
    pub(crate) window: Option<window::Id>,
    pub(crate) maximized: bool,
    /// Line of the todo under the mouse.
    pub(crate) hovered: Option<usize>,
    /// Bumped whenever the set of watched files changes.
    pub(crate) watch_generation: u64,
    /// The release check on the About page.
    pub(crate) update: UpdateState,
}

impl Floetask {
    pub fn boot(startup: Startup) -> (Self, Task<Message>) {
        let services = startup.services;
        let ports = &services.ports;
        let mut toasts = Vec::new();
        let settings = ports.settings.load().unwrap_or_else(|error| {
            toasts.push(error.to_string());
            Settings::default()
        });
        // Write the defaults once so users find a config file to edit.
        if !ports.files.exists(&ports.settings.location()) {
            let _ = ports.settings.save(&settings);
        }
        let color_overrides = ports.colors.load().unwrap_or_else(|error| {
            toasts.push(error.to_string());
            ColorOverrides::default()
        });
        let mut state = ports.state.load();
        if let Some(path) = startup.open {
            state.open_file(absolute(&path));
        }
        let today = ports.clock.today();
        let maximized = state.window.maximized;

        let mut app = Self {
            services,
            settings,
            state,
            color_overrides,
            system_dark: false,
            today,
            documents: HashMap::new(),
            listing: TodoListing::default(),
            board: Board::default(),
            drag: None,
            board_hover: None,
            calendar_anchor: today,
            calendar_drag: None,
            calendar_hover: None,
            calendar_undated_open: false,
            calendar_undated_hover: false,
            summaries: Vec::new(),
            projects: BTreeSet::new(),
            contexts: BTreeSet::new(),
            markdown: HashMap::new(),
            note_statuses: HashMap::new(),
            search: SearchState::default(),
            saved_filters: Vec::new(),
            selected: None,
            row_menu: None,
            file_menu: None,
            dialog: None,
            toasts: Vec::new(),
            next_toast_id: 0,
            window: None,
            maximized,
            hovered: None,
            modifiers: iced::keyboard::Modifiers::default(),
            watch_generation: 0,
            update: UpdateState::default(),
        };
        let toast_tasks: Vec<_> = toasts.into_iter().map(|text| app.toast_error(text)).collect();
        let load_files = Task::batch([app.load_all_files(), app.persist_state()]);
        let saved_filters = {
            let store = app.services.ports.saved_filters.clone();
            Task::perform(async move { store.load() }, Message::SavedFiltersLoaded)
        };
        let updates = if app.settings.check_for_updates {
            app.check_for_updates(false)
        } else {
            Task::none()
        };
        let window = window::oldest().map(Message::WindowReady);
        let system_theme = iced::system::theme().map(|mode| Message::SystemDark(mode == iced::theme::Mode::Dark));
        (
            app,
            Task::batch(
                [load_files, saved_filters, system_theme, window, updates]
                    .into_iter()
                    .chain(toast_tasks),
            ),
        )
    }

    pub fn title(&self) -> String {
        match self.state.active_entry() {
            Some(entry) => format!("{} - floetask", entry.file_name()),
            None => "floetask".to_owned(),
        }
    }

    pub fn theme(&self) -> Theme {
        self.colors().theme()
    }

    pub fn scale_factor(&self) -> f32 {
        f32::from(self.settings.zoom_percent) / 100.0
    }

    /// The side panels to draw: the open ones, minus those the week view
    /// needs the room of (see `layout`).
    pub(crate) fn shown_panels(&self) -> crate::layout::Panels {
        let open = crate::layout::Panels {
            files: self.state.files_drawer_open,
            filters: self.state.drawer_open && self.active_document().is_some(),
            undated: self.calendar_undated_open,
        };
        let week = self.state.main_view == floetask_application::MainView::Calendar
            && self.state.calendar_mode == floetask_domain::calendar::CalendarMode::Week;
        if week {
            let width = self.state.window.width / self.scale_factor();
            crate::layout::fit(open, width, crate::layout::WEEK_MIN_WIDTH)
        } else {
            open
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        crate::view::root(self)
    }

    pub fn subscription(&self) -> iced::Subscription<Message> {
        subscriptions::all(self)
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        use Message as M;
        match message {
            M::SystemDark(dark) => {
                self.system_dark = dark;
                Task::none()
            }
            M::Tick => self.tick(),
            M::KeyPressed(key, modifiers) => self.on_key(key, modifiers),
            M::ModifiersChanged(modifiers) => {
                self.modifiers = modifiers;
                Task::none()
            }
            M::Quit => self.quit(),
            M::WindowReady(..)
            | M::WindowDrag
            | M::WindowResize(_)
            | M::WindowMinimize
            | M::WindowToggleMaximize
            | M::RowHover(_) => self.update_window_frame(message),
            M::Window(id, event) => self.on_window_event(id, event),
            M::DismissToast(id) => {
                self.toasts.retain(|toast| toast.id != id);
                Task::none()
            }
            M::Noop => Task::none(),
            M::Failed(error) => self.report(&error),

            M::FileLoaded(..)
            | M::FileChanged(_)
            | M::Saved(..)
            | M::SavedCount(..)
            | M::OpenFileDialog
            | M::CreateFileDialog
            | M::FilePicked(_)
            | M::FileCreated(_)
            | M::SelectFile(_)
            | M::FileMenu(_)
            | M::OpenDoneFile(_)
            | M::ToggleFilesDrawer
            | M::ChangeDoneFile(_)
            | M::DoneFilePicked(..)
            | M::RevealTodoFile(_)
            | M::AskRemoveFile(_)
            | M::ArchiveCompleted
            | M::SavedFiltersLoaded(_) => self.update_files(message),

            M::OpenTodo(_)
            | M::ToggleComplete(_)
            | M::RowMenu(_)
            | M::CopyTodo(_)
            | M::ArchiveOne(_)
            | M::AskDelete(_)
            | M::ChipFilter(..)
            | M::ToggleListGroup(_)
            | M::OpenLink(_)
            | M::OpenRowDatePicker(..)
            | M::OpenNote(_)
            | M::NoteOpened(_) => self.update_list(message),

            M::NewTodo
            | M::EditorAction(_)
            | M::EditorSuggestion(_)
            | M::EditorSuggestionMove(_)
            | M::EditorAccept
            | M::EditorDismissSuggestions
            | M::EditorPriority(_)
            | M::EditorStatus(_)
            | M::EditorOpenCalendar(_)
            | M::EditorCloseCalendar
            | M::EditorRecurrenceCount(_)
            | M::EditorRecurrenceUnit(_)
            | M::EditorRecurrenceStrict(_)
            | M::EditorApplyRecurrence(_)
            | M::EditorPomodoros(_)
            | M::EditorApplyPomodoros(_)
            | M::EditorNoteAction(_)
            | M::NoteLoaded(..)
            | M::SaveEditor
            | M::CalendarMonth(_)
            | M::CalendarPick(_) => self.update_editing(message),

            M::ToggleDrawer
            | M::DrawerTab(_)
            | M::ToggleSection(_)
            | M::FilterValue(..)
            | M::ToggleCategory(_)
            | M::ViewToggle(..)
            | M::MoveSort(..)
            | M::InvertSort(_)
            | M::FileOrder(_)
            | M::CompletedLast(_)
            | M::AskRename(..)
            | M::RenameInput(_)
            | M::AskRemoveValue(..)
            | M::ResetFilters => self.update_drawer(message),

            M::FocusSearch
            | M::SearchInput(_)
            | M::SearchSubmitAsTodo
            | M::ToggleSavedFilters
            | M::AskSaveFilter
            | M::SaveFilterName(_)
            | M::ApplySavedFilter(_)
            | M::AskDeleteSavedFilter(_)
            | M::ToggleSuppress(_) => self.update_search(message),

            M::CheckForUpdates | M::UpdateChecked(..) | M::InstallUpdate | M::UpdateInstalled(_) => {
                self.update_updates(message)
            }
            M::OpenSettings | M::Setting(_) | M::ToggleTheme | M::SettingsSection(_) | M::SettingsStatusInput(_) => {
                self.update_settings(message)
            }

            M::ShowMainView(_)
            | M::CalendarMode(_)
            | M::CalendarStep(_)
            | M::CalendarToday
            | M::CalendarOpenDay(_)
            | M::CalendarPress(..)
            | M::CalendarToggleUndated
            | M::CalendarHoverUndated(_)
            | M::CalendarHover(..)
            | M::CalendarRelease => self.update_calendar(message),
            M::ToggleMainView
            | M::BoardGrouped(_)
            | M::ToggleBoardGroup(_)
            | M::BoardPress(..)
            | M::BoardHover(..)
            | M::BoardRelease
            | M::OpenBoardColumns
            | M::BoardColumnInput(_)
            | M::BoardColumnAdd(_)
            | M::BoardColumnMove(..)
            | M::BoardColumnRemove(_)
            | M::BoardColumnsReset => self.update_board(message),

            M::ConfirmDialog => self.confirm_dialog(),
            M::CloseDialog => self.close_dialog(),
        }
    }

    // Shared helpers used by the handlers.

    pub(crate) fn colors(&self) -> Colors {
        let dark = match self.settings.theme {
            ThemePreference::System => self.system_dark,
            ThemePreference::Light => false,
            ThemePreference::Dark => true,
        };
        Colors::new(dark, &self.color_overrides)
    }

    pub(crate) fn date_context(&self) -> DateContext {
        DateContext {
            today: self.today,
            week_start: self.settings.week_start,
            human_friendly: self.settings.human_friendly_dates,
        }
    }

    pub(crate) fn file_options(&self) -> FileOptions {
        FileOptions::from(&self.settings)
    }

    pub(crate) fn input_options(&self) -> InputOptions {
        InputOptions::from(&self.settings)
    }

    pub(crate) fn active_path(&self) -> Option<PathBuf> {
        self.state.active_entry().map(|entry| entry.path.clone())
    }

    pub(crate) fn active_document(&self) -> Option<&TodoDocument> {
        self.state
            .active_entry()
            .and_then(|entry| self.documents.get(&entry.path))
    }

    /// Recomputes everything derived from the active document and the view
    /// settings. Called after every change that can affect the list.
    pub(crate) fn refresh(&mut self) {
        let dates = self.date_context();
        let empty = TodoDocument::empty();
        let document = self.active_document().unwrap_or(&empty);
        let listing = build_listing(
            document,
            &self.state.view,
            &self.state.sorting,
            &self.search.query,
            &dates,
            &self.settings.statuses,
        );
        let summaries = summarize_attributes(
            document,
            &self.state.view,
            &self.search.query,
            &dates,
            &self.settings.statuses,
        );
        let board = build_board(
            document,
            &self.state.view,
            &self.state.sorting,
            &self.search.query,
            &dates,
            &self.settings.statuses,
            &BoardLayout {
                columns: self.active_board_columns(),
                grouped: self.state.board_grouped,
            },
        );
        let (projects, contexts) = known_names(document);
        // A few file checks, one per distinct note; cheap enough to run here.
        let note_statuses = match self.active_path() {
            Some(path) => self.services.notes.statuses(&path, document),
            None => HashMap::new(),
        };
        self.listing = listing;
        self.board = board;
        self.note_statuses = note_statuses;
        self.summaries = summaries;
        self.projects = projects;
        self.contexts = contexts;

        let bodies: BTreeSet<String> = self.listing.todos().map(|entry| entry.todo.display_text()).collect();
        self.markdown.retain(|body, _| bodies.contains(body));
        for body in bodies {
            if crate::view::list::looks_like_markdown(&body) && !self.markdown.contains_key(&body) {
                let content = markdown::Content::parse(&body);
                self.markdown.insert(body, content);
            }
        }

        let visible = self.selectable_todos().count();
        self.selected = self
            .selected
            .filter(|_| visible > 0)
            .map(|index| index.min(visible - 1));
    }

    pub(crate) fn persist_state(&self) -> Task<Message> {
        let store = self.services.ports.state.clone();
        let state = self.state.clone();
        Task::perform(async move { store.save(&state) }, Message::from_result)
    }

    pub(crate) fn toast(&mut self, text: impl Into<String>) -> Task<Message> {
        self.push_toast(text.into(), false)
    }

    pub(crate) fn toast_error(&mut self, text: impl Into<String>) -> Task<Message> {
        self.push_toast(text.into(), true)
    }

    fn push_toast(&mut self, text: String, error: bool) -> Task<Message> {
        let id = self.next_toast_id;
        self.next_toast_id += 1;
        self.toasts.push(Toast { id, text, error });
        let seconds = if error { 6 } else { 3 };
        Task::perform(tokio::time::sleep(Duration::from_secs(seconds)), move |()| {
            Message::DismissToast(id)
        })
    }

    pub(crate) fn report(&mut self, error: &AppError) -> Task<Message> {
        self.toast_error(error.to_string())
    }

    pub(crate) fn close_dialog(&mut self) -> Task<Message> {
        self.dialog = None;
        Task::none()
    }

    fn confirm_dialog(&mut self) -> Task<Message> {
        match self.dialog.take() {
            Some(Dialog::Confirm { pending, .. }) => self.run_pending(pending),
            Some(Dialog::Rename { attribute, from, input }) => self.rename_value(attribute, from, input),
            Some(Dialog::SaveFilter { name }) => self.save_filter(name),
            Some(Dialog::BoardColumns { columns, .. }) => self.save_board_columns(columns),
            Some(Dialog::Editor(editor)) => {
                self.dialog = Some(Dialog::Editor(editor));
                self.update_editing(Message::SaveEditor)
            }
            other => {
                self.dialog = other;
                Task::none()
            }
        }
    }

    fn run_pending(&mut self, pending: Pending) -> Task<Message> {
        match pending {
            Pending::DeleteTodo(target) => self.delete_todo(target),
            Pending::RemoveFile(index) => self.remove_file(index),
            Pending::RemoveValue(attribute, value) => self.remove_value(attribute, value),
            Pending::DeleteSavedFilter(index) => self.delete_saved_filter(index),
        }
    }

    /// Re-reads "today" and checks for due notifications.
    fn tick(&mut self) -> Task<Message> {
        let today = self.services.ports.clock.today();
        if today != self.today {
            self.today = today;
            self.refresh();
        }
        self.check_notifications()
    }

    fn check_notifications(&self) -> Task<Message> {
        if !self.settings.notifications {
            return Task::none();
        }
        let service = self.services.notifications.clone();
        let pending = service.pending(
            self.documents.values(),
            self.settings.notification_threshold_days,
            &self.saved_filters,
        );
        if pending.is_empty() {
            return Task::none();
        }
        Task::perform(
            async move {
                for notification in &pending {
                    let title = match notification.days_left {
                        0 => tr("due_today").to_owned(),
                        1 => tr("due_tomorrow").to_owned(),
                        days => crate::i18n::trf("due_in_days", &[&days]),
                    };
                    service.send(notification, &title)?;
                }
                Ok(())
            },
            Message::from_result,
        )
    }

    fn on_window_event(&mut self, _id: window::Id, event: window::Event) -> Task<Message> {
        match event {
            window::Event::FileDropped(path) => self.open_path(path),
            window::Event::Resized(size) => {
                self.state.window.width = size.width;
                self.state.window.height = size.height;
                Task::none()
            }
            window::Event::Moved(position) => {
                self.state.window.x = Some(position.x);
                self.state.window.y = Some(position.y);
                Task::none()
            }
            window::Event::CloseRequested => self.quit(),
            _ => Task::none(),
        }
    }
}

impl Floetask {
    /// Saves where the user left off, then exits.
    fn quit(&mut self) -> Task<Message> {
        if let Err(error) = self.services.ports.state.save(&self.state) {
            eprintln!("floetask: {error}");
        }
        iced::exit()
    }
}

/// Makes a path absolute against the current directory, for CLI arguments.
fn absolute(path: &Path) -> PathBuf {
    std::path::absolute(path).unwrap_or_else(|_| path.to_path_buf())
}
