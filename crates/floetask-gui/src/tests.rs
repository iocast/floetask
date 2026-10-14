//! Headless GUI tests: render views in iced's simulator, interact, and feed
//! the produced messages back into `update`.
//!
//! Set `FLOETASK_SNAPSHOTS=<dir>` to also write PNG snapshots of each screen
//! for a visual check.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use floetask_application::ports::*;
use floetask_application::{
    AppError, AppState, ColorOverrides, NoteStatus, Ports, SavedFilter, Services, Settings, TodoRef, WatcherOptions,
};
use floetask_domain::date::parse_iso;
use floetask_domain::listing::Attribute;
use floetask_domain::{Date, TodoDocument};

use iced::widget::text_editor;

use crate::app::{Dialog, Floetask, Message, Startup};
use crate::drawer::ViewToggle;
use crate::editor::NoteSave;
use crate::settings::SettingChange;
use crate::{board, calendar, drawer, editor, files, list, search, settings};

const TODO_PATH: &str = "/test/todo.txt";
const SAMPLE: &str = "\
(A) Call mom +family @phone due:2026-10-04
(B) Prepare slides +work due:2026-10-06 note:slides status:doing
Buy milk note:shopping
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

impl SystemLocale for Fixed {
    fn language(&self) -> Option<String> {
        Some("en-GB".to_owned())
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

impl Updater for Fixed {
    fn is_configured(&self) -> bool {
        true
    }
    /// A preview newer than the running build, and older releases.
    fn releases(&self) -> Result<Vec<PublishedRelease>, AppError> {
        let release = |version: &str, newer: bool, notes: Option<&str>| PublishedRelease {
            version: version.to_owned(),
            prerelease: version.contains("-rc."),
            notes: notes.map(str::to_owned),
            newer,
        };
        Ok(vec![
            release("9.0.0-rc.1", true, Some("Faster board")),
            release("0.3.1", false, None),
            release("0.3.1-rc.1", false, None),
            release("0.3.0", false, None),
        ])
    }
    fn install(&self, _version: &str) -> Result<(), AppError> {
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
        locale: fixed.clone(),
        settings: fixed.clone(),
        state: fixed.clone(),
        saved_filters: fixed.clone(),
        notification_log: Arc::new(Log::default()),
        notifier: fixed.clone(),
        watcher: fixed.clone(),
        desktop: fixed.clone(),
        colors: fixed.clone(),
        updater: fixed,
    };
    let (mut app, _) = Floetask::boot(Startup {
        services: Services::new(ports),
        open: None,
    });
    let _ = app.update(Message::Files(files::Message::Loaded(
        PathBuf::from(TODO_PATH),
        Ok(TodoDocument::parse(SAMPLE, &[])),
    )));
    // Wide enough for the main view and every side panel.
    app.state.window.width = 1800.0;
    app
}

/// Clicks the widget showing `label` and returns the messages it produced.
fn click(app: &Floetask, label: &str) -> Vec<Message> {
    let mut ui = simulator(app.view());
    ui.click(label).unwrap_or_else(|_| panic!("`{label}` is not on screen"));
    ui.into_messages().collect()
}

/// Writes a snapshot when `FLOETASK_SNAPSHOTS` names a directory.
/// iced's simulator with the fonts the notes editor measures text with.
fn simulator<'a>(
    element: impl Into<iced::Element<'a, Message>>,
) -> iced_test::Simulator<'a, Message, iced::Theme, iced::Renderer> {
    let settings = iced::Settings {
        fonts: iced_markdown_editor::FONTS.iter().map(|&font| font.into()).collect(),
        ..iced::Settings::default()
    };
    iced_test::Simulator::with_settings(settings, element)
}

fn snapshot(app: &Floetask, name: &str) {
    snapshot_view(app, app.view(), name);
}

/// Like [`snapshot`], for a view other than the main window's.
fn snapshot_view(app: &Floetask, view: iced::Element<'_, Message>, name: &str) {
    let Some(dir) = std::env::var_os("FLOETASK_SNAPSHOTS") else {
        return;
    };
    let path = PathBuf::from(dir).join(name);
    let _ = std::fs::remove_file(path.with_extension("png"));
    let mut ui = simulator(view);
    let snapshot = ui.snapshot(&app.theme()).unwrap();
    snapshot.matches_image(&path).unwrap();
}

#[test]
fn list_shows_grouped_todos() {
    let app = app();
    let mut ui = simulator(app.view());
    for text in [
        "Prepare slides",
        "Buy milk",
        "Legal review",
        "Doing",
        "To do",
        "Waiting",
        "Done",
        "family",
    ] {
        assert!(ui.find(text).is_ok(), "missing {text}");
    }
    // "Call mom @phone" is rich text, which the simulator cannot search.
    assert!(
        app.listing
            .todos()
            .any(|entry| entry.todo.display_text() == "Call mom @phone")
    );
    assert!(ui.find("Learn Rust").is_err(), "someday is hidden by default");
    drop(ui);
    snapshot(&app, "list");
}

#[test]
fn someday_todos_show_on_request() {
    let mut app = app();
    let _ = app.update(Message::Drawer(drawer::Message::ViewToggle(
        ViewToggle::HiddenStatuses,
        true,
    )));
    let mut ui = simulator(app.view());
    assert!(ui.find("Learn Rust").is_ok());
    assert!(ui.find("Someday").is_ok());
}

#[test]
fn editor_status_picker_rewrites_the_status_token() {
    let mut app = app();
    let _ = app.update(Message::Editor(editor::Message::New));
    let _ = app.update(Message::Editor(editor::Message::Action(text_editor::Action::Edit(
        text_editor::Edit::Paste(std::sync::Arc::new("Ask legal +work".to_owned())),
    ))));
    let _ = app.update(Message::Editor(editor::Message::Status("waiting".to_owned())));
    let _ = app.update(Message::Editor(editor::Message::Status("doing".to_owned())));
    let Some(Dialog::Editor(editor)) = &app.dialog else {
        panic!("editor closed")
    };
    assert_eq!(editor.text(), "Ask legal +work status:doing");
    assert!(simulator(app.view()).find("Doing").is_ok());
    snapshot(&app, "editor-status");

    let _ = app.update(Message::Editor(editor::Message::Status("todo".to_owned())));
    let Some(Dialog::Editor(editor)) = &app.dialog else {
        panic!("editor closed")
    };
    assert_eq!(editor.text(), "Ask legal +work");
}

