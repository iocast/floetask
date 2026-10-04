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
use floetask_application::{AppError, AppState, ColorOverrides, Ports, SavedFilter, Services, Settings, WatcherOptions};
use floetask_domain::date::parse_iso;
use floetask_domain::listing::Attribute;
use floetask_domain::{Date, TodoDocument};

use crate::app::{Dialog, Floetask, Message, Startup};

const TODO_PATH: &str = "/test/todo.txt";
const SAMPLE: &str = "\
(A) Call mom +family @phone due:2026-10-04
(B) Prepare slides +work due:2026-10-06
Buy milk @errands
x 2026-10-03 2026-10-01 Send invoice +work
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
        "(A)",
        "(B)",
        "+family",
        "@errands",
    ] {
        assert!(ui.find(text).is_ok(), "missing {text}");
    }
    drop(ui);
    snapshot(&app, "list");
}

#[test]
fn clicking_a_chip_filters_the_list() {
    let mut app = app();
    let messages = click(&app, "+family");
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
    assert_eq!(app.selected_todo().unwrap().todo.body(), "Prepare slides");
}

#[test]
fn renders_both_themes() {
    let mut app = app();
    let _ = app.update(Message::ToggleDrawer);
    snapshot(&app, "light");
    app.settings.theme = floetask_application::ThemePreference::Dark;
    assert!(app.colors().dark);
    snapshot(&app, "dark");
    let _ = app.update(Message::FileMenu(Some(0)));
    snapshot(&app, "dark-file-menu");
    let _ = app.update(Message::NewTodo);
    snapshot(&app, "dark-editor");
}
