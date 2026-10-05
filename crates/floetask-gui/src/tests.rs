//! Headless GUI tests: render views in iced's simulator, interact, and feed
//! the produced messages back into `update`.
//!
//! Set `FLOETASK_SNAPSHOTS=<dir>` to also write PNG snapshots of each screen
//! for a visual check.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use iced_test::simulator;

use floetask_application::ports::*;
use floetask_application::{
    AppError, AppState, ColorOverrides, NoteStatus, Ports, SavedFilter, Services, Settings, TodoRef, WatcherOptions,
};
use floetask_domain::date::parse_iso;
use floetask_domain::listing::Attribute;
use floetask_domain::{Date, TodoDocument};

use crate::app::{Dialog, Floetask, Message, NoteSave, SettingChange, Startup, ViewToggle};
use iced::widget::text_editor;

const TODO_PATH: &str = "/test/todo.txt";
const SAMPLE: &str = "\
(A) Call mom +family @phone due:2026-10-04
(B) Prepare slides +work due:2026-10-06 note:slides status:doing
Buy milk @errands note:shopping
x 2026-10-03 2026-10-01 Send invoice +work
Legal review +work status:waiting
Learn Rust status:someday
";

#[derive(Default)]
struct MemoryFiles(Mutex<HashMap<PathBuf, String>>);

impl FileSystem for MemoryFiles {
    fn read(&self, path: &Path) -> Result<String, AppError> {
        self.0
            .lock()
            .unwrap()
            .get(path)
            .cloned()
            .ok_or_else(|| AppError::io(path, "missing"))
    }
    fn write(&self, path: &Path, content: &str, _safe: bool) -> Result<(), AppError> {
        self.0.lock().unwrap().insert(path.to_owned(), content.to_owned());
        Ok(())
    }
    fn exists(&self, path: &Path) -> bool {
        self.0.lock().unwrap().contains_key(path)
    }
    fn remove(&self, path: &Path) -> Result<(), AppError> {
        self.0
            .lock()
            .unwrap()
            .remove(path)
            .map(drop)
            .ok_or_else(|| AppError::io(path, "missing"))
    }
}

struct Fixed;

impl Clock for Fixed {
    fn today(&self) -> Date {
        parse_iso("2026-10-04").unwrap()
    }
}

impl SettingsStore for Fixed {
    fn load(&self) -> Result<Settings, AppError> {
        Ok(Settings::default())
    }
    fn save(&self, _: &Settings) -> Result<(), AppError> {
        Ok(())
    }
    fn location(&self) -> PathBuf {
        PathBuf::from("/test/config.toml")
    }
}

impl StateStore for Fixed {
    fn load(&self) -> AppState {
        let mut state = AppState::default();
        state.open_file(PathBuf::from(TODO_PATH));
        state
    }
    fn save(&self, _: &AppState) -> Result<(), AppError> {
        Ok(())
    }
}

impl SavedFilterStore for Fixed {
    fn load(&self) -> Result<Vec<SavedFilter>, AppError> {
        Ok(Vec::new())
    }
    fn save(&self, _: &[SavedFilter]) -> Result<(), AppError> {
        Ok(())
    }
    fn location(&self) -> PathBuf {
        PathBuf::from("/test/filters.toml")
    }
}

#[derive(Default)]
struct Log(Mutex<HashSet<String>>);

impl NotificationLog for Log {
    fn contains(&self, key: &str) -> bool {
        self.0.lock().unwrap().contains(key)
    }
    fn insert(&self, key: &str) -> Result<(), AppError> {
        self.0.lock().unwrap().insert(key.to_owned());
        Ok(())
    }
}

impl Notifier for Fixed {
    fn notify(&self, _: &str, _: &str) -> Result<(), AppError> {
        Ok(())
    }
}

impl FileWatcher for Fixed {
    fn watch(
        &self,
        _: &[PathBuf],
        _: &WatcherOptions,
        _: Box<dyn Fn(PathBuf) + Send + Sync>,
    ) -> Result<WatchGuard, AppError> {
        Ok(WatchGuard(Box::new(())))
    }
}