#[test]
fn clicking_a_chip_filters_the_list() {
    let mut app = app();
    let messages = click(&app, "family");
    assert!(
        matches!(&messages[..], [Message::List(list::Message::ChipFilter(Attribute::Projects, value))] if value == "family")
    );
    for message in messages {
        let _ = app.update(message);
    }
    assert_eq!(app.listing.counts.visible, 1);
}

#[test]
fn new_todo_dialog_autocompletes_projects() {
    let mut app = app();
    let _ = app.update(Message::Editor(editor::Message::New));
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

    let _ = app.update(Message::Editor(editor::Message::Accept));
    let Some(Dialog::Editor(editor)) = &app.dialog else {
        panic!("editor closed")
    };
    assert_eq!(editor.text(), "Plan +work ");
}

#[test]
fn escape_closes_dialog_then_drawer() {
    let mut app = app();
    let _ = app.update(Message::Drawer(drawer::Message::Toggle));
    let _ = app.update(Message::Settings(settings::Message::Open));
    snapshot(&app, "settings");
    let escape = || {
        Message::KeyPressed(
            iced::window::Id::unique(),
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
    let _ = app.update(Message::Search(search::Message::Input("+work and due:".into())));
    assert_eq!(app.listing.counts.visible, 1);
    // Unfinished expression: keep the previous result.
    let _ = app.update(Message::Search(search::Message::Input("+work and due: <".into())));
    assert_eq!(app.listing.counts.visible, 1);
    let _ = app.update(Message::Search(search::Message::Input("milk".into())));
    assert_eq!(app.listing.counts.visible, 1);
}

#[test]
fn keyboard_moves_selection() {
    let mut app = app();
    let down = || {
        Message::KeyPressed(
            iced::window::Id::unique(),
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
fn switching_the_language_changes_every_screen_at_once() {
    let mut app = app();
    let _ = app.update(Message::Settings(settings::Message::Change(SettingChange::Language(
        "de".to_owned(),
    ))));
    assert_eq!(app.settings.language, "de");
    let mut ui = simulator(app.view());
    for text in ["In Arbeit", "Offen", "Wartet", "Erledigt", "Prepare slides"] {
        assert!(ui.find(text).is_ok(), "missing {text}");
    }
    assert!(ui.find("Doing").is_err(), "English left on the list");
    drop(ui);
    snapshot(&app, "de-list");

    let _ = app.update(Message::Drawer(drawer::Message::Toggle));
    let _ = app.update(Message::Settings(settings::Message::Open));
    assert!(simulator(app.view()).find("Einstellungen").is_ok());
    snapshot(&app, "de-settings");
    let _ = app.update(Message::CloseDialog);

    // Today is 2026-10-04.
    let _ = app.update(Message::ShowMainView(floetask_application::MainView::Calendar));
    assert!(simulator(app.view()).find("Oktober 2026").is_ok());
    snapshot(&app, "de-calendar");

    let _ = app.update(Message::Settings(settings::Message::Change(SettingChange::Language(
        "system".to_owned(),
    ))));
    assert!(simulator(app.view()).find("October 2026").is_ok());
}

#[test]
fn renders_both_themes() {
    let mut app = app();
    let _ = app.update(Message::Drawer(drawer::Message::Toggle));
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
    let _ = app.update(Message::Files(files::Message::Menu(Some(0))));
    snapshot(&app, "dark-file-menu");
    let _ = app.update(Message::Editor(editor::Message::New));
    snapshot(&app, "dark-editor");
}

#[test]
fn file_menu_floats_and_closes_on_outside_click() {
    let mut app = app();
    let _ = app.update(Message::Files(files::Message::Menu(Some(0))));
    let mut ui = simulator(app.view());
    // A click on the list, outside the menu, dismisses it.
    ui.click("Buy milk").unwrap();
    let messages: Vec<Message> = ui.into_messages().collect();
    assert!(
        messages
            .iter()
            .any(|m| matches!(m, Message::Files(files::Message::Menu(None))))
    );
}

#[test]
fn editor_date_picker_floats_over_the_dialog() {
    let mut app = app();
    let _ = app.update(Message::Editor(editor::Message::New));
    for message in click(&app, "Due") {
        let _ = app.update(message);
    }
    let editor_calendar = |app: &Floetask| match &app.dialog {
        Some(Dialog::Editor(editor)) => editor.calendar.as_ref().map(|calendar| calendar.key),
        _ => panic!("editor closed"),
    };
    assert_eq!(editor_calendar(&app), Some(crate::editor::DateKey::Due));
    // The repeat row stays where it was, under the floating calendar.
    assert!(simulator(app.view()).find("Repeat").is_ok());
    snapshot(&app, "editor-date-picker");

    // A click outside the calendar closes it and keeps the dialog.
    for message in click(&app, "Priority") {
        let _ = app.update(message);
    }
    assert_eq!(editor_calendar(&app), None);

    // Escape closes the calendar first, then the dialog.
    let _ = app.update(Message::Editor(editor::Message::OpenDatePicker(
        crate::editor::DateKey::Threshold,
    )));
    let escape = Message::KeyPressed(
        iced::window::Id::unique(),
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

/// Feeds `message` to `update`, runs the tasks it returns on `runtime` and
/// feeds their messages back in, until nothing is left. Window and widget
/// operations have no window to act on here and are dropped.
fn drive(runtime: &tokio::runtime::Runtime, app: &mut Floetask, message: Message) {
    use iced_test::futures::futures::StreamExt;
    let mut pending = vec![message];
    while let Some(message) = pending.pop() {
        let Some(stream) = iced_test::runtime::task::into_stream(app.update(message)) else {
            continue;
        };
        let outputs: Vec<Message> = runtime.block_on(
            stream
                .filter_map(|action| async move {
                    match action {
                        iced_test::runtime::Action::Output(message) => Some(message),
                        _ => None,
                    }
                })
                .collect(),
        );
        pending.extend(outputs.into_iter().rev());
    }
}

/// The note window's text, without the final newline.
fn note_window_text(app: &Floetask) -> String {
    app.note_window
        .as_ref()
        .expect("note window open")
        .note
        .text()
        .to_owned()
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
    let [Message::List(list::Message::OpenNote(target))] = &messages[..] else {
        panic!("expected OpenNote, got {messages:?}")
    };
    let todo = floetask_domain::Todo::parse(&target.raw);
    drive(
        &runtime,
        &mut app,
        Message::List(list::Message::OpenNote(target.clone())),
    );
    assert_eq!(
        app.services.ports.files.read(&note_path("shopping.md")).unwrap(),
        "# Buy milk\n"
    );
    assert_eq!(app.note_statuses["shopping"], NoteStatus::Present);

    // Deleting the only todo linking a note warns and keeps the file.
    let _ = app.update(Message::List(list::Message::AskDelete(TodoRef::new(2, &todo))));
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
        editor.note.replace(text);
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
    let _ = app.update(Message::Editor(editor::Message::New));
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
    // The editor saves Markdown in its own canonical form.
    let files = &app.services.ports.files;
    assert_eq!(
        files.read(&note_path("write-report.md")).unwrap(),
        "# Outline\n\n- intro\n"
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
    let _ = app.update(Message::List(list::Message::OpenTodo(target.clone())));
    let _ = app.update(Message::Editor(editor::Message::NoteLoaded(
        target,
        Ok(Some("# Slides\n".to_owned())),
    )));
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
        "# Slides\n\nAgenda first\n"
    );
}

/// The open dialog's editor.
fn editor(app: &Floetask) -> &crate::editor::Editor {
    match &app.dialog {
        Some(Dialog::Editor(editor)) => editor,
        _ => panic!("editor closed"),
    }
}

#[test]
fn picked_files_are_moved_into_the_notes_folder_or_linked_in_place() {
    let runtime = runtime();
    let _tokio = runtime.enter();
    let mut app = app();
    let files = app.services.ports.files.clone();
    let _ = app.update(Message::Editor(editor::Message::New));
    if let Some(Dialog::Editor(editor)) = &mut app.dialog {
        editor.set_text("Plan the offsite");
    }

    // A file outside the notes folder asks before it is linked.
    let plan = PathBuf::from("/elsewhere/My Plan.md");
    files.write(&plan, "# Agenda\n\n- budget\n", false).unwrap();
    let _ = app.update(Message::Editor(editor::Message::NoteFilePicked(Some(plan.clone()))));
    assert_eq!(editor(&app).pending_link, Some(plan.clone()));
    snapshot(&app, "editor-link-question");

    // Moving it gives it a valid name in the notes folder.
    drive(&runtime, &mut app, Message::Editor(editor::Message::MovePickedNote));
    assert_eq!(editor(&app).text(), "Plan the offsite note:my-plan.md");
    assert_eq!(editor(&app).note_text(), "# Agenda\n\n- budget");
    assert_eq!(editor(&app).pending_link, None);
    assert!(!files.exists(&plan));
    assert_eq!(files.read(&note_path("my-plan.md")).unwrap(), "# Agenda\n\n- budget\n");
    // An untouched linked note is not rewritten.
    assert_eq!(save_notes(&app), Ok("Plan the offsite note:my-plan.md".to_owned()));

    // Keeping it links the file where it is, by its path.
    let kept = PathBuf::from("/elsewhere/Keep Me.md");
    files.write(&kept, "# Kept\n", false).unwrap();
    let _ = app.update(Message::Editor(editor::Message::NoteFilePicked(Some(kept.clone()))));
    drive(&runtime, &mut app, Message::Editor(editor::Message::KeepPickedNote));
    assert_eq!(editor(&app).text(), "Plan the offsite note:/elsewhere/Keep%20Me.md");
    assert_eq!(editor(&app).note_text(), "# Kept");
    assert!(files.exists(&kept));

    // A file already in the notes folder links without asking.
    drive(
        &runtime,
        &mut app,
        Message::Editor(editor::Message::NoteFilePicked(Some(note_path("slides.md")))),
    );
    assert_eq!(editor(&app).text(), "Plan the offsite note:slides.md");
    assert_eq!(editor(&app).pending_link, None);

    // Unlinking removes the tag and keeps the file.
    let _ = app.update(Message::Editor(editor::Message::UnlinkNote));
    assert_eq!(editor(&app).text(), "Plan the offsite");
    assert_eq!(editor(&app).note_text(), "");
    assert_eq!(save_notes(&app), Ok("Plan the offsite".to_owned()));
    assert!(files.exists(&note_path("slides.md")));
}

#[test]
fn the_note_window_saves_edits_and_is_reused() {
    let runtime = runtime();
    let _tokio = runtime.enter();
    let mut app = app();
    let files = app.services.ports.files.clone();
    let target = |app: &Floetask, note: &str| {
        let entry = app
            .listing
            .todos()
            .find(|entry| entry.todo.note() == Some(note))
            .unwrap();
        TodoRef::new(entry.line, &entry.todo)
    };

    let open = Message::List(list::Message::OpenNote(target(&app, "slides")));
    drive(&runtime, &mut app, open);
    let id = app.note_window.as_ref().unwrap().id;
    assert_eq!(app.window_title(id), "todo-notes/slides.md - floetask");
    assert_eq!(note_window_text(&app), "# Slides");
    if let Some(window) = &app.note_window {
        let mut ui = simulator(crate::note_window::view(&app, window));
        assert!(ui.find("todo-notes/slides.md").is_ok());
        assert!(ui.find("Saved").is_ok());
        snapshot_view(&app, crate::note_window::view(&app, window), "note-window");
    }

    // Edits are written on Ctrl+S (and once typing pauses).
    if let Some(window) = &mut app.note_window {
        window.note.replace("# Slides\n\nAgenda first");
    }
    let save = Message::KeyPressed(
        id,
        iced::keyboard::Key::Character("s".into()),
        iced::keyboard::Modifiers::CTRL,
    );
    drive(&runtime, &mut app, save);
    assert_eq!(
        files.read(&note_path("slides.md")).unwrap(),
        "# Slides\n\nAgenda first\n"
    );
    assert!(!app.note_window.as_ref().unwrap().note.changed());

    // Another note opens in the same window, after the first one is saved.
    if let Some(window) = &mut app.note_window {
        window.note.replace("# Slides\n\nAgenda last");
    }
    let open = Message::List(list::Message::OpenNote(target(&app, "shopping")));
    drive(&runtime, &mut app, open);
    assert_eq!(app.note_window.as_ref().unwrap().id, id);
    assert_eq!(note_window_text(&app), "# Buy milk");
    assert_eq!(
        files.read(&note_path("slides.md")).unwrap(),
        "# Slides\n\nAgenda last\n"
    );

    // Where the window is moved and resized is remembered for next time.
    let _ = app.update(Message::Window(
        id,
        iced::window::Event::Moved(iced::Point::new(40.0, 60.0)),
    ));
    let _ = app.update(Message::Window(
        id,
        iced::window::Event::Resized(iced::Size::new(640.0, 700.0)),
    ));
    let main = app.state.window;

    // Closing the window saves it too.
    if let Some(window) = &mut app.note_window {
        window.note.replace("# Buy milk\n\nOat milk");
    }
    drive(
        &runtime,
        &mut app,
        Message::Window(id, iced::window::Event::CloseRequested),
    );
    assert!(app.note_window.is_none());
    assert_eq!(
        files.read(&note_path("shopping.md")).unwrap(),
        "# Buy milk\n\nOat milk\n"
    );
    let last = app.state.note_window;
    assert_eq!((last.x, last.y), (Some(40.0), Some(60.0)));
    assert_eq!((last.width, last.height), (640.0, 700.0));
    // The main window keeps its own place.
    assert_eq!(app.state.window, main);
}

#[test]
fn the_dialog_hands_its_note_to_the_note_window() {
    let runtime = runtime();
    let _tokio = runtime.enter();
    let mut app = app();
    let files = app.services.ports.files.clone();
    let _ = app.update(Message::Editor(editor::Message::New));
    if let Some(Dialog::Editor(editor)) = &mut app.dialog {
        editor.set_text("Plan the offsite");
    }
    type_note(&mut app, "# Agenda");

    // The note gets its file and tag, and what was typed goes along and is
    // saved from the window once typing pauses.
    drive(&runtime, &mut app, Message::Editor(editor::Message::OpenNoteWindow));
    assert_eq!(editor(&app).text(), "Plan the offsite note:plan-the-offsite.md");
    assert_eq!(note_window_text(&app), "# Agenda");
    assert_eq!(files.read(&note_path("plan-the-offsite.md")).unwrap(), "# Agenda\n");
    assert!(app.dialog_note_in_window());
    snapshot(&app, "editor-note-in-window");
    assert!(
        simulator(app.view())
            .find("This note is open in its own window.")
            .is_ok()
    );

    // When the window closes, the dialog shows the note as saved there.
    if let Some(window) = &mut app.note_window {
        window.note.replace("# Agenda\n\n- budget");
    }
    let id = app.note_window.as_ref().unwrap().id;
    drive(
        &runtime,
        &mut app,
        Message::Window(id, iced::window::Event::CloseRequested),
    );
    assert!(!app.dialog_note_in_window());
    assert_eq!(editor(&app).note_text(), "# Agenda\n\n- budget");
    assert!(!editor(&app).note_changed());
}

#[test]
fn notes_are_not_saved_under_an_invalid_name() {
    let mut app = app();
    let _ = app.update(Message::Editor(editor::Message::New));
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
    assert!(messages.iter().any(
        |m| matches!(m, Message::Board(board::Message::Press(target, id)) if target.line == 2 && *id == lane(0, 0))
    ));
    // Keep the button held: drop the release the click also produced.
    for message in messages
        .into_iter()
        .filter(|m| !matches!(m, Message::Board(board::Message::Release)))
    {
        let _ = app.update(message);
    }
    // Entering the next column can arrive before leaving the first.
    let _ = app.update(Message::Board(board::Message::Hover(lane(0, 1), true)));
    let _ = app.update(Message::Board(board::Message::Hover(lane(0, 0), false)));
    assert_eq!(app.board_hover, Some(lane(0, 1)));
    snapshot(&app, "board-drag");

    let target = app.drag.as_ref().unwrap().target.clone();
    let _ = app.update(Message::Board(board::Message::Release));
    assert!(app.drag.is_none());
    // The drop runs the move in the background; run it here and feed the result back.
    let moved = app.services.todo_files.move_to_column(
        Path::new(TODO_PATH),
        &target,
        &floetask_domain::board::Column::Status("doing".to_owned()),
        &app.file_options(),
    );
    let _ = app.update(Message::Files(files::Message::Saved(PathBuf::from(TODO_PATH), moved)));
    assert_eq!(board_lane_bodies(&app)[1], vec!["Prepare slides", "Buy milk"]);
}

#[test]
fn pressing_and_releasing_in_place_opens_the_todo() {
    let mut app = app();
    let _ = app.update(Message::ToggleMainView);
    let messages = click(&app, "Legal review");
    assert!(matches!(messages.last(), Some(Message::Board(board::Message::Release))));
    for message in messages {
        let _ = app.update(message);
    }
    assert!(matches!(app.dialog, Some(Dialog::Editor(_))));
}

#[test]
fn dropping_a_card_back_in_its_column_does_not_open_it() {
    let mut app = app();
    let _ = app.update(Message::ToggleMainView);
    let messages = click(&app, "Legal review");
    for message in messages
        .into_iter()
        .filter(|m| !matches!(m, Message::Board(board::Message::Release)))
    {
        let _ = app.update(message);
    }
    // The cursor leaves the card and comes back to its column.
    let _ = app.update(Message::Board(board::Message::CardLeft));
    let _ = app.update(Message::Board(board::Message::Release));
    assert!(app.drag.is_none());
    assert!(app.dialog.is_none());
}

#[test]
fn board_columns_are_set_per_file() {
    let mut app = app();
    let _ = app.update(Message::ToggleMainView);
    let _ = app.update(Message::Board(board::Message::OpenColumns));
    let _ = app.update(Message::Board(board::Message::ColumnAdd(Some("someday".to_owned()))));
    let _ = app.update(Message::Board(board::Message::ColumnInput("In Review".to_owned())));
    assert!(simulator(app.view()).find("status:in-review").is_ok(), "key preview");
    let _ = app.update(Message::Board(board::Message::ColumnAdd(None)));
    let _ = app.update(Message::Board(board::Message::ColumnRemove(2)));
    let _ = app.update(Message::Board(board::Message::ColumnMove(4, -1)));
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
    // The typed name labels the column, and the lane is titled with it.
    let columns = app.settings.board_columns(Path::new(TODO_PATH));
    assert_eq!(columns.label("in-review"), Some("In Review"));
    assert!(simulator(app.view()).find("IN REVIEW").is_ok());
}

#[test]
fn settings_sidebar_switches_sections_and_describes_settings() {
    let mut app = app();
    let _ = app.update(Message::Settings(settings::Message::Open));
    let mut ui = simulator(app.view());
    for text in ["Todos", "Dates", "Statuses", "Appearance", "Notifications", "Files"] {
        assert!(ui.find(text).is_ok(), "missing section {text}");
    }
    assert!(ui.find("In the add dialog, every line becomes its own todo.").is_ok());
    drop(ui);
    let messages = click(&app, "Appearance");
    assert!(messages.iter().any(|m| matches!(
        m,
        Message::Settings(settings::Message::Section(crate::settings::SettingsSection::Appearance))
    )));
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
    let _ = app.update(Message::Settings(settings::Message::Open));
    let _ = app.update(Message::Settings(settings::Message::Section(
        crate::settings::SettingsSection::Statuses,
    )));
    let _ = app.update(Message::Settings(settings::Message::StatusInput(
        "In Review".to_owned(),
    )));
    assert!(simulator(app.view()).find("status:in-review").is_ok(), "key preview");
    let _ = app.update(Message::Settings(settings::Message::Change(SettingChange::StatusAdd(
        "In Review".to_owned(),
    ))));
    let _ = app.update(Message::Settings(settings::Message::Change(SettingChange::StatusMove(
        4, -1,
    ))));
    let _ = app.update(Message::Settings(settings::Message::Change(
        SettingChange::StatusHidden("waiting".to_owned(), true),
    )));
    snapshot(&app, "settings-statuses");
    assert_eq!(
        app.settings.statuses.names(),
        ["doing", "todo", "waiting", "in-review", "someday"]
    );
    assert!(app.listing.todos().all(|t| t.todo.status() != Some("waiting")));
    assert!(app.status_options().contains(&"in-review".to_owned()));
    assert_eq!(app.settings.statuses.label("in-review"), Some("In Review"));
    assert!(simulator(app.view()).find("In Review").is_ok());
    let Some(Dialog::Settings(dialog)) = &app.dialog else {
        panic!("settings closed")
    };
    assert!(dialog.status_input.is_empty());
    let _ = app.update(Message::Settings(settings::Message::Change(
        SettingChange::StatusRemove("in-review".to_owned()),
    )));
    let _ = app.update(Message::Settings(settings::Message::Change(
        SettingChange::StatusRemove("doing".to_owned()),
    )));
    assert_eq!(app.settings.statuses.names(), ["doing", "todo", "waiting", "someday"]);
}

#[test]
fn grouping_gives_each_group_its_own_board() {
    let mut app = app();
    let _ = app.update(Message::ToggleMainView);
    assert!(!app.board.is_grouped());
    // Sort by priority first: the board splits into one board per priority.
    let _ = app.update(Message::Drawer(drawer::Message::MoveSort(0, 1)));
    assert!(app.board.is_grouped());
    let groups: Vec<String> = app.board.groups.iter().map(|g| g.values.join(",")).collect();
    assert_eq!(groups, ["A", "B", ""]);
    assert!(simulator(app.view()).find("(A)").is_ok());
    snapshot(&app, "board-grouped");

    // Dropping on the same column of another group changes nothing.
    let call_mom = app.board.groups[0].lanes[0].todos[0].clone();
    assert_eq!(call_mom.todo.body(), "Call mom");
    let target = TodoRef::new(call_mom.line, &call_mom.todo);
    let _ = app.update(Message::Board(board::Message::Press(target, lane(0, 0))));
    let _ = app.update(Message::Board(board::Message::Hover(lane(2, 0), true)));
    let _ = app.update(Message::Board(board::Message::Release));
    assert!(app.drag.is_none());
    assert!(app.dialog.is_none());
}

#[test]
fn grouping_switch_keeps_sorting_on_one_board() {
    let mut app = app();
    let _ = app.update(Message::ToggleMainView);
    let _ = app.update(Message::Drawer(drawer::Message::MoveSort(0, 1)));
    assert!(app.board.is_grouped());
    assert!(simulator(app.view()).find("Group by priority").is_ok());

    let _ = app.update(Message::Board(board::Message::Grouped(false)));
    assert!(!app.state.board_grouped);
    assert!(!app.board.is_grouped());
    // Still sorted by priority inside the column.
    assert_eq!(board_lane_bodies(&app)[0], vec!["Call mom", "Buy milk"]);
    snapshot(&app, "board-ungrouped");
}

#[test]
fn board_follows_filters_and_sorting() {
    let mut app = app();
    let _ = app.update(Message::ToggleMainView);
    let _ = app.update(Message::Drawer(drawer::Message::FilterValue(
        Attribute::Projects,
        "work".to_owned(),
        false,
    )));
    assert_eq!(
        board_lane_bodies(&app),
        vec![
            vec![],
            vec!["Prepare slides"],
            vec!["Legal review"],
            vec!["Send invoice"]
        ]
    );
    let _ = app.update(Message::Drawer(drawer::Message::ResetFilters));
    let _ = app.update(Message::Drawer(drawer::Message::ViewToggle(
        ViewToggle::Completed,
        false,
    )));
    assert!(board_lane_bodies(&app)[3].is_empty());
    // Sorting changes reach the board too: sorted by project, +work first.
    let _ = app.update(Message::Drawer(drawer::Message::ViewToggle(
        ViewToggle::Completed,
        true,
    )));
    let _ = app.update(Message::Drawer(drawer::Message::MoveSort(2, -2)));
    let groups: Vec<String> = app.board.groups.iter().map(|g| g.values.join(",")).collect();
    assert_eq!(groups, ["family", "work", ""]);
}

#[test]
fn contexts_are_highlighted_inside_the_todo_text() {
    let mut app = app();
    let line = "2026-08-17 SAP Cloud Logging @ktbe mit @DanielWalther besprechen +pitch";
    let _ = app.update(Message::Files(files::Message::Loaded(
        PathBuf::from(TODO_PATH),
        Ok(TodoDocument::parse(&format!("{line}\n"), &[])),
    )));
    assert_eq!(app.listing.counts.visible, 1);
    // The simulator cannot click a single span, so the ovals are checked in
    // the snapshot; the segments themselves are tested in the domain.
    snapshot(&app, "inline-contexts");
}

#[test]
fn board_groups_collapse_from_their_header() {
    // Call mom (A) is the only todo due 2026-10-04, Prepare slides (B) is due 2026-10-06.
    let mut app = app();
    let _ = app.update(Message::ToggleMainView);
    let _ = app.update(Message::Drawer(drawer::Message::MoveSort(0, 1)));
    assert!(simulator(app.view()).find("2026-10-04").is_ok());
    let messages = click(&app, "(A)");
    assert!(
        messages
            .iter()
            .any(|m| matches!(m, Message::Board(board::Message::ToggleGroup(key)) if key == "priority:A"))
    );
    for message in messages {
        let _ = app.update(message);
    }
    assert_eq!(app.state.collapsed_board_groups, ["priority:A"]);
    let mut ui = simulator(app.view());
    assert!(ui.find("2026-10-04").is_err(), "collapsed group hides its board");
    assert!(ui.find("2026-10-06").is_ok());
    drop(ui);
    snapshot(&app, "board-collapsed");
    let _ = app.update(Message::Board(board::Message::ToggleGroup("priority:A".to_owned())));
    assert!(simulator(app.view()).find("2026-10-04").is_ok());
}

#[test]
fn list_groups_collapse_from_their_header() {
    // Call mom is the only todo due 2026-10-04; it is in the "to do" group.
    let mut app = app();
    assert!(simulator(app.view()).find("2026-10-04").is_ok());
    let key = app.listing.groups[1].key();
    assert_eq!(key, "status:todo");
    let messages = click(&app, "To do");
    assert!(
        messages
            .iter()
            .any(|m| matches!(m, Message::List(list::Message::ToggleGroup(k)) if *k == key))
    );
    for message in messages {
        let _ = app.update(message);
    }
    assert_eq!(app.state.collapsed_list_groups, vec![key.clone()]);
    assert!(simulator(app.view()).find("2026-10-04").is_err());
    // Keyboard selection skips the hidden rows.
    assert_eq!(app.selectable_todos().count(), app.listing.counts.visible - 2);
    snapshot(&app, "list-collapsed");
    let _ = app.update(Message::List(list::Message::ToggleGroup(key)));
    assert!(simulator(app.view()).find("2026-10-04").is_ok());
}

#[test]
fn board_makes_room_for_the_drawer() {
    let mut app = app();
    let _ = app.update(Message::ToggleMainView);
    let _ = app.update(Message::Drawer(drawer::Message::Toggle));
    snapshot(&app, "board-drawer");
}

#[test]
fn grouped_board_makes_room_for_the_drawer() {
    let mut app = app();
    let _ = app.update(Message::ToggleMainView);
    let _ = app.update(Message::Drawer(drawer::Message::MoveSort(2, -2)));
    let _ = app.update(Message::Drawer(drawer::Message::Toggle));
    snapshot(&app, "board-grouped-drawer");
}

#[test]
fn wide_grouped_board_stays_left_of_the_drawer() {
    let mut app = app();
    let _ = app.update(Message::ToggleMainView);
    let _ = app.update(Message::Drawer(drawer::Message::MoveSort(2, -2)));
    let _ = app.update(Message::Drawer(drawer::Message::Toggle));
    let Some(dir) = std::env::var_os("FLOETASK_SNAPSHOTS") else {
        return;
    };
    let path = PathBuf::from(dir).join("board-grouped-drawer-wide");
    let mut ui = iced_test::Simulator::with_size(iced::Settings::default(), (1600.0, 900.0), app.view());
    let snapshot = ui.snapshot(&app.theme()).unwrap();
    snapshot.matches_image(&path).unwrap();
}

/// Renders the README screenshots: the dark board grouped by project, with the drawer open on
/// each of its tabs, at a size where every column fits.
#[test]
fn dark_board_snapshots_for_the_readme() {
    let mut app = app();
    let _ = app.update(Message::ToggleMainView);
    // Sort by project first: the board splits into one board per project.
    let _ = app.update(Message::Drawer(drawer::Message::MoveSort(2, -2)));
    assert!(app.board.is_grouped());
    let _ = app.update(Message::Drawer(drawer::Message::Toggle));
    app.settings.theme = floetask_application::ThemePreference::Dark;
    let Some(dir) = std::env::var_os("FLOETASK_SNAPSHOTS") else {
        return;
    };
    for (tab, name) in [
        (floetask_application::DrawerTab::Attributes, "readme-attributes"),
        (floetask_application::DrawerTab::Filters, "readme-filters"),
        (floetask_application::DrawerTab::Sorting, "readme-sorting"),
    ] {
        let _ = app.update(Message::Drawer(drawer::Message::Tab(tab)));
        let path = PathBuf::from(&dir).join(name);
        let mut ui = iced_test::Simulator::with_size(iced::Settings::default(), (1600.0, 1000.0), app.view());
        let snapshot = ui.snapshot(&app.theme()).unwrap();
        snapshot.matches_image(&path).unwrap();
    }
}

#[test]
fn about_page_shows_the_version_and_an_available_update() {
    let runtime = runtime();
    let _tokio = runtime.enter();
    let mut app = app();
    let _ = app.update(Message::Settings(settings::Message::Open));
    let _ = app.update(Message::Settings(settings::Message::Section(
        crate::settings::SettingsSection::About,
    )));
    assert!(
        simulator(app.view())
            .find(format!("Version {}", env!("CARGO_PKG_VERSION")).as_str())
            .is_ok()
    );

    // Before a check there is nothing to roll back to yet.
    assert!(
        simulator(app.view())
            .find("Check for updates to list earlier versions.")
            .is_ok()
    );
    let messages = click(&app, "Check now");
    assert!(matches!(
        &messages[..],
        [Message::Settings(settings::Message::CheckForUpdates)]
    ));
    let _ = app.update(Message::Settings(settings::Message::CheckForUpdates));
    assert_eq!(app.update, crate::settings::UpdateState::Checking);
    let found = app.services.ports.updater.releases();
    let _ = app.update(Message::Settings(settings::Message::UpdateChecked(true, found)));
    // The stable channel does not offer the release candidate.
    assert_eq!(app.update, crate::settings::UpdateState::UpToDate);

    // The preview channel does, and switching checks again.
    let _ = app.update(Message::Settings(settings::Message::Change(
        SettingChange::UpdateChannel(floetask_application::UpdateChannel::Preview),
    )));
    assert_eq!(app.update, crate::settings::UpdateState::Checking);
    let found = app.services.ports.updater.releases();
    let _ = app.update(Message::Settings(settings::Message::UpdateChecked(true, found)));
    assert!(matches!(&app.update, crate::settings::UpdateState::Available(release) if release.version == "9.0.0-rc.1"));
    assert!(
        simulator(app.view())
            .find("floetask 9.0.0-rc.1 (preview) is available.")
            .is_ok()
    );
    assert!(simulator(app.view()).find("Install and restart").is_ok());

    // Rolling back installs the earlier version picked.
    let _ = app.update(Message::Settings(settings::Message::RollbackChoice("0.3.1".to_owned())));
    snapshot(&app, "settings-about");
    let messages = click(&app, "Install");
    let [Message::Settings(settings::Message::InstallRelease(version))] = &messages[..] else {
        panic!("expected InstallRelease, got {messages:?}")
    };
    assert_eq!(version, "0.3.1");
    let _ = app.update(Message::Settings(settings::Message::InstallRelease(version.clone())));
    assert_eq!(app.update, crate::settings::UpdateState::Installing("0.3.1".to_owned()));
}

#[test]
fn calendar_shows_todos_by_due_date_and_moves_them_by_drag() {
    use floetask_application::MainView;
    use floetask_domain::calendar::CalendarMode;

    let mut app = app();
    let _ = app.update(Message::ShowMainView(MainView::Calendar));
    assert_eq!(app.state.main_view, MainView::Calendar);
    assert_eq!(app.calendar_anchor, app.today);
    assert!(simulator(app.view()).find("October 2026").is_ok());
    assert!(simulator(app.view()).find("Call mom").is_ok());
    snapshot(&app, "calendar-month");

    // Today (Sunday 4 October) ends the week that starts on Monday 28 September.
    // ("Call mom @phone" is rich text there, which the simulator cannot find.)
    let _ = app.update(Message::Calendar(calendar::Message::Mode(CalendarMode::Week)));
    assert!(simulator(app.view()).find("Prepare slides").is_err());
    let _ = app.update(Message::Calendar(calendar::Message::Step(1)));
    assert!(simulator(app.view()).find("Prepare slides").is_ok());
    snapshot(&app, "calendar-week");

    // Pressing and releasing on the same day is a click: it opens the todo.
    let slides = app.listing.todos().find(|e| e.todo.body() == "Prepare slides").unwrap();
    let target = TodoRef::new(slides.line, &slides.todo);
    let due = slides.todo.due().unwrap();
    let _ = app.update(Message::Calendar(calendar::Message::Press(target.clone(), Some(due))));
    let _ = app.update(Message::Calendar(calendar::Message::Release));
    assert!(matches!(app.dialog, Some(Dialog::Editor(_))));
    let _ = app.update(Message::CloseDialog);

    // Dragged away and dropped back on the same day, nothing opens.
    let _ = app.update(Message::Calendar(calendar::Message::Press(target.clone(), Some(due))));
    let _ = app.update(Message::Calendar(calendar::Message::CardLeft));
    let _ = app.update(Message::Calendar(calendar::Message::Release));
    assert!(app.dialog.is_none());

    // Dropped on another day, only the due date changes.
    let next_day = floetask_domain::date::add_days(due, 2);
    let _ = app.update(Message::Calendar(calendar::Message::Press(target.clone(), Some(due))));
    let _ = app.update(Message::Calendar(calendar::Message::Hover(next_day, true)));
    let _ = app.update(Message::Calendar(calendar::Message::Hover(due, false)));
    assert_eq!(app.calendar_hover, Some(next_day));
    let _ = app.update(Message::Calendar(calendar::Message::Release));
    assert!(app.calendar_drag.is_none());
    let moved = app.services.todo_files.set_date(
        Path::new(TODO_PATH),
        &target,
        "due",
        Some(next_day),
        &app.file_options(),
    );
    let _ = app.update(Message::Files(files::Message::Saved(PathBuf::from(TODO_PATH), moved)));
    let slides = app.listing.todos().find(|e| e.todo.body() == "Prepare slides").unwrap();
    assert_eq!(slides.todo.due(), Some(next_day));

    let _ = app.update(Message::Calendar(calendar::Message::OpenDay(next_day)));
    assert_eq!(app.state.calendar_mode, CalendarMode::Day);
    assert!(simulator(app.view()).find("Prepare slides").is_ok());
    snapshot(&app, "calendar-day");
}

#[test]
fn calendar_checkbox_completes_a_todo_without_dragging_it() {
    use floetask_application::MainView;

    let mut app = app();
    let _ = app.update(Message::ShowMainView(MainView::Calendar));
    let slides = app.listing.todos().find(|e| e.todo.body() == "Prepare slides").unwrap();
    let (line, due) = (slides.line, slides.todo.due().unwrap());
    let _ = app.update(Message::Calendar(calendar::Message::OpenDay(due)));

    // The checkbox sits just left of the todo text.
    let mut ui = simulator(app.view());
    let label = ui.find("Prepare slides").unwrap().bounds();
    ui.point_at(iced::Point::new(label.x - 18.0, label.y + label.height / 2.0));
    let _ = ui.simulate(iced_test::simulator::click());
    let messages: Vec<Message> = ui.into_messages().collect();
    assert!(
        matches!(messages.first(), Some(Message::List(list::Message::ToggleComplete(target))) if target.line == line),
        "{messages:?}"
    );
    assert!(
        !messages
            .iter()
            .any(|m| matches!(m, Message::Calendar(calendar::Message::Press(..)))),
        "{messages:?}"
    );
}

#[test]
fn undated_panel_lists_todos_without_due_date_and_plans_them() {
    use floetask_application::MainView;

    let mut app = app();
    let _ = app.update(Message::ShowMainView(MainView::Calendar));
    let messages = click(&app, "2 without a due date");
    assert!(
        matches!(&messages[..], [Message::Calendar(calendar::Message::ToggleUndated)]),
        "{messages:?}"
    );
    let _ = app.update(Message::Calendar(calendar::Message::ToggleUndated));
    assert!(app.calendar_undated_open);
    assert!(simulator(app.view()).find("Without due date").is_ok());
    assert!(simulator(app.view()).find("Legal review").is_ok());
    snapshot(&app, "calendar-undated");

    // A click on a card in the panel opens it.
    let legal = app.listing.todos().find(|e| e.todo.body() == "Legal review").unwrap();
    let target = TodoRef::new(legal.line, &legal.todo);
    let _ = app.update(Message::Calendar(calendar::Message::Press(target.clone(), None)));
    let _ = app.update(Message::Calendar(calendar::Message::Release));
    assert!(matches!(app.dialog, Some(Dialog::Editor(_))));
    let _ = app.update(Message::CloseDialog);

    // Dragged and dropped back in the panel, nothing opens.
    let _ = app.update(Message::Calendar(calendar::Message::Press(target.clone(), None)));
    let _ = app.update(Message::Calendar(calendar::Message::CardLeft));
    let _ = app.update(Message::Calendar(calendar::Message::Release));
    assert!(app.dialog.is_none());

    // Dragged onto a day, it gets that due date.
    let day = app.today;
    let _ = app.update(Message::Calendar(calendar::Message::Press(target.clone(), None)));
    let _ = app.update(Message::Calendar(calendar::Message::HoverUndated(false)));
    let _ = app.update(Message::Calendar(calendar::Message::Hover(day, true)));
    let _ = app.update(Message::Calendar(calendar::Message::Release));
    assert!(app.calendar_drag.is_none());
    let moved = app
        .services
        .todo_files
        .set_date(Path::new(TODO_PATH), &target, "due", Some(day), &app.file_options());
    let _ = app.update(Message::Files(files::Message::Saved(PathBuf::from(TODO_PATH), moved)));
    let legal = app.listing.todos().find(|e| e.todo.body() == "Legal review").unwrap();
    assert_eq!(legal.todo.due(), Some(day));
    assert!(simulator(app.view()).find("1 without a due date").is_ok());
}

#[test]
fn main_view_closes_side_panels_before_getting_narrower_than_its_minimum() {
    use floetask_application::MainView;
    use floetask_domain::calendar::CalendarMode;

    let mut app = app();
    let _ = app.update(Message::Drawer(drawer::Message::Toggle));
    let _ = app.update(Message::ShowMainView(MainView::Calendar));
    let _ = app.update(Message::Calendar(calendar::Message::ToggleUndated));
    let open = |app: &Floetask| {
        let shown = app.shown_panels();
        (shown.filters, shown.files, shown.undated)
    };

    let _ = app.update(Message::Calendar(calendar::Message::Mode(CalendarMode::Week)));
    app.state.window.width = 1700.0;
    assert_eq!(open(&app), (true, true, true));
    app.state.window.width = 1500.0;
    assert_eq!(open(&app), (false, true, true), "filter drawer closes first");
    app.state.window.width = 1200.0;
    assert_eq!(open(&app), (false, false, true), "then the file drawer");
    // The user's choice is kept for when the window grows again.
    assert!(app.state.drawer_open && app.state.files_drawer_open);

    app.state.window.width = 1024.0;
    snapshot(&app, "calendar-week-narrow");

    // The same in the list: filter drawer first, then the file drawer.
    let _ = app.update(Message::ShowMainView(MainView::List));
    app.state.window.width = 1400.0;
    assert_eq!(
        open(&app),
        (true, true, false),
        "the undated panel belongs to the calendar"
    );
    app.state.window.width = 1300.0;
    assert_eq!(open(&app), (false, true, false));
    app.state.window.width = 1000.0;
    assert_eq!(open(&app), (false, false, false));

    // Asked for while there is no room, a panel opens over the main view;
    // the next click (or Escape) hides it, and it stays open for later.
    let _ = app.update(Message::Drawer(drawer::Message::Toggle));
    assert_eq!(app.panel_overlay, Some(crate::layout::Panel::Filters));
    assert!(app.state.drawer_open);
    snapshot(&app, "filter-overlay");
    let _ = app.update(Message::Drawer(drawer::Message::Toggle));
    assert_eq!(app.panel_overlay, None);
    assert!(app.state.drawer_open);
    let _ = app.update(Message::Files(files::Message::ToggleDrawer));
    assert_eq!(app.panel_overlay, Some(crate::layout::Panel::Files));
    let _ = app.update(Message::ClosePanelOverlay);
    assert_eq!(app.panel_overlay, None);
    // Closed panels opened in a narrow window show as an overlay too.
    app.state.drawer_open = false;
    let _ = app.update(Message::Drawer(drawer::Message::Toggle));
    assert!(app.state.drawer_open);
    assert_eq!(app.panel_overlay, Some(crate::layout::Panel::Filters));
}
