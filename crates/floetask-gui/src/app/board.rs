//! The status board: switching views, dragging cards between columns and
//! editing a file's columns.

use iced::Task;

use floetask_application::{MainView, TodoRef};
use floetask_domain::board::{BoardColumns, Column};
use floetask_domain::listing::LaneId;

use super::{Dialog, Floetask, Message};

/// A card being dragged on the board.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoardDrag {
    pub target: TodoRef,
    /// Lane the card was picked up from.
    pub from: LaneId,
}

impl Floetask {
    pub(super) fn update_board(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::ToggleMainView => {
                self.state.main_view = match self.state.main_view {
                    MainView::List => MainView::Board,
                    MainView::Board => MainView::List,
                };
                self.drag = None;
                self.persist_state()
            }
            Message::BoardPress(target, lane) => {
                self.drag = Some(BoardDrag { target, from: lane });
                self.board_hover = Some(lane);
                Task::none()
            }
            Message::BoardHover(lane, entered) => {
                if entered {
                    self.board_hover = Some(lane);
                } else if self.board_hover == Some(lane) {
                    // Leaving one column and entering the next can arrive in
                    // either order, so only clear our own lane.
                    self.board_hover = None;
                }
                Task::none()
            }
            Message::BoardRelease => self.drop_card(),
            Message::OpenBoardColumns => {
                let columns = self.active_board_columns().keys();
                self.dialog = Some(Dialog::BoardColumns {
                    columns,
                    input: String::new(),
                });
                Task::none()
            }
            other => {
                let Some(Dialog::BoardColumns { columns, input }) = &mut self.dialog else {
                    return Task::none();
                };
                match other {
                    Message::BoardColumnInput(text) => *input = text.to_lowercase().replace(' ', "-"),
                    Message::BoardColumnAdd(key) => {
                        let key = key.unwrap_or_else(|| std::mem::take(input));
                        if Column::parse(&key).is_some() && !columns.contains(&key) {
                            columns.push(key);
                        }
                    }
                    Message::BoardColumnMove(index, step) => {
                        let target = index as isize + step;
                        if target >= 0 && (target as usize) < columns.len() {
                            columns.swap(index, target as usize);
                        }
                    }
                    Message::BoardColumnRemove(index) => {
                        // A board keeps at least one column.
                        if columns.len() > 1 && index < columns.len() {
                            columns.remove(index);
                        }
                    }
                    Message::BoardColumnsReset => *columns = BoardColumns::default().keys(),
                    _ => {}
                }
                Task::none()
            }
        }
    }

    /// The columns of the active file's board.
    pub(crate) fn active_board_columns(&self) -> BoardColumns {
        self.active_path()
            .map(|path| self.settings.board_columns(&path))
            .unwrap_or_default()
    }

    /// Statuses offered in the todo dialog: the global list, then statuses
    /// that only the active file's board has a column for.
    pub(crate) fn status_options(&self) -> Vec<String> {
        let mut options = self.settings.statuses.names().to_vec();
        for status in self.active_board_columns().statuses() {
            if !options.iter().any(|known| known == status) {
                options.push(status.to_owned());
            }
        }
        options
    }

    /// Saves the columns from the dialog for the active file.
    pub(super) fn save_board_columns(&mut self, columns: Vec<String>) -> Task<Message> {
        let Some(path) = self.active_path() else {
            return Task::none();
        };
        self.settings.set_board_columns(&path, BoardColumns::from_keys(columns));
        self.refresh();
        let store = self.services.ports.settings.clone();
        let settings = self.settings.clone();
        Task::perform(async move { store.save(&settings) }, Message::from_result)
    }

    /// Ends a drag. Released over another column, the card moves there;
    /// released in its own column without moving away, it was a click and
    /// opens the todo. Moving only changes the status: dropped on the same
    /// column of another group's board, nothing changes.
    fn drop_card(&mut self) -> Task<Message> {
        let Some(drag) = self.drag.take() else {
            return Task::none();
        };
        let Some(lane) = self.board_hover else {
            return Task::none();
        };
        if lane == drag.from {
            return self.update(Message::OpenTodo(drag.target));
        }
        let Some(column) = self.board.lane(lane).and_then(|lane| lane.column.clone()) else {
            return Task::none();
        };
        if self.board.lane(drag.from).and_then(|from| from.column.as_ref()) == Some(&column) {
            return Task::none();
        }
        let target = drag.target;
        self.change_active_file(move |service, path, options| service.move_to_column(path, &target, &column, options))
    }
}