impl Desktop for Fixed {
    fn open_uri(&self, _: &str) -> Result<(), AppError> {
        Ok(())
    }
    fn reveal(&self, _: &Path) -> Result<(), AppError> {
        Ok(())
    }
    fn open_file(&self, _: &Path) -> Result<(), AppError> {
        Ok(())
    }
}

impl ColorStore for Fixed {
    fn load(&self) -> Result<ColorOverrides, AppError> {
        Ok(ColorOverrides::default())
    }
    fn location(&self) -> PathBuf {
        PathBuf::from("/test/colors.toml")
    }
}

/// An app showing `SAMPLE`, as if the file had finished loading.
fn app() -> Floetask {
    let files = Arc::new(MemoryFiles::default());
    files.write(Path::new(TODO_PATH), SAMPLE, false).unwrap();
    files.write(Path::new("/test/config.toml"), "", false).unwrap();
    files
        .write(
            &note_path("slides.md"),
            "# Slides
",
            false,
        )
        .unwrap();
    let fixed = Arc::new(Fixed);
    let ports = Ports {
        files,
        clock: fixed.clone(),
        settings: fixed.clone(),
        state: fixed.clone(),
        saved_filters: fixed.clone(),
        notification_log: Arc::new(Log::default()),
        notifier: fixed.clone(),
        watcher: fixed.clone(),
        desktop: fixed.clone(),
        colors: fixed,
    };
    let (mut app, _) = Floetask::boot(Startup {
        services: Services::new(ports),
        open: None,
    });
    let _ = app.update(Message::FileLoaded(
        PathBuf::from(TODO_PATH),
        Ok(TodoDocument::parse(SAMPLE, &[])),
    ));
    app
}

/// Clicks the widget showing `label` and returns the messages it produced.
fn click(app: &Floetask, label: &str) -> Vec<Message> {
    let mut ui = simulator(app.view());
    ui.click(label).unwrap_or_else(|_| panic!("`{label}` is not on screen"));
    ui.into_messages().collect()
}

/// Writes a snapshot when `FLOETASK_SNAPSHOTS` names a directory.
fn snapshot(app: &Floetask, name: &str) {
    let Some(dir) = std::env::var_os("FLOETASK_SNAPSHOTS") else {
        return;
    };
    let path = PathBuf::from(dir).join(name);
    let _ = std::fs::remove_file(path.with_extension("png"));
    let mut ui = simulator(app.view());
    let snapshot = ui.snapshot(&app.theme()).unwrap();
    snapshot.matches_image(&path).unwrap();
}

#[test]
fn list_shows_grouped_todos() {
    let app = app();
    let mut ui = simulator(app.view());
    for text in [
        "Call mom",
        "Prepare slides",
        "Buy milk",
        "Legal review",
        "Doing",
        "To do",
        "Waiting",
        "Done",
        "family",
        "errands",
    ] {
        assert!(ui.find(text).is_ok(), "missing {text}");
    }
    assert!(ui.find("Learn Rust").is_err(), "someday is hidden by default");
    drop(ui);
    snapshot(&app, "list");
}

#[test]
fn someday_todos_show_on_request() {
    let mut app = app();
    let _ = app.update(Message::ViewToggle(ViewToggle::HiddenStatuses, true));
    let mut ui = simulator(app.view());
    assert!(ui.find("Learn Rust").is_ok());
    assert!(ui.find("Someday").is_ok());
}

