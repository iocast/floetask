use iced::{keyboard, window};

use floetask_application::{AppError, MainView};

use crate::{board, calendar, drawer, editor, files, list, note_window, search, settings};

/// Everything that can happen in the GUI. Runtime and window events live
/// here; each feature has its own message type, wrapped in one variant, and
/// is handled by its own `update` (see `Floetask::dispatch`).
#[derive(Debug, Clone)]
pub enum Message {
    // Runtime events
    SystemDark(bool),
    Tick,
    /// A key no widget took, in the window it was pressed in.
    KeyPressed(window::Id, keyboard::Key, keyboard::Modifiers),
    ModifiersChanged(keyboard::Modifiers),
    Quit,
    WindowReady(Option<window::Id>),
    WindowDrag,
    WindowResize(window::Direction),
    WindowMinimize,
    WindowToggleMaximize,
    Window(window::Id, window::Event),
    /// `Escape` taken by a focused notes editor, in the window it came from.
    NoteEditorEscape(window::Id),
    DismissToast(u64),
    Failed(AppError),
    Noop,

    // Layout shared by every view
    ToggleMainView,
    ShowMainView(MainView),
    /// Hides a side panel shown over the main view (a click beside it).
    ClosePanelOverlay,
    /// Opens a link with the system: from todo cards and both notes editors.
    OpenLink(String),
    ConfirmDialog,
    CloseDialog,

    // Features
    Files(files::Message),
    List(list::Message),
    Editor(editor::Message),
    NoteWindow(note_window::Message),
    Drawer(drawer::Message),
    Search(search::Message),
    Settings(settings::Message),
    Board(board::Message),
    Calendar(calendar::Message),
}

impl Message {
    /// Turns the result of a background task without output into a message.
    pub fn from_result(result: Result<(), AppError>) -> Self {
        match result {
            Ok(()) => Message::Noop,
            Err(error) => Message::Failed(error),
        }
    }
}
