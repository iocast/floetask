//! The search bar above the list, with saved filters and the counts.

use iced::widget::{Column, button, column, container, row, text, text_input};
use iced::{Alignment, Element, Fill};

use super::SEARCH_ID;
use super::icons::{Icon, icon};
use super::widgets::icon_button;
use crate::app::{Floetask, Message};
use crate::i18n::{tr, trf};
use crate::theme::{self, Colors};

/// `None` when there is no file open.
pub fn view(app: &Floetask) -> Option<Element<'_, Message>> {
    app.active_document()?;
    let colors = app.colors();
    let counts = app.listing.counts;
    let summary = text(trf("counts", &[&counts.visible, &counts.total, &counts.completed]))
        .size(12)
        .color(colors.muted);
    if !app.state.search_visible {
        return Some(container(summary).padding([0, 4]).into());
    }

    // Ctrl+Enter turns the search text into a new todo.
    let submit = if app.modifiers.command() {
        Message::SearchSubmitAsTodo
    } else {
        Message::Noop
    };
    let input = text_input(tr("search_placeholder"), &app.search.input)
        .id(SEARCH_ID)
        .on_input(Message::SearchInput)
        .on_submit(submit)
        .padding([10, 0])
        .size(14)
        .style(move |theme, status| text_input::Style {
            border: iced::Border::default(),
            ..theme::input(colors)(theme, status)
        })
        .width(Fill);
    let mut bar = row![icon(Icon::Search, 16.0, colors.muted), input]
        .spacing(10)
        .align_y(Alignment::Center);
    if !app.search.input.is_empty() {
        bar = bar.push(icon_button(
            Icon::Close,
            tr("clear"),
            Some(Message::SearchInput(String::new())),
            colors,
            false,
        ));
    }
    bar = bar
        .push(icon_button(
            Icon::Star,
            tr("save_filter"),
            Some(Message::AskSaveFilter),
            colors,
            false,
        ))
        .push(icon_button(
            Icon::ChevronDown,
            tr("saved_filters"),
            Some(Message::ToggleSavedFilters),
            colors,
            app.search.saved_open,
        ));

    let mut panel = column![bar].spacing(6);
    if app.search.saved_open {
        panel = panel.push(saved_filters(app, colors));
    }
    Some(
        column![
            container(panel).padding([2, 12]).style(theme::panel(colors)),
            container(summary).padding([0, 4]),
        ]
        .spacing(8)
        .into(),
    )
}

fn saved_filters(app: &Floetask, colors: Colors) -> Element<'_, Message> {
    if app.saved_filters.is_empty() {
        return container(text(tr("no_saved_filters")).size(13).color(colors.muted))
            .padding([6, 4])
            .into();
    }
    Column::with_children(app.saved_filters.iter().enumerate().map(|(index, filter)| {
        let bell = if filter.suppress_notifications {
            Icon::BellOff
        } else {
            Icon::Bell
        };
        row![
            button(
                column![
                    text(&filter.name).size(14),
                    text(&filter.query).size(12).color(colors.muted)
                ]
                .spacing(2)
            )
            .padding([6, 8])
            .style(theme::ghost(colors, false))
            .width(Fill)
            .on_press(Message::ApplySavedFilter(index)),
            icon_button(
                bell,
                tr("suppress_notifications"),
                Some(Message::ToggleSuppress(index)),
                colors,
                false
            ),
            icon_button(
                Icon::Trash,
                tr("delete"),
                Some(Message::AskDeleteSavedFilter(index)),
                colors,
                false
            ),
        ]
        .spacing(2)
        .align_y(Alignment::Center)
        .into()
    }))
    .padding(iced::padding::bottom(8))
    .into()
}
