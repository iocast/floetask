//! The drawer on the right: attributes, view filters and sorting.

use iced::widget::{Column, Row, button, column, container, mouse_area, row, rule, scrollable, text};
use iced::{Alignment, Element, Fill};

use floetask_application::DrawerTab;
use floetask_domain::listing::AttributeSummary;

use super::list::attribute_label;
use super::widgets::switch;
use crate::app::{Floetask, Message, ViewToggle};
use crate::i18n::tr;
use crate::theme::{self, Colors};

pub fn view(app: &Floetask) -> Element<'_, Message> {
    let tab = |label: &'static str, value: DrawerTab| {
        button(text(tr(label)).size(14))
            .style(if app.state.drawer_tab == value {
                button::primary
            } else {
                button::text
            })
            .on_press(Message::DrawerTab(value))
    };
    let tabs = row![
        tab("attributes", DrawerTab::Attributes),
        tab("filters", DrawerTab::Filters),
        tab("sorting", DrawerTab::Sorting)
    ]
    .spacing(4);
    let content = match app.state.drawer_tab {
        DrawerTab::Attributes => attributes(app),
        DrawerTab::Filters => filters(app),
        DrawerTab::Sorting => sorting(app),
    };
    container(column![tabs, rule::horizontal(1), scrollable(content).height(Fill)].spacing(8))
        .width(320)
        .height(Fill)
        .padding(10)
        .style(theme::navigation(app.colors()))
        .into()
}

fn attributes(app: &Floetask) -> Element<'_, Message> {
    let colors = app.colors();
    let mut sections = Column::new().spacing(10).padding([0, 6]);
    if app.state.view.has_active_filters() {
        sections = sections.push(button(text(tr("reset_filters")).size(13)).on_press(Message::ResetFilters));
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
        colors.text
    };
    let header = row![
        button(
            text(format!("{} {}", if collapsed { "▸" } else { "▾" }, tr(attribute.key())))
                .size(14)
                .color(title_color)
        )
        .style(button::text)
        .width(Fill)
        .on_press(Message::ToggleSection(attribute)),
        iced::widget::tooltip(
            button(text(if hidden { "⊘" } else { "○" }).size(14))
                .style(button::text)
                .on_press(Message::ToggleCategory(attribute)),
            text(tr(if hidden { "show_category" } else { "hide_category" })).size(12),
            iced::widget::tooltip::Position::Left,
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
            _ => colors.primary,
        };
        let chip = button(
            text(format!(
                "{} · {}",
                attribute_label(attribute, &value.value),
                value.count
            ))
            .size(12),
        )
        .padding([2, 8])
        .style(theme::chip(color, colors, state.is_some()))
        .on_press(Message::FilterValue(attribute, value.value.clone(), exclude_click));
        let on_right = if attribute.is_renamable() {
            Message::AskRename(attribute, value.value.clone())
        } else {
            Message::FilterValue(attribute, value.value.clone(), true)
        };
        mouse_area(chip).on_right_press(on_right).into()
    }))
    .spacing(4)
    .wrap();
    column![header, values].spacing(4).into()
}

fn filters(app: &Floetask) -> Element<'_, Message> {
    let view = &app.state.view;
    let toggle = |label: &'static str, value: bool, which: ViewToggle| {
        switch(tr(label), value, move |on| Message::ViewToggle(which, on))
    };
    column![
        toggle("show_completed", view.show_completed, ViewToggle::Completed),
        toggle("show_hidden", view.show_hidden, ViewToggle::Hidden),
        toggle(
            "show_hidden_attributes",
            view.show_hidden_attributes,
            ViewToggle::HiddenAttributes
        ),
        toggle(
            "show_future_threshold",
            view.show_future_threshold,
            ViewToggle::FutureThreshold
        ),
        toggle("show_future_due", view.show_future_due, ViewToggle::FutureDue),
    ]
    .spacing(12)
    .padding([0, 6])
    .into()
}

fn sorting(app: &Floetask) -> Element<'_, Message> {
    let sorting = &app.state.sorting;
    let last = sorting.criteria.len().saturating_sub(1);
    let criteria = Column::with_children(sorting.criteria.iter().enumerate().map(|(index, criterion)| {
        let small = |label: &'static str, message: Option<Message>| {
            button(text(label).size(13))
                .style(button::text)
                .padding([2, 6])
                .on_press_maybe(message)
        };
        let grouping = index == 0 && !sorting.file_order;
        row![
            text(format!(
                "{}{}",
                tr(criterion.attribute.key()),
                if grouping { " ▣" } else { "" }
            ))
            .size(14)
            .width(Fill),
            small(
                if criterion.descending { "↓" } else { "↑" },
                Some(Message::InvertSort(index))
            ),
            small("▲", (index > 0).then_some(Message::MoveSort(index, -1))),
            small("▼", (index < last).then_some(Message::MoveSort(index, 1))),
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
    .spacing(10)
    .padding([0, 6])
    .into()
}
