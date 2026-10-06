//! The calendar view: switching modes, stepping through periods and dragging
//! todos to another day.

use iced::Task;

use floetask_application::MainView;
use floetask_domain::calendar::{CalendarMode, step};

use super::{Floetask, Message};

impl Floetask {
    pub(super) fn update_calendar(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::ShowMainView(view) => {
                if view == MainView::Calendar && self.state.main_view != MainView::Calendar {
                    // The calendar always opens on today.
                    self.calendar_anchor = self.today;
                }
                self.state.main_view = view;
                self.drag = None;
                self.calendar_drag = None;
                self.persist_state()
            }
            Message::CalendarMode(mode) => {
                self.state.calendar_mode = mode;
                self.persist_state()
            }
            Message::CalendarStep(steps) => {
                self.calendar_anchor = step(self.state.calendar_mode, self.calendar_anchor, steps);
                Task::none()
            }
            Message::CalendarToday => {
                self.calendar_anchor = self.today;
                Task::none()
            }
            Message::CalendarOpenDay(day) => {
                self.calendar_anchor = day;
                self.state.calendar_mode = CalendarMode::Day;
                self.persist_state()
            }
            Message::CalendarPress(target, day) => {
                self.calendar_drag = Some((target, day));
                self.calendar_hover = Some(day);
                Task::none()
            }
            Message::CalendarHover(day, entered) => {
                if entered {
                    self.calendar_hover = Some(day);
                } else if self.calendar_hover == Some(day) {
                    // Leaving one day and entering the next can arrive in
                    // either order, so only clear our own day.
                    self.calendar_hover = None;
                }
                Task::none()
            }
            Message::CalendarRelease => self.drop_on_day(),
            _ => Task::none(),
        }
    }

    /// Ends a drag. Released on another day, the todo's `due:` moves there;
    /// released on its own day, it was a click and opens the todo.
    fn drop_on_day(&mut self) -> Task<Message> {
        let Some((target, from)) = self.calendar_drag.take() else {
            return Task::none();
        };
        let Some(day) = self.calendar_hover else {
            return Task::none();
        };
        if day == from {
            return self.update(Message::OpenTodo(target));
        }
        self.change_active_file(move |service, path, options| {
            service.set_date(path, &target, "due", Some(day), options)
        })
    }
}
