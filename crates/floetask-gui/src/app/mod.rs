//! The application state and the Elm-style loop around it.
//!
//! `dispatch` hands each feature message to the feature's own `update`
//! (`crate::board`, `crate::files`, ...); each handler only calls
//! application services and changes presentation state. What every view
//! shares, the window, dialogs, keys and toasts, is handled here.

mod dialog;
mod message;
mod shortcuts;
mod subscriptions;
mod window_frame;

use std::collections::{BTreeSet, HashMap};
use std::path::{Path, PathBuf};
use std::time::Duration;

use iced::widget::markdown;
use iced::{Element, Task, Theme, window};

use floetask_application::TodoRef;
use floetask_application::{
    AppError, AppState, ColorOverrides, FileOptions, InputOptions, MainView, NoteStatus, SavedFilter, Services,
    Settings, ThemePreference,
};
use floetask_domain::listing::{
    AttributeSummary, Board, BoardLayout, DateContext, Group, LaneId, ListedTodo, TodoListing, build_board,
    build_listing, build_overview, known_names, overview_board, summarize_attributes,
};
use floetask_domain::{Date, TodoDocument, WeekStart};

use crate::board::BoardDrag;
use crate::files::FileDrag;
use crate::i18n::{self, tr};
use crate::note_window::NoteWindow;
use crate::search;
use crate::search::SearchState;
use crate::settings::UpdateState;
use crate::theme::Colors;

pub use dialog::{Dialog, Pending};
pub use message::Message;

/// What the composition root hands to the GUI.
pub struct Startup {
    pub services: Services,
    /// A file passed on the command line, opened and made active.
    pub open: Option<PathBuf>,
}

/// A todo the user acted on and the file it is in. The overview mixes
/// files, so every action on a todo names its file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Target {
    pub file: PathBuf,
    pub todo: TodoRef,
}

#[cfg(test)]
impl Target {
    pub fn new(file: impl Into<PathBuf>, line: usize, todo: &floetask_domain::Todo) -> Self {
        Self {
            file: file.into(),
            todo: TodoRef::new(line, todo),
        }
    }
}

/// A short message at the bottom of the window.
#[derive(Debug, Clone)]
pub struct Toast {
    pub id: u64,
    pub text: String,
    pub error: bool,
}

pub struct Floetask {
    pub(crate) services: Services,
    pub(crate) settings: Settings,
    pub(crate) state: AppState,
    pub(crate) color_overrides: ColorOverrides,
    pub(crate) system_dark: bool,
    pub(crate) today: Date,
    /// Every registered file, loaded. The active one is shown, or all of
    /// them in the overview.
    pub(crate) documents: HashMap<PathBuf, TodoDocument>,
    /// The files shown: the active one, or the overview's. A listed todo's
    /// `file` indexes this.
    pub(crate) listed_files: Vec<PathBuf>,
    pub(crate) listing: TodoListing,
    /// The shown files on the status board.
    pub(crate) board: Board,
    /// The card being dragged on the board.
    pub(crate) drag: Option<BoardDrag>,
    /// Board column under the mouse.
    pub(crate) board_hover: Option<LaneId>,
    /// The day the calendar is centred on.
    pub(crate) calendar_anchor: Date,
    /// A todo being dragged on the calendar, with the day it came from.
    pub(crate) calendar_drag: Option<(Target, Option<Date>)>,
    /// The cursor left the calendar todo since it was pressed, so releasing
    /// it is a drop, never a click.
    pub(crate) calendar_drag_moved: bool,
    pub(crate) calendar_hover: Option<Date>,
    /// The panel listing todos without a due date is open.
    pub(crate) calendar_undated_open: bool,
    pub(crate) calendar_undated_hover: bool,
    /// A side panel the user opened while it does not fit beside the main
    /// view: drawn on top of it instead.
    pub(crate) panel_overlay: Option<crate::layout::Panel>,
    pub(crate) summaries: Vec<AttributeSummary>,
    pub(crate) projects: BTreeSet<String>,
    pub(crate) contexts: BTreeSet<String>,
    pub(crate) markdown: HashMap<String, markdown::Content>,
    /// Whether each `note:` value of a shown file points at a file, by
    /// todo file.
    pub(crate) note_statuses: HashMap<PathBuf, HashMap<String, NoteStatus>>,
    pub(crate) search: SearchState,
    pub(crate) saved_filters: Vec<SavedFilter>,
    /// Index into the flattened visible list.
    pub(crate) selected: Option<usize>,
    /// The todo (`ListedTodo::id`) whose menu (copy, archive, delete) is
    /// open.
    pub(crate) row_menu: Option<(usize, usize)>,
    pub(crate) file_menu: Option<usize>,
    /// The file drawer's sort menu is open.
    pub(crate) file_sort_menu: bool,
    /// The file being dragged in the file drawer.
    pub(crate) file_drag: Option<FileDrag>,
    /// The file (with its archive) under the mouse in the file drawer.
    pub(crate) file_hover: Option<usize>,
    pub(crate) dialog: Option<Dialog>,
    /// The second window, editing a note; reused for every note.
    pub(crate) note_window: Option<NoteWindow>,
    pub(crate) toasts: Vec<Toast>,
    next_toast_id: u64,
    /// Held modifier keys, so Ctrl+Enter in the search field can be told
    /// apart from Enter.
    pub(crate) modifiers: iced::keyboard::Modifiers,
    /// The main window, for dragging, resizing and the window buttons.
    pub(crate) window: Option<window::Id>,
    pub(crate) maximized: bool,
    /// The todo (`ListedTodo::id`) under the mouse.
    pub(crate) hovered: Option<(usize, usize)>,
    /// Bumped whenever the set of watched files changes.
    pub(crate) watch_generation: u64,
    /// The release check on the About page.
    pub(crate) update: UpdateState,
    /// The releases the last check found, newest first: for the channel's
    /// offer and the rollback list.
    pub(crate) published: Vec<floetask_application::ports::PublishedRelease>,
    /// The earlier version picked on the About page to roll back to.
    pub(crate) rollback_to: Option<String>,
}

