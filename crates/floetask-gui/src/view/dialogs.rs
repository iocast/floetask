//! Confirmation, rename, date and save-filter dialogs.

use iced::widget::{column, container, row, space, text, text_input};
use iced::{Alignment, Element, Fill};

use floetask_domain::listing::Attribute;

use super::calendar;
use super::list::attribute_label;
use super::widgets::{danger_button, primary_button, secondary_button, title};
use crate::app::{Calendar, Floetask, Message};
use crate::i18n::{tr, trf};
use crate::theme::{self, Colors};

pub fn confirm<'a>(app: &'a Floetask, message: &'a str, detail: Option<&'a str>) -> Element<'a, Message> {
    let colors = app.colors();
    let mut content = column![title(message)].spacing(16).width(460);
    if let Some(detail) = detail {
        content = content.push(raw_text(detail, colors));
    }
    content
        .push(buttons(
            danger_button(tr("confirm"), Message::ConfirmDialog, colors),
            colors,
        ))
        .into()
}

pub fn rename<'a>(app: &'a Floetask, attribute: Attribute, from: &'a str, input: &'a str) -> Element<'a, Message> {
    let colors = app.colors();
    let field = text_input("", input)
        .on_input(Message::RenameInput)
        .on_submit(Message::ConfirmDialog)
        .padding(10)
        .style(theme::input(colors));
    column![
        title(trf("rename_title", &[&attribute_label(attribute, from)])),
        field,
        row![
            danger_button(
                tr("remove_value"),
                Message::AskRemoveValue(attribute, from.to_owned()),
                colors
            ),
            space().width(Fill),
            secondary_button(tr("cancel"), Message::CloseDialog, colors),
            primary_button(tr("rename"), Message::ConfirmDialog, colors),
        ]
        .spacing(8)
        .align_y(Alignment::Center),
    ]
    .spacing(16)
    .width(460)
    .into()
}

pub fn date<'a>(app: &'a Floetask, calendar: &'a Calendar) -> Element<'a, Message> {
    let colors = app.colors();
    column![
        title(tr(calendar.key.key())),
        calendar::view(calendar, app.today, app.settings.week_start, colors),
    ]
    .spacing(16)
    .into()
}

pub fn save_filter<'a>(app: &'a Floetask, name: &'a str) -> Element<'a, Message> {
    let colors = app.colors();
    column![
        title(tr("save_filter")),
        raw_text(&app.search.input, colors),
        text_input(tr("filter_name"), name)
            .on_input(Message::SaveFilterName)
            .on_submit(Message::ConfirmDialog)
            .padding(10)
            .style(theme::input(colors)),
        buttons(primary_button(tr("save"), Message::ConfirmDialog, colors), colors),
    ]
    .spacing(16)
    .width(460)
    .into()
}

fn raw_text(content: &str, colors: Colors) -> Element<'_, Message> {
    container(text(content).size(13).font(iced::Font::MONOSPACE))
        .padding(12)
        .width(Fill)
        .style(theme::code(colors))
        .into()
}

/// Cancel plus the dialog's main action, right-aligned.
fn buttons<'a>(action: Element<'a, Message>, colors: Colors) -> Element<'a, Message> {
    container(row![secondary_button(tr("cancel"), Message::CloseDialog, colors), action].spacing(8))
        .align_right(Fill)
        .into()
}
