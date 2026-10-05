//! Confirmation, rename, date and save-filter dialogs.

use iced::widget::{button, column, container, row, space, text, text_input};
use iced::{Alignment, Element, Fill};

use floetask_domain::board::{Column, DONE_COLUMN};
use floetask_domain::listing::Attribute;

use super::calendar;
use super::icons::Icon;
use super::list::{attribute_label, status_label};
use super::widgets::{danger_button, icon_button, primary_button, secondary_button, title};
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

/// Edits the board columns of the active file: reorder, remove, add a
/// known status or type a new one.
pub fn board_columns<'a>(app: &'a Floetask, columns: &'a [String], input: &'a str) -> Element<'a, Message> {
    let colors = app.colors();
    let file = app
        .state
        .active_entry()
        .map(|entry| entry.file_name())
        .unwrap_or_default();
    let last = columns.len().saturating_sub(1);
    let rows = column(columns.iter().enumerate().map(|(index, key)| {
        let step = |glyph: Icon, label: &'static str, message: Message, enabled: bool| {
            icon_button(glyph, tr(label), enabled.then_some(message), colors, false)
        };
        row![
            text(column_label(key)).size(14).width(Fill),
            text(key.as_str())
                .size(12)
                .color(colors.muted)
                .font(iced::Font::MONOSPACE),
            step(Icon::ArrowUp, "move_up", Message::BoardColumnMove(index, -1), index > 0),
            step(
                Icon::ArrowDown,
                "move_down",
                Message::BoardColumnMove(index, 1),
                index < last
            ),
            step(
                Icon::Trash,
                "remove_column",
                Message::BoardColumnRemove(index),
                last > 0
            ),
        ]
        .spacing(4)
        .align_y(Alignment::Center)
        .into()
    }))
    .spacing(2);

    let mut known: Vec<String> = app.settings.statuses.names().to_vec();
    known.push(DONE_COLUMN.to_owned());
    let suggestions = row(known.into_iter().filter(|key| !columns.contains(key)).map(|key| {
        button(text(column_label(&key)).size(12))
            .padding([3, 10])
            .style(theme::chip(colors.primary, colors, false))
            .on_press(Message::BoardColumnAdd(Some(key)))
            .into()
    }))
    .spacing(6)
    .wrap();
    let valid = Column::parse(input).is_some() && !columns.iter().any(|key| key == input);
    let add = row![
        text_input(tr("column_placeholder"), input)
            .on_input(Message::BoardColumnInput)
            .on_submit_maybe(valid.then_some(Message::BoardColumnAdd(None)))
            .padding(8)
            .size(13)
            .style(theme::input(colors)),
        secondary_button(tr("add"), Message::BoardColumnAdd(None), colors),
    ]
    .spacing(8)
    .align_y(Alignment::Center);

    column![
        title(trf("board_columns_title", &[&file])),
        text(tr("board_columns_hint")).size(13).color(colors.muted),
        rows,
        suggestions,
        add,
        row![
            secondary_button(tr("reset_columns"), Message::BoardColumnsReset, colors),
            space().width(Fill),
            secondary_button(tr("cancel"), Message::CloseDialog, colors),
            primary_button(tr("save"), Message::ConfirmDialog, colors),
        ]
        .spacing(8)
        .align_y(Alignment::Center),
    ]
    .spacing(16)
    .width(480)
    .into()
}

fn column_label(key: &str) -> String {
    if key == DONE_COLUMN {
        tr("status_done").to_owned()
    } else {
        status_label(key)
    }
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