#[test]
fn editor_status_picker_rewrites_the_status_token() {
    let mut app = app();
    let _ = app.update(Message::NewTodo);
    let _ = app.update(Message::EditorAction(text_editor::Action::Edit(
        text_editor::Edit::Paste(std::sync::Arc::new("Ask legal +work".to_owned())),
    )));
    let _ = app.update(Message::EditorStatus("waiting".to_owned()));
    let _ = app.update(Message::EditorStatus("doing".to_owned()));
    let Some(Dialog::Editor(editor)) = &app.dialog else {
        panic!("editor closed")
    };
    assert_eq!(editor.text(), "Ask legal +work status:doing");
    assert!(simulator(app.view()).find("Doing").is_ok());
    snapshot(&app, "editor-status");

    let _ = app.update(Message::EditorStatus("todo".to_owned()));
    let Some(Dialog::Editor(editor)) = &app.dialog else {
        panic!("editor closed")
    };
    assert_eq!(editor.text(), "Ask legal +work");
}

#[test]
fn clicking_a_chip_filters_the_list() {
    let mut app = app();
    let messages = click(&app, "family");
    assert!(matches!(&messages[..], [Message::ChipFilter(Attribute::Projects, value)] if value == "family"));
    for message in messages {
        let _ = app.update(message);
    }
    assert_eq!(app.listing.counts.visible, 1);
}

#[test]
fn new_todo_dialog_autocompletes_projects() {
    let mut app = app();
    let _ = app.update(Message::NewTodo);
    assert!(matches!(app.dialog, Some(Dialog::Editor(_))));

    let mut ui = simulator(app.view());
    ui.click(iced::widget::Id::from(crate::view::EDITOR_ID))
        .expect("editor on screen");
    ui.typewrite("Plan +wo");
    let messages: Vec<Message> = ui.into_messages().collect();
    for message in messages {
        let _ = app.update(message);
    }
    let Some(Dialog::Editor(editor)) = &app.dialog else {
        panic!("editor closed")
    };
    assert_eq!(editor.text(), "Plan +wo");
    assert_eq!(editor.suggestions, vec!["work".to_owned()]);
    snapshot(&app, "editor");

    let _ = app.update(Message::EditorAccept);
    let Some(Dialog::Editor(editor)) = &app.dialog else {
        panic!("editor closed")
    };
    assert_eq!(editor.text(), "Plan +work ");
}

#[test]
fn escape_closes_dialog_then_drawer() {
    let mut app = app();
    let _ = app.update(Message::ToggleDrawer);
    let _ = app.update(Message::OpenSettings);
    snapshot(&app, "settings");
    let escape = || {
        Message::KeyPressed(
            iced::keyboard::Key::Named(iced::keyboard::key::Named::Escape),
            Default::default(),
        )
    };
    let _ = app.update(escape());
    assert!(app.dialog.is_none());
    assert!(app.state.drawer_open);
    snapshot(&app, "drawer");
    let _ = app.update(escape());
    assert!(!app.state.drawer_open);
}

#[test]
fn search_expressions_filter_and_keep_last_valid_result() {
    let mut app = app();
    let _ = app.update(Message::SearchInput("+work and due:".into()));
    assert_eq!(app.listing.counts.visible, 1);
    // Unfinished expression: keep the previous result.
    let _ = app.update(Message::SearchInput("+work and due: <".into()));
    assert_eq!(app.listing.counts.visible, 1);
    let _ = app.update(Message::SearchInput("milk".into()));
    assert_eq!(app.listing.counts.visible, 1);
}

#[test]
fn keyboard_moves_selection() {
    let mut app = app();
    let down = || {
        Message::KeyPressed(
            iced::keyboard::Key::Named(iced::keyboard::key::Named::ArrowDown),
            Default::default(),
        )
    };
    let _ = app.update(down());
    let _ = app.update(down());
    assert_eq!(app.selected, Some(1));
    assert_eq!(app.selected_todo().unwrap().todo.body(), "Call mom");
}

