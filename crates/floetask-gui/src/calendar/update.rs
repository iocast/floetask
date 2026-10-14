//! Switching modes, stepping through periods and dragging todos to another
//! day.

use iced::Task;

use floetask_domain::calendar::{CalendarMode, step};

use crate::app::{Floetask, Message};
use crate::{calendar, list};

impl Floetask {
    pub(crate) fn update_calendar(&mut self, message: calendar::Message) -> Task<Message> {
        match message {
            calendar::Message::Mode(mode) => {
                self.state.calendar_mode = mode;
                self.persist_state()
            }
            calendar::Message::Step(steps) => {
                self.calendar_anchor = step(self.state.calendar_mode, self.calendar_anchor, steps);
                Task::none()
            }
            calendar::Message::Today => {
                self.calendar_anchor = self.today;
                Task::none()
            }
            calendar::Message::OpenDay(day) => {
                self.calendar_anchor = day;
                self.state.calendar_mode = CalendarMode::Day;
                self.persist_state()
            }
            calendar::Message::Press(target, day) => {
                self.calendar_drag = Some((target, day));
                self.calendar_drag_moved = false;
                self.calendar_hover = day;
                self.calendar_undated_hover = day.is_none();
                Task::none()
            }
            calendar::Message::ToggleUndated => {
                self.calendar_undated_open = !self.calendar_undated_open;
                Task::none()
            }
            calendar::Message::HoverUndated(inside) => {
                self.calendar_undated_hover = inside;
                if inside {
                    self.calendar_hover = None;
                }
                Task::none()
            }
            calendar::Message::Hover(day, entered) => {
                if entered {
                    self.calendar_hover = Some(day);
                } else if self.calendar_hover == Some(day) {
                    // Leaving one day and entering the next can arrive in
                    // either order, so only clear our own day.
                    self.calendar_hover = None;
                }
                Task::none()
            }
            calendar::Message::CardLeft => {
                if self.calendar_drag.is_some() {
                    self.calendar_drag_moved = true;
                }
                Task::none()
            }
            calendar::Message::Release => self.drop_on_day(),
        }
    }

    /// Ends a drag. Released on another day, the todo's `due:` moves there
    /// (or is set, for a todo from the undated panel); released on the todo
    /// itself, it was a click and opens the todo. Dragged and dropped back
    /// where it was picked up, nothing happens.
    fn drop_on_day(&mut self) -> Task<Message> {
        let Some((target, from)) = self.calendar_drag.take() else {
            return Task::none();
        };
        let moved = std::mem::take(&mut self.calendar_drag_moved);
        match (from, self.calendar_hover) {
            (Some(from), Some(day)) if from == day && moved => Task::none(),
            (Some(from), Some(day)) if from == day => self.update(Message::List(list::Message::OpenTodo(target))),
            (None, None) if self.calendar_undated_hover && !moved => {
                self.update(Message::List(list::Message::OpenTodo(target)))
            }
            (_, Some(day)) => self.change_active_file(move |service, path, options| {
                service.set_date(path, &target, "due", Some(day), options)
            }),
            _ => Task::none(),
        }
    }
}
