//! The drawer: a floating panel with attributes, view filters and sorting.

use iced::widget::{Column, Row, button, column, container, mouse_area, row, rule, scrollable, space, text};
use iced::{Alignment, Element, Fill};

use floetask_application::DrawerTab;
use floetask_domain::listing::{Attribute, AttributeSummary};

use super::icons::{Icon, icon};
use super::list::{attribute_label, status_label};
use super::widgets::{caption, icon_button, switch};
use crate::app::{Floetask, Message, ViewToggle};
use crate::i18n::{tr, trf};
use crate::theme::{self, Colors};

const WIDTH: f32 = 300.0;

pub fn view(app: &Floetask) -> Element<'_, Message> {
    let colors = app.colors();
    let content = match app.state.drawer_tab {
        DrawerTab::Attributes => attributes(app, colors),
        DrawerTab::Filters => filters(app),
        DrawerTab::Sorting => sorting(app, colors),
    };
    container(
        column![
            segmented(app, colors),
            scrollable(container(content).padding(iced::padding::right(12).bottom(12)))
                .direction(theme::thin_scrollbar())
                .style(theme::slim_scroller(colors))
                .height(Fill),
        ]
        .spacing(12),
    )
    .width(WIDTH)
    .height(Fill)
    .padding(iced::padding::top(12).left(12))
    .style(theme::panel(colors))
    .into()
}

/// The three tabs as one segmented control.
fn segmented(app: &Floetask, colors: Colors) -> Element<'_, Message> {
    let segment = |label: &'static str, value: DrawerTab| {
        let active = app.state.drawer_tab == value;
        button(text(tr(label)).size(13).width(Fill).align_x(Alignment::Center))
            .width(Fill)
            .padding([6, 0])
            .style(theme::tab(colors, active))
            .on_press(Message::DrawerTab(value))
    };
    container(
        row![
            segment("attributes", DrawerTab::Attributes),
            segment("filters", DrawerTab::Filters),
            segment("sorting", DrawerTab::Sorting),
        ]
        .spacing(2),
    )
    .padding(3)
    .style(move |_| container::Style {
        background: Some(colors.background.into()),
        border: iced::border::rounded(10),
        ..container::Style::default()
    })
    .width(WIDTH - 24.0)
    .into()
}

fn attributes(app: &Floetask, colors: Colors) -> Element<'_, Message> {
    let mut sections = Column::new().spacing(16);
    if app.state.view.has_active_filters() {
        sections = sections.push(
            button(
                row![
                    icon(Icon::Close, 14.0, colors.primary),
                    text(tr("reset_filters")).size(13)
                ]
                .spacing(6)
                .align_y(Alignment::Center),
            )
            .padding([6, 10])
            .style(theme::ghost(colors, true))
            .on_press(Message::ResetFilters),
        );
    }
    for summary in &app.summaries {
        sections = sections.push(section(app, summary, colors));
    }
    sections
        .push(text(tr("include_hint")).size(11).color(colors.muted))
        .into()
}