#[test]
fn renders_both_themes() {
    let mut app = app();
    let _ = app.update(Message::ToggleDrawer);
    snapshot(&app, "light");
    app.settings.theme = floetask_application::ThemePreference::Dark;
    assert!(app.colors().dark);
    snapshot(&app, "dark");
    app.state.files[0].done_path = Some(PathBuf::from("/test/done.txt"));
    app.state
        .files
        .push(floetask_application::TodoFileEntry::new(PathBuf::from(
            "/test/work.txt",
        )));
    snapshot(&app, "dark-files");
    let _ = app.update(Message::FileMenu(Some(0)));
    snapshot(&app, "dark-file-menu");
    let _ = app.update(Message::NewTodo);
    snapshot(&app, "dark-editor");
}

#[test]
fn file_menu_floats_and_closes_on_outside_click() {
    let mut app = app();
    let _ = app.update(Message::FileMenu(Some(0)));
    let mut ui = simulator(app.view());
    // A click on the list, outside the menu, dismisses it.
    ui.click("Buy milk").unwrap();
    let messages: Vec<Message> = ui.into_messages().collect();
    assert!(messages.iter().any(|m| matches!(m, Message::FileMenu(None))));
}

#[test]
fn editor_date_picker_floats_over_the_dialog() {
    let mut app = app();
    let _ = app.update(Message::NewTodo);
    for message in click(&app, "Due") {
        let _ = app.update(message);
    }
    let editor_calendar = |app: &Floetask| match &app.dialog {
        Some(Dialog::Editor(editor)) => editor.calendar.as_ref().map(|calendar| calendar.key),
        _ => panic!("editor closed"),
    };
    assert_eq!(editor_calendar(&app), Some(crate::app::DateKey::Due));
    // The repeat row stays where it was, under the floating calendar.
    assert!(simulator(app.view()).find("Repeat").is_ok());
    snapshot(&app, "editor-date-picker");

    // A click outside the calendar closes it and keeps the dialog.
    for message in click(&app, "Priority") {
        let _ = app.update(message);
    }
    assert_eq!(editor_calendar(&app), None);

    // Escape closes the calendar first, then the dialog.
    let _ = app.update(Message::EditorOpenCalendar(crate::app::DateKey::Threshold));
    let escape = Message::KeyPressed(
        iced::keyboard::Key::Named(iced::keyboard::key::Named::Escape),
        Default::default(),
    );
    let _ = app.update(escape.clone());
    assert_eq!(editor_calendar(&app), None);
    let _ = app.update(escape);
    assert!(app.dialog.is_none());
}

/// Toasts schedule their dismissal on tokio; tests that show one need a
/// runtime to enter.
fn runtime() -> tokio::runtime::Runtime {
    tokio::runtime::Builder::new_current_thread()
        .enable_time()
        .build()
        .unwrap()
}

/// Where the sample file's note `name` lives.
fn note_path(name: &str) -> PathBuf {
    Path::new(TODO_PATH).parent().unwrap().join("todo-notes").join(name)
}

#[test]
fn notes_link_open_and_warn_when_orphaned() {
    let runtime = runtime();
    let _tokio = runtime.enter();
    let mut app = app();
    assert_eq!(app.note_statuses["slides"], NoteStatus::Present);
    assert_eq!(app.note_statuses["shopping"], NoteStatus::Missing);
    snapshot(&app, "notes");

    // A broken link still opens: the note is created, then shown as present.
    let messages = click(&app, "shopping");
    let [Message::OpenNote(target)] = &messages[..] else {
        panic!("expected OpenNote, got {messages:?}")
    };
    let todo = floetask_domain::Todo::parse(&target.raw);
    let opened = app.services.notes.open(Path::new(TODO_PATH), &todo);
    assert_eq!(opened, Ok(note_path("shopping.md")));
    assert_eq!(
        app.services.ports.files.read(&note_path("shopping.md")).unwrap(),
        "# Buy milk
"
    );
    let _ = app.update(Message::NoteOpened(opened));
    assert_eq!(app.note_statuses["shopping"], NoteStatus::Present);

    // Deleting the only todo linking a note warns and keeps the file.
    let _ = app.update(Message::AskDelete(TodoRef::new(2, &todo)));
    let _ = app.update(Message::ConfirmDialog);
    assert!(
        app.toasts
            .iter()
            .any(|toast| toast.text.contains("todo-notes/shopping.md"))
    );
    assert!(app.services.ports.files.exists(&note_path("shopping.md")));
}