impl Floetask {
    pub fn boot(startup: Startup) -> (Self, Task<Message>) {
        let services = startup.services;
        let ports = &services.ports;
        let mut toasts = Vec::new();
        let system_locale = ports.locale.language();
        let mut settings = ports.settings.load().unwrap_or_else(|error| {
            toasts.push(i18n::error_text(&error));
            Settings::default()
        });
        // Write the defaults once so users find a config file to edit. The
        // week starts where it does in the user's region.
        if !ports.files.exists(&ports.settings.location()) {
            settings.week_start = system_locale.as_deref().map(WeekStart::for_locale).unwrap_or_default();
            let _ = ports.settings.save(&settings);
        }
        i18n::set_language(i18n::resolve(&settings.language, system_locale.as_deref()));
        let color_overrides = ports.colors.load().unwrap_or_else(|error| {
            toasts.push(i18n::error_text(&error));
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
            listed_files: Vec::new(),
            listing: TodoListing::default(),
            board: Board::default(),
            drag: None,
            board_hover: None,
            calendar_anchor: today,
            calendar_drag: None,
            calendar_drag_moved: false,
            calendar_hover: None,
            calendar_undated_open: false,
            calendar_undated_hover: false,
            panel_overlay: None,
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
            file_sort_menu: false,
            file_drag: None,
            file_hover: None,
            dialog: None,
            note_window: None,
            toasts: Vec::new(),
            next_toast_id: 0,
            window: None,
            maximized,
            hovered: None,
            modifiers: iced::keyboard::Modifiers::default(),
            watch_generation: 0,
            update: UpdateState::default(),
            published: Vec::new(),
            rollback_to: None,
        };
        let toast_tasks: Vec<_> = toasts.into_iter().map(|text| app.toast_error(text)).collect();
        let load_files = Task::batch([app.load_all_files(), app.persist_state()]);
        let saved_filters = {
            let store = app.services.ports.saved_filters.clone();
            Task::perform(async move { store.load() }, |result| {
                Message::Search(search::Message::SavedFiltersLoaded(result))
            })
        };
        let updates = if app.settings.check_for_updates {
            app.check_for_updates(false)
        } else {
            Task::none()
        };
        let system_theme = iced::system::theme().map(|mode| Message::SystemDark(mode == iced::theme::Mode::Dark));
        (
            app,
            Task::batch(
                [load_files, saved_filters, system_theme, updates]
                    .into_iter()
                    .chain(toast_tasks),
            ),
        )
    }

    /// The title of either window.
    pub fn window_title(&self, id: window::Id) -> String {
        match &self.note_window {
            Some(window) if window.id == id => format!("{} - floetask", window.title()),
            _ => self.title(),
        }
    }

    /// Draws either window: the note window, or the main one.
    pub fn view_window(&self, id: window::Id) -> Element<'_, Message> {
        match &self.note_window {
            Some(window) if window.id == id => crate::note_window::view(self, window),
            _ => self.view(),
        }
    }

