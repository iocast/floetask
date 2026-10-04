//! File tabs and the search bar.

use iced::widget::{button, column, container, row, space, text, text_input};
use iced::{Alignment, Element, Fill};

use super::SEARCH_ID;
use super::widgets::link_button;
use crate::app::{Floetask, Message};
use crate::i18n::{tr, trf};
use crate::theme;

/// One tab per registered file, shown when there is more than one. A tab's
/// "⋯" opens its menu: done file, reveal, remove.
pub fn tabs(app: &Floetask) -> Element<'_, Message> {
    if !app.state.tabs_visible || app.state.files.len() < 2 {
        return space().into();
    }
    let colors = app.colors();
    let tabs = row(app.state.files.iter().enumerate().map(|(index, entry)| {
        let active = index == app.state.active_file;
        let label = button(text(entry.file_name()).size(14))
            .style(if active { button::primary } else { button::text })
            .on_press(Message::SelectFile(index));
        let menu_open = app.tab_menu == Some(index);
        let menu = button(text("⋯").size(14))
            .style(button::text)
            .on_press(Message::TabMenu(if menu_open { None } else { Some(index) }));
        row![label, menu].align_y(Alignment::Center).into()
    }))
    .spacing(4);

    let mut bar = column![container(tabs).padding([6, 10])];
    if let Some(index) = app.tab_menu {
        bar = bar.push(
            container(
                row![
                    link_button(tr("change_done_file"), Message::ChangeDoneFile(index)),
                    link_button(tr("reveal_todo_file"), Message::RevealTodoFile(index)),
                    link_button(tr("reveal_done_file"), Message::RevealDoneFile(index)),
                    link_button(tr("remove_file"), Message::AskRemoveFile(index)),
                ]
                .spacing(6),
            )
            .padding([0, 10]),
        );
    }
    container(bar).width(Fill).style(theme::navigation(colors)).into()
}

/// The search bar, saved filters, and the visible / total counts.
pub fn search(app: &Floetask) -> Element<'_, Message> {
    let counts = app.listing.counts;
    let summary = text(trf("counts", &[&counts.visible, &counts.total, &counts.completed]))
        .size(13)
        .color(app.colors().muted);
    if !app.state.search_visible || app.active_document().is_none() {
        return container(summary).padding([6, 16]).into();
    }

    let modifiers = app.modifiers;
    let input = text_input(tr("search_placeholder"), &app.search.input)
        .id(SEARCH_ID)
        .on_input(Message::SearchInput)
        // Ctrl+Enter turns the search text into a new todo.
        .on_submit(if modifiers.command() {
            Message::SearchSubmitAsTodo
        } else {
            Message::Noop
        })
        .padding(8)
        .width(Fill);
    let bar = row![
        input,
        button(text("☆").size(16))
            .style(button::text)
            .on_press(Message::AskSaveFilter),
        button(text("▾").size(16))
            .style(button::text)
            .on_press(Message::ToggleSavedFilters),
    ]
    .spacing(4)
    .align_y(Alignment::Center);

    let mut content = column![bar, summary].spacing(4);
    if app.search.saved_open {
        content = content.push(saved_filters(app));
    }
    container(content).padding([8, 16]).into()
}

fn saved_filters(app: &Floetask) -> Element<'_, Message> {
    if app.saved_filters.is_empty() {
        return text(tr("no_saved_filters")).size(13).into();
    }
    column(app.saved_filters.iter().enumerate().map(|(index, filter)| {
        let bell = if filter.suppress_notifications { "🔕" } else { "🔔" };
        row![
            button(column![text(&filter.name).size(14), text(&filter.query).size(12)])
                .style(button::text)
                .width(Fill)
                .on_press(Message::ApplySavedFilter(index)),
            iced::widget::tooltip(
                button(text(bell).size(14))
                    .style(button::text)
                    .on_press(Message::ToggleSuppress(index)),
                text(tr("suppress_notifications")).size(12),
                iced::widget::tooltip::Position::Left,
            ),
            button(text("✕").size(14))
                .style(button::text)
                .on_press(Message::AskDeleteSavedFilter(index)),
        ]
        .align_y(Alignment::Center)
        .into()
    }))
    .spacing(2)
    .into()
}
