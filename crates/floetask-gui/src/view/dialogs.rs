//! Confirmation, rename, date and save-filter dialogs.

use iced::widget::{button, column, container, row, text, text_input};
use iced::{Element, Fill};

use floetask_domain::listing::Attribute;

use super::calendar;
use super::list::attribute_label;
use crate::app::{Calendar, Floetask, Message};
use crate::i18n::{tr, trf};

pub fn confirm<'a>(message: &'a str, detail: Option<&'a str>) -> Element<'a, Message> {
    let mut content = column![text(message).size(16)].spacing(12).width(440);
    if let Some(detail) = detail {
        content = content.push(
            container(text(detail).size(13).font(iced::Font::MONOSPACE))
                .padding(10)
                .width(Fill)
                .style(container::rounded_box),
        );
    }
    content.push(buttons(tr("confirm"), button::danger)).into()
}

pub fn rename<'a>(attribute: Attribute, from: &'a str, input: &'a str) -> Element<'a, Message> {
    let field = text_input("", input)
        .on_input(Message::RenameInput)
        .on_submit(Message::ConfirmDialog)
        .padding(8);
    column![
        text(trf("rename_title", &[&attribute_label(attribute, from)])).size(16),
        field,
        row![
            button(text(tr("remove_value")))
                .style(button::danger)
                .on_press(Message::AskRemoveValue(attribute, from.to_owned())),
            container(buttons(tr("rename"), button::primary)).align_right(Fill),
        ],
    ]
    .spacing(12)
    .width(440)
    .into()
}

pub fn date<'a>(app: &'a Floetask, calendar: &'a Calendar) -> Element<'a, Message> {
    column![
        text(tr(calendar.key.key())).size(16),
        calendar::view(calendar, app.today, app.settings.week_start),
        container(
            button(text(tr("cancel")))
                .style(button::secondary)
                .on_press(Message::CloseDialog)
        )
        .align_right(Fill),
    ]
    .spacing(12)
    .into()
}

pub fn save_filter<'a>(app: &'a Floetask, name: &'a str) -> Element<'a, Message> {
    column![
        text(tr("save_filter")).size(16),
        container(text(&app.search.input).size(13).font(iced::Font::MONOSPACE))
            .padding(8)
            .style(container::rounded_box),
        text_input(tr("filter_name"), name)
            .on_input(Message::SaveFilterName)
            .on_submit(Message::ConfirmDialog)
            .padding(8),
        buttons(tr("save"), button::primary),
    ]
    .spacing(12)
    .width(440)
    .into()
}

fn buttons<'a>(label: &'a str, style: fn(&iced::Theme, button::Status) -> button::Style) -> Element<'a, Message> {
    container(
        row![
            button(text(tr("cancel")))
                .style(button::secondary)
                .on_press(Message::CloseDialog),
            button(text(label)).style(style).on_press(Message::ConfirmDialog),
        ]
        .spacing(8),
    )
    .align_right(Fill)
    .into()
}
