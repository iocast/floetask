use std::path::PathBuf;

use iced::widget::text_editor;
use iced::{keyboard, window};

use floetask_application::{AppError, DrawerTab, SavedFilter, TodoRef};
use floetask_domain::listing::Attribute;
use floetask_domain::{Date, Priority, RecurrenceUnit, TodoDocument};

use super::dialog::DateKey;

#[derive(Debug, Clone)]
pub enum Message {
    // Runtime events
    SystemDark(bool),
    Tick,
    KeyPressed(keyboard::Key, keyboard::Modifiers),
    ModifiersChanged(keyboard::Modifiers),
    Quit,
    WindowReady(Option<window::Id>),
    WindowDrag,
    WindowResize(window::Direction),
    WindowMinimize,
    WindowToggleMaximize,
    RowHover(Option<usize>),
    Window(window::Id, window::Event),
    DismissToast(u64),
    Failed(AppError),
    Noop,

    // Files
    FileLoaded(PathBuf, Result<TodoDocument, AppError>),
    FileChanged(PathBuf),
    /// Result of a change written to the active file.
    Saved(PathBuf, Result<TodoDocument, AppError>),
    /// Result of a change that also reports how many todos it touched.
    SavedCount(PathBuf, Result<(TodoDocument, usize), AppError>, &'static str),
    OpenFileDialog,
    CreateFileDialog,
    FilePicked(Option<PathBuf>),
    FileCreated(Option<PathBuf>),
    SelectFile(usize),
    FileMenu(Option<usize>),
    OpenDoneFile(usize),
    ToggleFilesDrawer,
    ChangeDoneFile(usize),
    DoneFilePicked(usize, Option<PathBuf>),
    RevealTodoFile(usize),
    AskRemoveFile(usize),
    ArchiveCompleted,
    SavedFiltersLoaded(Result<Vec<SavedFilter>, AppError>),

    // List
    OpenTodo(TodoRef),
    ToggleComplete(TodoRef),
    RowMenu(Option<usize>),
    CopyTodo(String),
    ArchiveOne(TodoRef),
    AskDelete(TodoRef),
    ChipFilter(Attribute, String),
    OpenLink(String),
    OpenRowDatePicker(TodoRef, DateKey),
    OpenNote(TodoRef),
    NoteOpened(Result<PathBuf, AppError>),

    // Add / edit dialog
    NewTodo,
    EditorAction(text_editor::Action),
    EditorSuggestion(usize),
    EditorSuggestionMove(isize),
    EditorAccept,
    EditorDismissSuggestions,
    EditorPriority(Option<Priority>),
    EditorOpenCalendar(DateKey),
    EditorCloseCalendar,
    EditorRecurrenceCount(String),
    EditorRecurrenceUnit(RecurrenceUnit),
    EditorRecurrenceStrict(bool),
    EditorApplyRecurrence(bool),
    EditorPomodoros(String),
    EditorApplyPomodoros(bool),
    EditorNote(String),
    EditorApplyNote(bool),
    SaveEditor,

    // Calendar (row date picker or editor)
    CalendarMonth(i32),
    CalendarPick(Option<Date>),

    // Drawer
    ToggleDrawer,
    DrawerTab(DrawerTab),
    ToggleSection(Attribute),
    FilterValue(Attribute, String, bool),
    ToggleCategory(Attribute),
    ViewToggle(ViewToggle, bool),
    MoveSort(usize, isize),
    InvertSort(usize),
    FileOrder(bool),
    CompletedLast(bool),
    AskRename(Attribute, String),
    RenameInput(String),
    AskRemoveValue(Attribute, String),
    ResetFilters,

    // Search
    FocusSearch,
    SearchInput(String),
    SearchSubmitAsTodo,
    ToggleSavedFilters,
    AskSaveFilter,
    SaveFilterName(String),
    ApplySavedFilter(usize),
    AskDeleteSavedFilter(usize),
    ToggleSuppress(usize),

    // Settings
    OpenSettings,
    Setting(SettingChange),
    ToggleTheme,

    // Dialogs
    ConfirmDialog,
    CloseDialog,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewToggle {
    Completed,
    Hidden,
    HiddenAttributes,
    FutureThreshold,
    FutureDue,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SettingChange {
    AppendCreationDate(bool),
    ConvertRelativeDates(bool),
    HumanFriendlyDates(bool),
    SafeWrites(bool),
    BulkCreation(bool),
    DisableAnimations(bool),
    Compact(bool),
    Notifications(bool),
    NotificationThreshold(u8),
    Zoom(u16),
    Theme(floetask_application::ThemePreference),
    WeekStart(floetask_domain::WeekStart),
    Language(String),
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
