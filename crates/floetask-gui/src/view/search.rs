//! The search field in the title bar, the saved-filter list under it and
//! the counts above the list.

use iced::widget::{Column, button, column, container, row, text, text_input};
use iced::{Alignment, Element, Fill};

use super::SEARCH_ID;
use super::icons::{Icon, icon};
use super::widgets::{icon_button, with_tooltip};
use crate::app::{Floetask, Message};
use crate::i18n::{tr, trf};
use crate::theme::{self, Colors};

/// The search field, or `None` when no file is open.
pub fn field(app: &Floetask) -> Option<Element<'_, Message>> {
    app.active_document()?;
    let colors = app.colors();
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
        .padding([6, 0])
        .size(13)
        .style(move |theme, status| text_input::Style {
            border: iced::Border::default(),
            background: iced::Color::TRANSPARENT.into(),
            ..theme::input(colors)(theme, status)
        })
        .width(Fill);
    let mut bar = row![icon(Icon::Search, 15.0, colors.muted), input]
        .spacing(8)
        .align_y(Alignment::Center);
    if !app.search.input.is_empty() {
        bar = bar.push(small(
            Icon::Close,
            tr("clear"),
            Message::SearchInput(String::new()),
            colors,
            false,
        ));
        bar = bar.push(small(
            Icon::Star,
            tr("save_filter"),
            Message::AskSaveFilter,
            colors,
            false,
        ));
    }
    bar = bar.push(small(
        Icon::ChevronDown,
        tr("saved_filters"),
        Message::ToggleSavedFilters,
        colors,
        app.search.saved_open,
    ));
    Some(
        container(bar)
            .padding(iced::padding::left(12).right(4))
            .style(move |_| container::Style {
                background: Some(colors.surface.into()),
                border: iced::Border {
                    radius: 10.0.into(),
                    width: 1.0,
                    color: colors.border,
                },
                ..container::Style::default()
            })
            .into(),
    )
}

fn small<'a>(glyph: Icon, hint: &'a str, message: Message, colors: Colors, active: bool) -> Element<'a, Message> {
    with_tooltip(
        button(icon(glyph, 14.0, if active { colors.primary } else { colors.muted }))
            .padding(5)
            .style(theme::ghost(colors, active))
            .on_press(message),
        hint,
        colors,
    )
}

/// Visible / total / completed counts above the list.
pub fn summary(app: &Floetask) -> Element<'_, Message> {
    let counts = app.listing.counts;
    container(
        text(trf("counts", &[&counts.visible, &counts.total, &counts.completed]))
            .size(12)
            .color(app.colors().muted),
    )
    .padding([0, 4])
    .into()
}

/// The saved filters, shown under the title bar while open.
pub fn saved_panel(app: &Floetask) -> Option<Element<'_, Message>> {
    if !app.search.saved_open {
        return None;
    }
    let colors = app.colors();
    let content: Element<'_, Message> = if app.saved_filters.is_empty() {
        text(tr("no_saved_filters")).size(13).color(colors.muted).into()
    } else {
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
        .into()
    };
    Some(
        container(column![super::widgets::caption(tr("saved_filters"), colors), content].spacing(8))
            .padding(12)
            .width(520)
            .style(theme::card(colors))
            .into(),
    )
}
