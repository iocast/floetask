//! Views: pure functions from application state to widgets.
//!
//! Layout: a custom title bar on top, then the list (with the search bar
//! above it) and the drawer as a floating panel on the right. Dialogs are
//! modal cards; toasts sit bottom-right.

mod calendar;
mod dialogs;
mod drawer;
mod editor;
mod empty;
mod files;
pub mod icons;
pub mod list;
mod search;
mod settings;
mod title_bar;
mod widgets;

use iced::widget::{column, container, mouse_area, row, space, stack, text};
use iced::{Element, Fill, Length, mouse, window};

use crate::app::{Dialog, Floetask, Message};
use crate::theme;

pub const LIST_ID: &str = "todo-list";
pub const SEARCH_ID: &str = "search";
pub const EDITOR_ID: &str = "todo-editor";

/// Width of the invisible window-resize border.
const EDGE: f32 = 5.0;

pub fn root(app: &Floetask) -> Element<'_, Message> {
    let colors = app.colors();

    let mut main = column![].spacing(10).width(Fill).height(Fill);
    if app.active_document().is_some() {
        main = main.push(search::summary(app));
    }
    main = main.push(body(app));

    let mut content = row![].spacing(12).height(Fill);
    if app.state.files_drawer_open {
        content = content.push(files::drawer(app));
    }
    content = content.push(main);
    if app.state.drawer_open && app.active_document().is_some() {
        content = content.push(drawer::view(app));
    }

    let mut page = column![title_bar::view(app)].spacing(8);
    if let Some(tabs) = files::tabs(app) {
        page = page.push(tabs);
    }
    page = page.push(
        container(content)
            .padding(iced::padding::left(16).right(16).bottom(16))
            .height(Fill),
    );
    let page: Element<'_, Message> = match search::saved_panel(app) {
        // Saved filters float under the centred search field.
        Some(panel) => stack![
            page,
            container(panel)
                .center_x(Fill)
                .padding(iced::padding::top(title_bar::HEIGHT))
        ]
        .into(),
        None => page.into(),
    };

    let base: Element<'_, Message> = container(page)
        .width(Fill)
        .height(Fill)
        .style(theme::app(colors))
        .into();

    let main_layer = match &app.dialog {
        None => base,
        Some(dialog) => {
            let content = match dialog {
                Dialog::Editor(state) => editor::view(app, state),
                Dialog::Settings => settings::view(app),
                Dialog::Confirm { message, detail, .. } => dialogs::confirm(app, message, detail.as_deref()),
                Dialog::Rename { attribute, from, input } => dialogs::rename(app, *attribute, from, input),
                Dialog::RowDate { calendar, .. } => dialogs::date(app, calendar),
                Dialog::SaveFilter { name } => dialogs::save_filter(app, name),
            };
            widgets::modal(base, content, colors)
        }
    };
    let mut layers = stack![main_layer];

    if !app.toasts.is_empty() {
        let toasts = column(app.toasts.iter().map(|toast| {
            container(text(&toast.text).size(14))
                .padding([10, 16])
                .style(theme::toast(colors, toast.error))
                .into()
        }))
        .spacing(8);
        layers = layers.push(
            container(toasts)
                .width(Fill)
                .height(Fill)
                .padding(20)
                .align_bottom(Fill)
                .align_right(Fill),
        );
    }
    if !app.maximized {
        layers = layers.push(resize_edges());
    }
    layers.into()
}

fn body(app: &Floetask) -> Element<'_, Message> {
    match app.active_document() {
        None => empty::no_file(app),
        Some(document) if document.todo_count() == 0 => empty::no_todos(app),
        Some(_) if app.listing.counts.visible == 0 => empty::nothing_visible(app),
        Some(_) => list::view(app),
    }
}

/// Thin areas along the window border that resize the borderless window.
fn resize_edges() -> Element<'static, Message> {
    use window::Direction as D;
    let edge = |direction: D, width: Length, height: Length| -> Element<'static, Message> {
        let cursor = match direction {
            D::North | D::South => mouse::Interaction::ResizingVertically,
            D::East | D::West => mouse::Interaction::ResizingHorizontally,
            D::NorthWest | D::SouthEast => mouse::Interaction::ResizingDiagonallyDown,
            D::NorthEast | D::SouthWest => mouse::Interaction::ResizingDiagonallyUp,
        };
        mouse_area(space().width(width).height(height))
            .on_press(Message::WindowResize(direction))
            .interaction(cursor)
            .into()
    };
    let e = Length::Fixed(EDGE);
    column![
        row![
            edge(D::NorthWest, e, e),
            edge(D::North, Fill, e),
            edge(D::NorthEast, e, e)
        ],
        row![
            edge(D::West, e, Fill),
            space().width(Fill).height(Fill),
            edge(D::East, e, Fill)
        ]
        .height(Fill),
        row![
            edge(D::SouthWest, e, e),
            edge(D::South, Fill, e),
            edge(D::SouthEast, e, e)
        ],
    ]
    .into()
}
