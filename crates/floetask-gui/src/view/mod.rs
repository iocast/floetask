//! Views: pure functions from application state to widgets.

mod calendar;
mod dialogs;
mod drawer;
mod editor;
mod empty;
mod header;
pub mod list;
mod navigation;
mod settings;
mod widgets;

use iced::widget::{column, container, row, stack, text};
use iced::{Element, Fill, Length};

use crate::app::{Dialog, Floetask, Message};
use crate::theme;

pub const LIST_ID: &str = "todo-list";
pub const SEARCH_ID: &str = "search";
pub const EDITOR_ID: &str = "todo-editor";

pub fn root(app: &Floetask) -> Element<'_, Message> {
    let colors = app.colors();
    let main = column![header::tabs(app), header::search(app), body(app)].height(Fill);
    let mut layout = row![];
    if app.state.navigation_visible {
        layout = layout.push(navigation::view(app));
    }
    layout = layout.push(container(main).width(Fill).height(Fill));
    if app.state.drawer_open {
        layout = layout.push(drawer::view(app));
    }
    let base: Element<'_, Message> = container(layout)
        .width(Fill)
        .height(Fill)
        .style(move |_| iced::widget::container::Style {
            background: Some(colors.background.into()),
            text_color: Some(colors.text),
            ..Default::default()
        })
        .into();

    let with_dialog = match &app.dialog {
        None => base,
        Some(dialog) => {
            let content = match dialog {
                Dialog::Editor(state) => editor::view(app, state),
                Dialog::Settings => settings::view(app),
                Dialog::Confirm { message, detail, .. } => dialogs::confirm(message, detail.as_deref()),
                Dialog::Rename { attribute, from, input } => dialogs::rename(*attribute, from, input),
                Dialog::RowDate { calendar, .. } => dialogs::date(app, calendar),
                Dialog::SaveFilter { name } => dialogs::save_filter(app, name),
            };
            widgets::modal(base, content, colors)
        }
    };

    if app.toasts.is_empty() {
        return with_dialog;
    }
    let toasts = column(app.toasts.iter().map(|toast| {
        container(text(&toast.text).size(14))
            .padding([8, 14])
            .style(theme::toast(colors, toast.error))
            .into()
    }))
    .spacing(6);
    stack![
        with_dialog,
        container(toasts)
            .width(Fill)
            .height(Fill)
            .padding(16)
            .align_bottom(Fill)
            .align_right(Length::Fill)
    ]
    .into()
}

fn body(app: &Floetask) -> Element<'_, Message> {
    match app.active_document() {
        None => empty::no_file(app),
        Some(document) if document.todo_count() == 0 => empty::no_todos(app),
        Some(_) if app.listing.counts.visible == 0 => empty::nothing_visible(app),
        Some(_) => list::view(app),
    }
}
