//! The confirmation dialog and pieces the features' dialogs share.

use iced::widget::{column, container, row, text};
use iced::{Element, Fill};

use super::widgets::{danger_button, secondary_button, title};
use crate::app::{Floetask, Message};
use crate::i18n::tr;
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

pub(crate) fn raw_text(content: &str, colors: Colors) -> Element<'_, Message> {
    container(text(content).size(13).font(iced::Font::MONOSPACE))
        .padding(12)
        .width(Fill)
        .style(theme::code(colors))
        .into()
}

/// Cancel plus the dialog's main action, right-aligned.
pub(crate) fn buttons<'a>(action: Element<'a, Message>, colors: Colors) -> Element<'a, Message> {
    container(row![secondary_button(tr("cancel"), Message::CloseDialog, colors), action].spacing(8))
        .align_right(Fill)
        .into()
}
