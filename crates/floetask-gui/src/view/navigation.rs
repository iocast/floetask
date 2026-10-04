//! The vertical icon bar on the left.

use iced::widget::{column, container, space};
use iced::{Element, Fill};

use super::widgets::icon_button;
use crate::app::{Floetask, Message};
use crate::i18n::tr;
use crate::theme;

pub fn view(app: &Floetask) -> Element<'_, Message> {
    let has_file = app.active_document().is_some();
    let can_archive =
        app.state.active_entry().is_some_and(|entry| entry.done_path.is_some()) && app.listing.counts.completed > 0;

    let mut items = column![
        icon_button("+", tr("add_todo"), has_file.then_some(Message::NewTodo)),
        icon_button("≡", tr("toggle_drawer"), has_file.then_some(Message::ToggleDrawer)),
        icon_button("⌕", tr("search"), has_file.then_some(Message::ToggleSearch)),
    ]
    .spacing(4);
    if can_archive {
        items = items.push(icon_button("⇩", tr("archive"), Some(Message::ArchiveCompleted)));
    }
    items = items
        .push(space::vertical())
        .push(icon_button("▤", tr("open_file"), Some(Message::OpenFileDialog)))
        .push(icon_button("⚙", tr("settings"), Some(Message::OpenSettings)));

    container(items.height(Fill))
        .padding(6)
        .height(Fill)
        .style(theme::navigation(app.colors()))
        .into()
}
