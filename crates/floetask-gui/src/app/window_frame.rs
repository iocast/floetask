//! floetask draws its own title bar, so moving, resizing and the window
//! buttons are handled here.

use iced::{Task, window};

use super::{Floetask, Message};

impl Floetask {
    pub(super) fn update_window_frame(&mut self, message: Message) -> Task<Message> {
        if let Message::WindowReady(id) = message {
            self.window = id;
            return Task::none();
        }
        if let Message::RowHover(line) = message {
            self.hovered = line;
            return Task::none();
        }
        let Some(id) = self.window else {
            return Task::none();
        };
        match message {
            Message::WindowDrag => window::drag(id),
            Message::WindowResize(direction) => window::drag_resize(id, direction),
            Message::WindowMinimize => window::minimize(id, true),
            Message::WindowToggleMaximize => {
                self.maximized = !self.maximized;
                self.state.window.maximized = self.maximized;
                window::toggle_maximize(id)
            }
            _ => Task::none(),
        }
    }
}
