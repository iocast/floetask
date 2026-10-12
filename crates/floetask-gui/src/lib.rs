//! Presentation layer of floetask: the iced user interface.
//!
//! The GUI depends on the application layer (use cases and port traits) and
//! on domain types for display, never on infrastructure. The composition
//! root hands it ready-made [`floetask_application::Services`].

mod app;
mod compose;
mod i18n;
mod layout;
mod logo;
mod theme;
mod view;

#[cfg(test)]
mod tests;

use iced::{Size, Task, window};

pub use app::Startup;

/// Runs the GUI until the main window is closed.
pub fn run(startup: Startup) -> iced::Result {
    let window_state = startup.services.ports.state.load().window;
    let main_window = window::Settings {
        size: Size::new(window_state.width, window_state.height),
        position: match (window_state.x, window_state.y) {
            (Some(x), Some(y)) => window::Position::Specific(iced::Point::new(x, y)),
            _ => window::Position::Centered,
        },
        maximized: window_state.maximized,
        min_size: Some(Size::new(480.0, 360.0)),
        exit_on_close_request: false,
        // floetask draws its own title bar with the window buttons.
        decorations: false,
        icon: logo::window_icon(),
        ..window::Settings::default()
    };
    // `boot` may only run once, but iced wants a reusable function.
    let startup = std::sync::Mutex::new(Some(startup));
    // A daemon rather than an application, because notes open in a second
    // window; it opens the main window itself and exits when that closes.
    let daemon = iced::daemon(
        move || {
            let startup = startup
                .lock()
                .ok()
                .and_then(|mut slot| slot.take())
                .expect("floetask boots once");
            let (app, boot) = app::Floetask::boot(startup);
            let (_, open) = window::open(main_window.clone());
            (
                app,
                Task::batch([open.map(|id| app::Message::WindowReady(Some(id))), boot]),
            )
        },
        app::Floetask::update,
        app::Floetask::view_window,
    )
    .title(app::Floetask::window_title)
    .theme(|app: &app::Floetask, _| app.theme())
    .scale_factor(|app: &app::Floetask, _| app.scale_factor())
    .subscription(app::Floetask::subscription);
    // The notes editor measures text with its own fonts; without them the
    // caret would not line up with the text.
    iced_markdown_editor::FONTS
        .into_iter()
        .fold(daemon, |daemon, font| daemon.font(font))
        .run()
}