/// Puts `text` in the open dialog's notes field.
fn type_note(app: &mut Floetask, text: &str) {
    if let Some(Dialog::Editor(editor)) = &mut app.dialog {
        editor.note = iced::widget::text_editor::Content::with_text(text);
    }
}

/// What saving the open dialog writes: the todo text it would save.
fn save_notes(app: &Floetask) -> Result<String, AppError> {
    let Some(Dialog::Editor(editor)) = &app.dialog else {
        panic!("editor closed")
    };
    let note = NoteSave::from_editor(editor).map_err(AppError::from)?;
    note.write(&app.services.notes, Path::new(TODO_PATH), editor.text())
}

#[test]
fn notes_typed_in_a_new_todo_create_the_file_and_tag() {
    let mut app = app();
    let _ = app.update(Message::NewTodo);
    if let Some(Dialog::Editor(editor)) = &mut app.dialog {
        editor.set_text("(A) Write report +work");
    }
    assert_eq!(save_notes(&app), Ok("(A) Write report +work".to_owned()));

    type_note(&mut app, "# Outline\n- intro");
    snapshot(&app, "editor-note");
    assert_eq!(
        save_notes(&app),
        Ok("(A) Write report +work note:write-report.md".to_owned())
    );
    let files = &app.services.ports.files;
    assert_eq!(
        files.read(&note_path("write-report.md")).unwrap(),
        "# Outline\n- intro\n"
    );
    // Another todo with the same text never overwrites that note.
    assert_eq!(
        save_notes(&app),
        Ok("(A) Write report +work note:write-report-2.md".to_owned())
    );
}

#[test]
fn editing_a_todo_loads_and_saves_its_note() {
    let mut app = app();
    let slides = app
        .listing
        .todos()
        .find(|entry| entry.todo.note() == Some("slides"))
        .unwrap();
    let target = TodoRef::new(slides.line, &slides.todo);
    let _ = app.update(Message::OpenTodo(target.clone()));
    let _ = app.update(Message::NoteLoaded(target, Ok(Some("# Slides\n".to_owned()))));
    let Some(Dialog::Editor(editor)) = &app.dialog else {
        panic!("editor closed")
    };
    assert_eq!(editor.note_text(), "# Slides");
    let text = editor.text();
    // Unchanged notes are not written back.
    assert_eq!(save_notes(&app), Ok(text.clone()));

    type_note(&mut app, "# Slides\nAgenda first");
    assert_eq!(save_notes(&app), Ok(text));
    assert_eq!(
        app.services.ports.files.read(&note_path("slides.md")).unwrap(),
        "# Slides\nAgenda first\n"
    );
}

#[test]
fn notes_are_not_saved_under_an_invalid_name() {
    let mut app = app();
    let _ = app.update(Message::NewTodo);
    if let Some(Dialog::Editor(editor)) = &mut app.dialog {
        editor.set_text("Task note:Bad");
    }
    type_note(&mut app, "text");
    assert!(matches!(save_notes(&app), Err(AppError::Note(_))));
}

fn lane(group: usize, lane: usize) -> floetask_domain::listing::LaneId {
    floetask_domain::listing::LaneId { group, lane }
}

fn board_lane_bodies(app: &Floetask) -> Vec<Vec<String>> {
    assert_eq!(app.board.groups.len(), 1, "board should not be grouped");
    app.board.groups[0]
        .lanes
        .iter()
        .map(|lane| lane.todos.iter().map(|t| t.todo.body().to_owned()).collect())
        .collect()
}

