//! Dragging cards between columns and editing a file's columns.

use iced::Task;

use floetask_domain::board::BoardColumns;
use floetask_domain::status::key_from_label;

use super::BoardDrag;
use crate::app::{Dialog, Floetask, Message, Target};
use crate::{board, list};

impl Floetask {
    pub(crate) fn update_board(&mut self, message: board::Message) -> Task<Message> {
        match message {
            board::Message::Grouped(on) => {
                self.state.board_grouped = on;
                self.drag = None;
                self.refresh();
                self.persist_state()
            }
            board::Message::ToggleGroup(key) => {
                self.state.toggle_board_group(&key);
                self.persist_state()
            }
            board::Message::Press(target, lane) => {
                self.drag = Some(BoardDrag {
                    target,
                    from: lane,
                    moved: false,
                });
                self.board_hover = Some(lane);
                Task::none()
            }
            board::Message::Hover(lane, entered) => {
                if entered {
                    self.board_hover = Some(lane);
                } else if self.board_hover == Some(lane) {
                    // Leaving one column and entering the next can arrive in
                    // either order, so only clear our own lane.
                    self.board_hover = None;
                }
                Task::none()
            }
            board::Message::CardLeft => {
                if let Some(drag) = &mut self.drag {
                    drag.moved = true;
                }
                Task::none()
            }
            board::Message::Release => self.drop_card(),
            board::Message::OpenColumns => {
                let columns = self.active_board_columns();
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
                    board::Message::ColumnInput(text) => *input = text,
                    board::Message::ColumnAdd(Some(key)) => {
                        columns.add(&key);
                    }
                    // A typed name: the key is derived from it, and the name
                    // labels the column unless settings already name it.
                    board::Message::ColumnAdd(None) => {
                        let key = key_from_label(input);
                        if columns.add(&key) {
                            if !self.settings.statuses.names().contains(&key) {
                                columns.set_label(&key, input);
                            }
                            input.clear();
                        }
                    }
                    board::Message::ColumnMove(index, step) => columns.move_column(index, step),
                    board::Message::ColumnRemove(index) => columns.remove(index),
                    board::Message::ColumnsReset => *columns = BoardColumns::default(),
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
    pub(crate) fn save_board_columns(&mut self, columns: BoardColumns) -> Task<Message> {
        let Some(path) = self.active_path() else {
            return Task::none();
        };
        self.settings.set_board_columns(&path, columns);
        self.refresh();
        let store = self.services.ports.settings.clone();
        let settings = self.settings.clone();
        Task::perform(async move { store.save(&settings) }, Message::from_result)
    }

    /// Ends a drag. Released over another column, the card moves there;
    /// released on the card itself, it was a click and opens the todo; dragged
    /// and dropped back in its own column, nothing happens. Moving only changes the status: dropped on the same
    /// column of another group's board, nothing changes.
    fn drop_card(&mut self) -> Task<Message> {
        let Some(drag) = self.drag.take() else {
            return Task::none();
        };
        let Some(lane) = self.board_hover else {
            return Task::none();
        };
        if lane == drag.from {
            return if drag.moved {
                Task::none()
            } else {
                self.update(Message::List(list::Message::OpenTodo(drag.target)))
            };
        }
        let Some(column) = self.board.lane(lane).and_then(|lane| lane.column.clone()) else {
            return Task::none();
        };
        if self.board.lane(drag.from).and_then(|from| from.column.as_ref()) == Some(&column) {
            return Task::none();
        }
        let Target { file, todo } = drag.target;
        self.change_file(file, move |service, path, options| {
            service.move_to_column(path, &todo, &column, options)
        })
    }
}
