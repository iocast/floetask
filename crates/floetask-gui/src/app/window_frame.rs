//! floetask draws its own title bar, so moving, resizing and the window
//! buttons are handled here.

use iced::{Task, window};

use super::{Floetask, Message};

impl Floetask {
    pub(super) fn update_window_frame(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::WindowReady(id) => {
                self.window = id;
                // The saved flag may be stale, so it is read back from the OS.
                return id.map_or_else(Task::none, |id| window::is_maximized(id).map(Message::WindowMaximized));
            }
            // Hidden resize edges while maximised; the flag must follow the
            // real window, so it only changes here.
            Message::WindowMaximized(maximized) => {
                self.maximized = maximized;
                self.state.window.maximized = maximized;
                return Task::none();
            }
            _ => {}
        }
        let Some(id) = self.window else {
            return Task::none();
        };
        match message {
            Message::WindowDrag => window::drag(id),
            Message::WindowResize(direction) => window::drag_resize(id, direction),
            Message::WindowMinimize => window::minimize(id, true),
            // The window resizes, and the resize reads the new state back.
            Message::WindowToggleMaximize => window::toggle_maximize(id),
            _ => Task::none(),
        }
    }
}