    pub fn title(&self) -> String {
        match self.state.active_entry() {
            _ if self.state.shows_overview() => format!("{} - floetask", tr("overview")),
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

    /// The side panels to draw: the open ones, minus those the main view
    /// needs the room of (see `layout`).
    pub(crate) fn shown_panels(&self) -> crate::layout::Panels {
        let open = crate::layout::Panels {
            files: self.state.files_drawer_open,
            filters: self.state.drawer_open && self.active_document().is_some(),
            undated: self.calendar_undated_open && self.state.main_view == floetask_application::MainView::Calendar,
        };
        let width = self.state.window.width / self.scale_factor();
        crate::layout::fit(open, width, crate::layout::MAIN_MIN_WIDTH)
    }

    /// Opens or closes a side panel from its button. A panel that is open
    /// but hidden for lack of room is shown over the main view instead; the
    /// next click hides that overlay again.
    pub(crate) fn toggle_panel(&mut self, panel: crate::layout::Panel) -> Task<Message> {
        use crate::layout::Panel;
        if self.panel_overlay == Some(panel) {
            self.panel_overlay = None;
            return Task::none();
        }
        let open = match panel {
            Panel::Files => &mut self.state.files_drawer_open,
            Panel::Filters => &mut self.state.drawer_open,
        };
        let opened = !*open;
        let hidden = |app: &Self| {
            let shown = app.shown_panels();
            !match panel {
                Panel::Files => shown.files,
                Panel::Filters => shown.filters,
            }
        };
        if !opened && hidden(self) {
            // Open but squeezed out: the user asks to see it, not to close it.
            self.panel_overlay = Some(panel);
            return Task::none();
        }
        *match panel {
            Panel::Files => &mut self.state.files_drawer_open,
            Panel::Filters => &mut self.state.drawer_open,
        } = opened;
        if opened && hidden(self) {
            self.panel_overlay = Some(panel);
        }
        self.persist_state()
    }

    pub fn view(&self) -> Element<'_, Message> {
        crate::view::root(self)
    }

    pub fn subscription(&self) -> iced::Subscription<Message> {
        subscriptions::all(self)
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        let task = self.dispatch(message);
        // The notes editor draws its own colours and texts, so it follows
        // the theme and language whichever message changed them.
        let style = crate::theme::markdown_editor(self.colors());
        let language = crate::i18n::language().editor();
        if let Some(Dialog::Editor(editor)) = &mut self.dialog {
            editor.note.set_style(style);
            editor.note.set_language(language);
        }
        if let Some(window) = &mut self.note_window {
            window.note.set_language(language);
        }
        task
    }

    fn dispatch(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::SystemDark(dark) => {
                self.system_dark = dark;
                Task::none()
            }
            Message::Tick => self.tick(),
            Message::KeyPressed(id, key, modifiers) => self.on_key(id, key, modifiers),
            Message::ModifiersChanged(modifiers) => {
                self.modifiers = modifiers;
                Task::none()
            }
            Message::Quit => self.quit(),
            Message::WindowReady(..)
            | Message::WindowDrag
            | Message::WindowResize(_)
            | Message::WindowMinimize
            | Message::WindowToggleMaximize => self.update_window_frame(message),
            Message::Window(id, event) => self.on_window_event(id, event),
            // A focused notes editor takes `Escape`; in the dialog it still
            // closes the dialog, in the note window it does nothing.
            Message::NoteEditorEscape(id) => match &self.note_window {
                Some(window) if window.id == id => Task::none(),
                _ => self.close_dialog(),
            },
            Message::DismissToast(id) => {
                self.toasts.retain(|toast| toast.id != id);
                Task::none()
            }
            Message::Noop => Task::none(),
            Message::Failed(error) => self.report(&error),

            Message::ToggleMainView => self.show_main_view(match self.state.main_view {
                MainView::List | MainView::Calendar => MainView::Board,
                MainView::Board => MainView::List,
            }),
            Message::ShowMainView(view) => self.show_main_view(view),
            Message::ClosePanelOverlay => {
                self.panel_overlay = None;
                Task::none()
            }
            Message::OpenLink(uri) => {
                let desktop = self.services.ports.desktop.clone();
                Task::perform(async move { desktop.open_uri(&uri) }, Message::from_result)
            }
            Message::ConfirmDialog => self.confirm_dialog(),
            Message::CloseDialog => self.close_dialog(),

            Message::Files(message) => self.update_files(message),
            Message::List(message) => self.update_list(message),
            Message::Editor(message) => self.update_editor(message),
            Message::NoteWindow(message) => self.update_note_window(message),
            Message::Drawer(message) => self.update_drawer(message),
            Message::Search(message) => self.update_search(message),
            Message::Settings(message) => self.update_settings(message),
            Message::Board(message) => self.update_board(message),
            Message::Calendar(message) => self.update_calendar(message),
        }
    }