fn section<'a>(app: &'a Floetask, summary: &'a AttributeSummary, colors: Colors) -> Element<'a, Message> {
    let attribute = summary.attribute;
    let collapsed = app.state.collapsed_sections.contains(&attribute);
    let hidden = app.state.view.hidden_categories.contains(&attribute);
    let title_color = if summary.has_overdue() {
        colors.danger
    } else {
        colors.muted
    };
    let header = row![
        button(
            row![
                icon(
                    if collapsed {
                        Icon::ChevronRight
                    } else {
                        Icon::ChevronDown
                    },
                    14.0,
                    title_color
                ),
                text(tr(attribute.key()).to_uppercase())
                    .size(11)
                    .color(title_color)
                    .font(iced::Font {
                        weight: iced::font::Weight::Semibold,
                        ..iced::Font::default()
                    }),
            ]
            .spacing(6)
            .align_y(Alignment::Center),
        )
        .padding([4, 4])
        .style(theme::ghost(colors, false))
        .width(Fill)
        .on_press(Message::ToggleSection(attribute)),
        icon_button(
            if hidden { Icon::EyeOff } else { Icon::Eye },
            tr(if hidden { "show_category" } else { "hide_category" }),
            Some(Message::ToggleCategory(attribute)),
            colors,
            hidden,
        ),
    ]
    .align_y(Alignment::Center);
    if collapsed {
        return header.into();
    }

    // Alt or Ctrl while clicking excludes instead of includes.
    let exclude_click = app.modifiers.alt() || app.modifiers.command();
    let values = Row::with_children(summary.values.iter().map(|value| {
        let state = app.state.view.filter_state(attribute, &value.value);
        let color = match state {
            Some(true) => colors.danger,
            _ if value.overdue => colors.danger,
            // Same colours as the chips on the todo cards.
            _ if attribute == Attribute::Contexts => colors.success,
            _ => colors.primary,
        };
        let count_color = if state.is_some() { colors.surface } else { colors.muted };
        let chip = button(
            row![
                text(attribute_label(attribute, &value.value)).size(12),
                text(value.count.to_string()).size(11).color(count_color),
            ]
            .spacing(6)
            .align_y(Alignment::Center),
        )
        .padding([3, 10])
        .style(theme::chip(color, colors, state.is_some()))
        .on_press(Message::FilterValue(attribute, value.value.clone(), exclude_click));
        let on_right = if attribute.is_renamable() {
            Message::AskRename(attribute, value.value.clone())
        } else {
            Message::FilterValue(attribute, value.value.clone(), true)
        };
        mouse_area(chip).on_right_press(on_right).into()
    }))
    .spacing(6)
    .wrap()
    .vertical_spacing(6);
    column![header, values].spacing(8).into()
}

fn filters(app: &Floetask) -> Element<'_, Message> {
    let view = &app.state.view;
    let toggle = |label: &'static str, value: bool, which: ViewToggle| {
        switch(tr(label), value, move |on| Message::ViewToggle(which, on))
    };
    let hidden_statuses: Vec<String> = app.settings.statuses.hidden().iter().map(|s| status_label(s)).collect();
    let mut toggles = column![];
    if !hidden_statuses.is_empty() {
        let label = trf("show_hidden_statuses", &[&hidden_statuses.join(", ").to_lowercase()]);
        toggles = toggles.push(switch(label, view.show_hidden_statuses, |on| {
            Message::ViewToggle(ViewToggle::HiddenStatuses, on)
        }));
    }
    toggles
        .extend([
            toggle("show_completed", view.show_completed, ViewToggle::Completed),
            toggle("show_hidden", view.show_hidden, ViewToggle::Hidden),
            toggle(
                "show_hidden_attributes",
                view.show_hidden_attributes,
                ViewToggle::HiddenAttributes,
            ),
            toggle(
                "show_future_threshold",
                view.show_future_threshold,
                ViewToggle::FutureThreshold,
            ),
            toggle("show_future_due", view.show_future_due, ViewToggle::FutureDue),
        ])
        .spacing(16)
        .into()
}

fn sorting(app: &Floetask, colors: Colors) -> Element<'_, Message> {
    let sorting = &app.state.sorting;
    let last = sorting.criteria.len().saturating_sub(1);
    let criteria = Column::with_children(sorting.criteria.iter().enumerate().map(|(index, criterion)| {
        let grouping = index == 0 && !sorting.file_order;
        let mut label = row![text(tr(criterion.attribute.key())).size(14)]
            .spacing(8)
            .align_y(Alignment::Center);
        if grouping {
            label = label.push(caption(tr("groups"), colors));
        }
        row![
            label,
            space().width(Fill),
            icon_button(
                if criterion.descending {
                    Icon::ArrowDown
                } else {
                    Icon::ArrowUp
                },
                tr("invert"),
                Some(Message::InvertSort(index)),
                colors,
                criterion.descending,
            ),
            icon_button(
                Icon::ChevronUp,
                tr("move_up"),
                (index > 0).then_some(Message::MoveSort(index, -1)),
                colors,
                false
            ),
            icon_button(
                Icon::ChevronDown,
                tr("move_down"),
                (index < last).then_some(Message::MoveSort(index, 1)),
                colors,
                false,
            ),
        ]
        .align_y(Alignment::Center)
        .into()
    }))
    .spacing(2);
    column![
        criteria,
        rule::horizontal(1),
        switch(tr("file_order"), sorting.file_order, Message::FileOrder),
        switch(tr("completed_last"), sorting.completed_last, Message::CompletedLast),
    ]
    .spacing(14)
    .into()
}
