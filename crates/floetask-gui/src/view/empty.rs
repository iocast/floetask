//! Splash screens when there is nothing to list.

use iced::widget::{button, center, column, row, text};
use iced::{Alignment, Element};

use crate::app::{Floetask, Message};
use crate::i18n::tr;

pub fn no_file(_app: &Floetask) -> Element<'_, Message> {
    branded_splash(
        tr("no_file_title"),
        Some(tr("no_file_body")),
        row![
            button(text(tr("open_file")))
                .style(button::primary)
                .on_press(Message::OpenFileDialog),
            button(text(tr("create_file")))
                .style(button::secondary)
                .on_press(Message::CreateFileDialog),
        ]
        .spacing(8)
        .into(),
    )
}

pub fn no_todos(_app: &Floetask) -> Element<'_, Message> {
    splash(
        tr("no_todos_title"),
        None,
        button(text(tr("add_todo")))
            .style(button::primary)
            .on_press(Message::NewTodo)
            .into(),
    )
}

pub fn nothing_visible(_app: &Floetask) -> Element<'_, Message> {
    splash(
        tr("no_visible_title"),
        None,
        button(text(tr("reset_filters")))
            .style(button::primary)
            .on_press(Message::ResetFilters)
            .into(),
    )
}

/// The start screen, with the logo above the message.
fn branded_splash<'a>(title: &'a str, body: Option<&'a str>, actions: Element<'a, Message>) -> Element<'a, Message> {
    center(
        column![crate::logo::view(96.0), splash(title, body, actions)]
            .spacing(20)
            .align_x(Alignment::Center),
    )
    .into()
}

fn splash<'a>(title: &'a str, body: Option<&'a str>, actions: Element<'a, Message>) -> Element<'a, Message> {
    let mut content = column![text(title).size(22)]
        .spacing(14)
        .align_x(Alignment::Center)
        .max_width(460);
    if let Some(body) = body {
        content = content.push(text(body).size(14).align_x(Alignment::Center));
    }
    center(content.push(actions)).into()
}