    /// Switches between the list, the board and the calendar. The calendar
    /// always opens on today.
    fn show_main_view(&mut self, view: MainView) -> Task<Message> {
        if view == MainView::Calendar && self.state.main_view != MainView::Calendar {
            self.calendar_anchor = self.today;
        }
        self.state.main_view = view;
        self.row_menu = None;
        self.drag = None;
        self.calendar_drag = None;
        self.persist_state()
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

    /// The files to show: every file of the overview, else the active one.
    pub(crate) fn shown_paths(&self) -> Vec<PathBuf> {
        if self.state.shows_overview() {
            self.state
                .overview_files()
                .into_iter()
                .map(|entry| entry.path.clone())
                .collect()
        } else {
            self.active_path().into_iter().collect()
        }
    }

    /// The target of a listed todo, in the file it was listed from.
    pub(crate) fn target(&self, entry: &ListedTodo) -> Target {
        Target {
            file: self.listed_files.get(entry.file).cloned().unwrap_or_default(),
            todo: TodoRef::new(entry.line, &entry.todo),
        }
    }

    /// The path of a listed file, by its index in the listing.
    pub(crate) fn listed_file(&self, file: usize) -> Option<&Path> {
        self.listed_files.get(file).map(PathBuf::as_path)
    }

    /// The name a list group's collapsed state is kept under. An overview's
    /// file groups are named by path, so they survive reordering the files.
    pub(crate) fn list_group_key(&self, group: &Group) -> String {
        match group.file.and_then(|file| self.listed_file(file)) {
            Some(path) => format!("file:{}", path.display()),
            None => group.key(),
        }
    }

    /// Recomputes everything derived from the shown documents and the view
    /// settings. Called after every change that can affect the list.
    pub(crate) fn refresh(&mut self) {
        let dates = self.date_context();
        let empty = TodoDocument::empty();
        let paths = self.shown_paths();
        let documents: Vec<&TodoDocument> = paths
            .iter()
            .map(|path| self.documents.get(path).unwrap_or(&empty))
            .collect();
        let (view, sorting, query, statuses) = (
            &self.state.view,
            &self.state.sorting,
            &self.search.query,
            &self.settings.statuses,
        );
        let board_of = |document: &TodoDocument, layout: BoardLayout| {
            build_board(document, view, sorting, query, &dates, statuses, &layout)
        };
        let (listing, board) = if self.state.shows_overview() {
            let listing = build_overview(&documents, view, sorting, query, &dates, statuses);
            // Each file keeps its own columns; the files are the groups.
            let boards = paths.iter().zip(&documents).map(|(path, document)| {
                let layout = BoardLayout {
                    columns: self.settings.board_columns(path),
                    grouped: false,
                };
                board_of(document, layout)
            });
            (listing, overview_board(boards))
        } else {
            let document = documents.first().copied().unwrap_or(&empty);
            let layout = BoardLayout {
                columns: self.active_board_columns(),
                grouped: self.state.board_grouped,
            };
            let listing = build_listing(document, view, sorting, query, &dates, statuses);
            (listing, board_of(document, layout))
        };
        let summaries = summarize_attributes(&documents, view, query, &dates, statuses);
        let (projects, contexts) = known_names(&documents);
        // A few file checks, one per distinct note; cheap enough to run here.
        let note_statuses = paths
            .iter()
            .zip(&documents)
            .map(|(path, document)| (path.clone(), self.services.notes.statuses(path, document)))
            .collect();
        self.listed_files = paths;
        self.listing = listing;
        self.board = board;
        self.note_statuses = note_statuses;
        self.summaries = summaries;
        self.projects = projects;
        self.contexts = contexts;

        let bodies: BTreeSet<String> = self.listing.todos().map(|entry| entry.todo.display_text()).collect();
        self.markdown.retain(|body, _| bodies.contains(body));
        for body in bodies {
            if crate::list::looks_like_markdown(&body) && !self.markdown.contains_key(&body) {
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
        self.toast_error(i18n::error_text(error))
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
                self.update_editor(crate::editor::Message::Save)
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
            self.settings.week_start,
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
                        days => i18n::trf("due_in_days", &[("days", days.into())]),
                    };
                    service.send(notification, &title)?;
                }
                Ok(())
            },
            Message::from_result,
        )
    }

    fn on_window_event(&mut self, id: window::Id, event: window::Event) -> Task<Message> {
        if self.note_window.as_ref().is_some_and(|window| window.id == id) {
            return self.on_note_window_event(event);
        }
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
        self.save_note_window_now();
        if let Err(error) = self.services.ports.state.save(&self.state) {
            eprintln!("floetask: {error}");
        }
        iced::exit()
    }
}

/// Makes a path absolute against the current directory, for CLI arguments.
pub(crate) fn absolute(path: &Path) -> PathBuf {
    std::path::absolute(path).unwrap_or_else(|_| path.to_path_buf())
}
