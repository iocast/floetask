//! Presentation layer of floetask: the iced user interface.
//!
//! The GUI depends on the application layer (use cases and port traits) and
//! on domain types for display, never on infrastructure. The composition
//! root hands it ready-made [`floetask_application::Services`].

mod app;
mod compose;
mod i18n;
mod theme;
mod view;

#[cfg(test)]
mod tests;

use iced::{Size, window};

pub use app::Startup;

/// Runs the GUI until the window is closed.
pub fn run(startup: Startup) -> iced::Result {
    let window_state = startup.services.ports.state.load().window;
    // `boot` may only run once, but iced wants a reusable function.
    let startup = std::sync::Mutex::new(Some(startup));
    iced::application(
        move || {
            let startup = startup
                .lock()
                .ok()
                .and_then(|mut slot| slot.take())
                .expect("floetask boots once");
            app::Floetask::boot(startup)
        },
        app::Floetask::update,
        app::Floetask::view,
    )
    .title(app::Floetask::title)
    .theme(app::Floetask::theme)
    .scale_factor(app::Floetask::scale_factor)
    .subscription(app::Floetask::subscription)
    .window(window::Settings {
        size: Size::new(window_state.width, window_state.height),
        position: match (window_state.x, window_state.y) {
            (Some(x), Some(y)) => window::Position::Specific(iced::Point::new(x, y)),
            _ => window::Position::Centered,
        },
        maximized: window_state.maximized,
        min_size: Some(Size::new(480.0, 360.0)),
        exit_on_close_request: false,
        ..window::Settings::default()
    })
    .run()
}
