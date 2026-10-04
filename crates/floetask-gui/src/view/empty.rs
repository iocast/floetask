//! Splash screens when there is nothing to list.

use iced::widget::{center, column, row, text};
use iced::{Alignment, Element};

use super::widgets::{primary_button, secondary_button};
use crate::app::{Floetask, Message};
use crate::i18n::tr;

pub fn no_file(app: &Floetask) -> Element<'_, Message> {
    let colors = app.colors();
    splash(
        app,
        Some(crate::logo::view(88.0)),
        tr("no_file_title"),
        Some(tr("no_file_body")),
        row![
            primary_button(tr("open_file"), Message::OpenFileDialog, colors),
            secondary_button(tr("create_file"), Message::CreateFileDialog, colors),
        ]
        .spacing(8)
        .into(),
    )
}

pub fn no_todos(app: &Floetask) -> Element<'_, Message> {
    let colors = app.colors();
    splash(
        app,
        None,
        tr("no_todos_title"),
        Some(tr("no_todos_body")),
        primary_button(tr("add_todo"), Message::NewTodo, colors),
    )
}

pub fn nothing_visible(app: &Floetask) -> Element<'_, Message> {
    let colors = app.colors();
    splash(
        app,
        None,
        tr("no_visible_title"),
        None,
        secondary_button(tr("reset_filters"), Message::ResetFilters, colors),
    )
}

fn splash<'a>(
    app: &'a Floetask,
    image: Option<Element<'a, Message>>,
    title: &'a str,
    body: Option<&'a str>,
    actions: Element<'a, Message>,
) -> Element<'a, Message> {
    let mut content = column![].spacing(14).align_x(Alignment::Center).max_width(440);
    if let Some(image) = image {
        content = content.push(image);
    }
    content = content.push(text(title).size(22));
    if let Some(body) = body {
        content = content.push(text(body).size(14).color(app.colors().muted).align_x(Alignment::Center));
    }
    center(content.push(actions)).into()
}
