use std::path::PathBuf;

use iced::widget::text_editor;
use iced::{keyboard, window};

use floetask_application::{AppError, DrawerTab, SavedFilter, TodoRef};
use floetask_domain::listing::{Attribute, LaneId};
use floetask_domain::{Date, NoteLink, Priority, RecurrenceUnit, TodoDocument};

use super::dialog::DateKey;

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
    /// Collapses or expands the list group with this key.
    ToggleListGroup(String),
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
    EditorStatus(String),
    EditorOpenCalendar(DateKey),
    EditorCloseCalendar,
    EditorRecurrenceCount(String),
    EditorRecurrenceUnit(RecurrenceUnit),
    EditorRecurrenceStrict(bool),
    EditorApplyRecurrence(bool),
    EditorPomodoros(String),
    EditorApplyPomodoros(bool),
    EditorNoteAction(iced_markdown_editor::Action),
    /// The note file of the todo being edited, read from disk (`None`: missing).
    NoteLoaded(TodoRef, Result<Option<String>, AppError>),
    /// Asks for a Markdown file on disk to link instead of creating one.
    EditorPickNoteFile,
    NoteFilePicked(Option<PathBuf>),
    /// Answers to "move the picked file into the notes folder?".
    EditorMovePickedNote,
    EditorKeepPickedNote,
    EditorCancelPickedNote,
    /// A picked file, moved into the notes folder, ready to link.
    NoteFileMoved(Result<NoteLink, AppError>),
    /// Removes the todo's `note:` tag; the file is kept.
    EditorUnlinkNote,
    /// A note just linked in the dialog, read from disk (`None`: missing).
    NoteLinked(NoteLink, Result<Option<String>, AppError>),
    /// Hands the dialog's note over to the note window.
    EditorOpenNoteWindow,
    /// The dialog's note, ready for the note window: its link and its text
    /// on disk (`None`: no file yet).
    NoteHandedOver(Result<(NoteLink, Option<String>), AppError>),
    SaveEditor,

    // Note window
    /// A note opened from a todo card, created first when missing.
    NoteWindowLoaded(PathBuf, Result<(NoteLink, String), AppError>),
    NoteWindowAction(iced_markdown_editor::Action),
    /// Saves the note window if this is still its latest edit.
    NoteWindowAutosave(u64),
    NoteWindowSave,
    /// A note window save finished: the file, the text written, the result.
    NoteWindowSaved(PathBuf, String, Result<(), AppError>),
    NoteWindowOpenExternally,
    /// `Escape` taken by a focused notes editor, in the window it came from.
    NoteEditorEscape(window::Id),

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
    SettingsSection(super::SettingsSection),
    SettingsStatusInput(String),

    // Updates (Settings → About)
    CheckForUpdates,
    /// Whether the user asked, and what the release server said.
    UpdateChecked(
        bool,
        Result<Option<floetask_application::ports::AvailableUpdate>, AppError>,
    ),
    InstallUpdate,
    UpdateInstalled(Result<(), AppError>),

    // Dialogs
    // Status board
    ToggleMainView,
    ShowMainView(floetask_application::MainView),
    // Calendar view
    CalendarMode(floetask_domain::calendar::CalendarMode),
    /// Moves the calendar by this many days, weeks or months.
    CalendarStep(i64),
    CalendarToday,
    /// Opens one day in Day mode.
    CalendarOpenDay(Date),
    /// A todo is pressed on a day, or in the undated panel (`None`): the
    /// start of a drag or a click.
    CalendarPress(TodoRef, Option<Date>),
    /// Shows or hides the panel of todos without a due date.
    CalendarToggleUndated,
    /// Hides a side panel shown over the main view (a click beside it).
    ClosePanelOverlay,
    /// The cursor entered or left the undated panel.
    CalendarHoverUndated(bool),
    CalendarHover(Date, bool),
    /// The cursor left a todo on the calendar: a press there became a drag.
    CalendarCardLeft,
    CalendarRelease,
    /// Turns the board's one-board-per-group layout on or off.
    BoardGrouped(bool),
    /// Collapses or expands the board group with this key.
    ToggleBoardGroup(String),
    /// A card was pressed in a lane: start dragging it.
    BoardPress(TodoRef, LaneId),
    /// The mouse entered (`true`) or left a lane.
    BoardHover(LaneId, bool),
    /// The cursor left a card: a press there became a drag.
    BoardCardLeft,
    BoardRelease,
    OpenBoardColumns,
    BoardColumnInput(String),
    /// Adds the given column, or the typed one for `None`.
    BoardColumnAdd(Option<String>),
    BoardColumnMove(usize, isize),
    BoardColumnRemove(usize),
    BoardColumnsReset,

    ConfirmDialog,
    CloseDialog,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewToggle {
    Completed,
    Hidden,
    HiddenAttributes,
    HiddenStatuses,
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
    CheckForUpdates(bool),
    NotificationThreshold(u8),
    Zoom(u16),
    Theme(floetask_application::ThemePreference),
    WeekStart(floetask_domain::WeekStart),
    MonthOverflow(floetask_application::MonthOverflow),
    StatusAdd(String),
    StatusRemove(String),
    StatusMove(usize, isize),
    StatusHidden(String, bool),
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