#[test]
fn board_shows_a_column_per_status() {
    let mut app = app();
    let _ = app.update(Message::ToggleMainView);
    assert_eq!(app.state.main_view, floetask_application::MainView::Board);
    assert_eq!(
        board_lane_bodies(&app),
        vec![
            vec!["Call mom", "Buy milk"],
            vec!["Prepare slides"],
            vec!["Legal review"],
            vec!["Send invoice"],
        ]
    );
    let mut ui = simulator(app.view());
    for text in ["TO DO", "DOING", "WAITING", "DONE", "Columns", "Learn Rust"] {
        assert_eq!(ui.find(text).is_ok(), text != "Learn Rust", "{text}");
    }
    drop(ui);
    snapshot(&app, "board");
}

#[test]
fn dragging_a_card_moves_it_to_another_column() {
    let mut app = app();
    let _ = app.update(Message::ToggleMainView);
    let messages = click(&app, "Buy milk");
    assert!(
        messages
            .iter()
            .any(|m| matches!(m, Message::BoardPress(target, id) if target.line == 2 && *id == lane(0, 0)))
    );
    // Keep the button held: drop the release the click also produced.
    for message in messages.into_iter().filter(|m| !matches!(m, Message::BoardRelease)) {
        let _ = app.update(message);
    }
    // Entering the next column can arrive before leaving the first.
    let _ = app.update(Message::BoardHover(lane(0, 1), true));
    let _ = app.update(Message::BoardHover(lane(0, 0), false));
    assert_eq!(app.board_hover, Some(lane(0, 1)));
    snapshot(&app, "board-drag");

    let target = app.drag.as_ref().unwrap().target.clone();
    let _ = app.update(Message::BoardRelease);
    assert!(app.drag.is_none());
    // The drop runs the move in the background; run it here and feed the result back.
    let moved = app.services.todo_files.move_to_column(
        Path::new(TODO_PATH),
        &target,
        &floetask_domain::board::Column::Status("doing".to_owned()),
        &app.file_options(),
    );
    let _ = app.update(Message::Saved(PathBuf::from(TODO_PATH), moved));
    assert_eq!(board_lane_bodies(&app)[1], vec!["Prepare slides", "Buy milk"]);
}

#[test]
fn pressing_and_releasing_in_place_opens_the_todo() {
    let mut app = app();
    let _ = app.update(Message::ToggleMainView);
    let messages = click(&app, "Legal review");
    assert!(matches!(messages.last(), Some(Message::BoardRelease)));
    for message in messages {
        let _ = app.update(message);
    }
    assert!(matches!(app.dialog, Some(Dialog::Editor(_))));
}

#[test]
fn board_columns_are_set_per_file() {
    let mut app = app();
    let _ = app.update(Message::ToggleMainView);
    let _ = app.update(Message::OpenBoardColumns);
    let _ = app.update(Message::BoardColumnAdd(Some("someday".to_owned())));
    let _ = app.update(Message::BoardColumnInput("In Review".to_owned()));
    let _ = app.update(Message::BoardColumnAdd(None));
    let _ = app.update(Message::BoardColumnRemove(2));
    let _ = app.update(Message::BoardColumnMove(4, -1));
    snapshot(&app, "board-columns");
    let _ = app.update(Message::ConfirmDialog);
    assert!(app.dialog.is_none());
    assert_eq!(
        app.settings.board_columns(Path::new(TODO_PATH)).keys(),
        ["todo", "doing", "done", "in-review", "someday"]
    );
    assert_eq!(board_lane_bodies(&app)[4], vec!["Learn Rust"]);
    // The column's status is offered for this file only, not added globally.
    assert!(!app.settings.statuses.names().contains(&"in-review".to_owned()));
    assert_eq!(app.status_options().last().map(String::as_str), Some("in-review"));
    assert_eq!(board_lane_bodies(&app)[5], vec!["Legal review"]);
}

#[test]
fn settings_sidebar_switches_sections_and_describes_settings() {
    let mut app = app();
    let _ = app.update(Message::OpenSettings);
    let mut ui = simulator(app.view());
    for text in ["Todos", "Dates", "Statuses", "Appearance", "Notifications", "Files"] {
        assert!(ui.find(text).is_ok(), "missing section {text}");
    }
    assert!(ui.find("In the add dialog, every line becomes its own todo.").is_ok());
    drop(ui);
    let messages = click(&app, "Appearance");
    assert!(
        messages
            .iter()
            .any(|m| matches!(m, Message::SettingsSection(crate::app::SettingsSection::Appearance)))
    );
    for message in messages {
        let _ = app.update(message);
    }
    assert!(
        simulator(app.view())
            .find("Light, dark, or follow your system.")
            .is_ok()
    );
    snapshot(&app, "settings-appearance");
}

#[test]
fn statuses_are_managed_in_settings() {
    let mut app = app();
    let _ = app.update(Message::OpenSettings);
    let _ = app.update(Message::SettingsSection(crate::app::SettingsSection::Statuses));
    let _ = app.update(Message::SettingsStatusInput("In Review".to_owned()));
    let _ = app.update(Message::Setting(SettingChange::StatusAdd("in-review".to_owned())));
    let _ = app.update(Message::Setting(SettingChange::StatusMove(4, -1)));
    let _ = app.update(Message::Setting(SettingChange::StatusHidden(
        "waiting".to_owned(),
        true,
    )));
    snapshot(&app, "settings-statuses");
    assert_eq!(
        app.settings.statuses.names(),
        ["doing", "todo", "waiting", "in-review", "someday"]
    );
    assert!(app.listing.todos().all(|t| t.todo.status() != Some("waiting")));
    assert!(app.status_options().contains(&"in-review".to_owned()));
    let Some(Dialog::Settings(dialog)) = &app.dialog else {
        panic!("settings closed")
    };
    assert!(dialog.status_input.is_empty());
    let _ = app.update(Message::Setting(SettingChange::StatusRemove("in-review".to_owned())));
    let _ = app.update(Message::Setting(SettingChange::StatusRemove("doing".to_owned())));
    assert_eq!(app.settings.statuses.names(), ["doing", "todo", "waiting", "someday"]);
}

#[test]
fn grouping_gives_each_group_its_own_board() {
    let mut app = app();
    let _ = app.update(Message::ToggleMainView);
    assert!(!app.board.is_grouped());
    // Sort by priority first: the board splits into one board per priority.
    let _ = app.update(Message::MoveSort(0, 1));
    assert!(app.board.is_grouped());
    let groups: Vec<String> = app.board.groups.iter().map(|g| g.values.join(",")).collect();
    assert_eq!(groups, ["A", "B", ""]);
    assert!(simulator(app.view()).find("(A)").is_ok());
    snapshot(&app, "board-grouped");

    // Dropping on the same column of another group changes nothing.
    let call_mom = app.board.groups[0].lanes[0].todos[0].clone();
    assert_eq!(call_mom.todo.body(), "Call mom");
    let target = TodoRef::new(call_mom.line, &call_mom.todo);
    let _ = app.update(Message::BoardPress(target, lane(0, 0)));
    let _ = app.update(Message::BoardHover(lane(2, 0), true));
    let _ = app.update(Message::BoardRelease);
    assert!(app.drag.is_none());
    assert!(app.dialog.is_none());
}

#[test]
fn grouping_switch_keeps_sorting_on_one_board() {
    let mut app = app();
    let _ = app.update(Message::ToggleMainView);
    let _ = app.update(Message::MoveSort(0, 1));
    assert!(app.board.is_grouped());
    assert!(simulator(app.view()).find("Group by priority").is_ok());

    let _ = app.update(Message::BoardGrouped(false));
    assert!(!app.state.board_grouped);
    assert!(!app.board.is_grouped());
    // Still sorted by priority inside the column.
    assert_eq!(board_lane_bodies(&app)[0], vec!["Call mom", "Buy milk"]);
    snapshot(&app, "board-ungrouped");
}
