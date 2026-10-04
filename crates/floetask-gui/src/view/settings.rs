//! The settings dialog.

use iced::widget::{column, container, pick_list, row, scrollable, slider, text};
use iced::{Alignment, Element, Fill};

use floetask_application::{Settings, ThemePreference};
use floetask_domain::WeekStart;

use super::widgets::{caption, primary_button, switch, title};
use crate::app::{Floetask, Message, SettingChange};
use crate::i18n::{LANGUAGES, tr, trf};

pub fn view<'a>(app: &'a Floetask) -> Element<'a, Message> {
    let s = &app.settings;
    let set = |change: fn(bool) -> SettingChange| move |on| Message::Setting(change(on));

    let toggles = column![
        switch(
            tr("append_creation_date"),
            s.append_creation_date,
            set(SettingChange::AppendCreationDate)
        ),
        switch(
            tr("convert_relative_dates"),
            s.convert_relative_dates,
            set(SettingChange::ConvertRelativeDates)
        ),
        switch(
            tr("human_friendly_dates"),
            s.human_friendly_dates,
            set(SettingChange::HumanFriendlyDates)
        ),
        switch(tr("bulk_creation"), s.bulk_creation, set(SettingChange::BulkCreation)),
        switch(tr("safe_writes"), s.safe_writes, set(SettingChange::SafeWrites)),
        switch(tr("compact"), s.compact, set(SettingChange::Compact)),
        switch(
            tr("disable_animations"),
            s.disable_animations,
            set(SettingChange::DisableAnimations)
        ),
    ]
    .spacing(12);

    let threshold_row = row![
        text(trf("notification_threshold", &[&s.notification_threshold_days]))
            .size(14)
            .width(Fill),
        slider(
            0..=Settings::NOTIFICATION_THRESHOLD_MAX,
            s.notification_threshold_days,
            |days| { Message::Setting(SettingChange::NotificationThreshold(days)) }
        )
        .width(180),
    ]
    .align_y(Alignment::Center);
    let threshold = column![
        switch(tr("notifications"), s.notifications, set(SettingChange::Notifications)),
        threshold_row,
    ]
    .spacing(12);

    let zoom = row![
        text(trf("zoom", &[&s.zoom_percent])).size(14).width(Fill),
        slider(Settings::ZOOM_RANGE, s.zoom_percent, |percent| Message::Setting(
            SettingChange::Zoom(percent)
        ))
        .step(10u16)
        .width(180),
    ]
    .align_y(Alignment::Center);

    let choices = column![
        choice(
            tr("theme"),
            &[ThemePreference::System, ThemePreference::Light, ThemePreference::Dark],
            s.theme,
            |theme| tr(match theme {
                ThemePreference::System => "theme_system",
                ThemePreference::Light => "theme_light",
                ThemePreference::Dark => "theme_dark",
            }),
            |theme| Message::Setting(SettingChange::Theme(theme)),
        ),
        choice(
            tr("week_start"),
            &[WeekStart::Monday, WeekStart::Saturday, WeekStart::Sunday],
            s.week_start,
            |day| tr(match day {
                WeekStart::Monday => "monday",
                WeekStart::Saturday => "saturday",
                WeekStart::Sunday => "sunday",
            }),
            |day| Message::Setting(SettingChange::WeekStart(day)),
        ),
        choice(
            tr("language"),
            &LANGUAGES,
            LANGUAGES.into_iter().find(|l| *l == s.language).unwrap_or("system"),
            |language| tr(if language == "system" {
                "language_system"
            } else {
                "language_en"
            }),
            |language| Message::Setting(SettingChange::Language(language.to_owned())),
        ),
    ]
    .spacing(10);

    let colors = app.colors();
    let location = text(trf(
        "config_location",
        &[&app.services.ports.settings.location().display()],
    ))
    .size(12)
    .color(colors.muted);

    let section = |label: &'static str, body: Element<'a, Message>| -> Element<'a, Message> {
        column![
            caption(tr(label), colors),
            container(body)
                .padding(14)
                .width(Fill)
                .style(crate::theme::panel(colors)),
        ]
        .spacing(8)
        .into()
    };

    column![
        title(tr("settings_title")),
        scrollable(
            column![
                section("section_todos", toggles.into()),
                section("section_notifications", threshold.into()),
                section("section_appearance", column![choices, zoom].spacing(12).into()),
            ]
            .spacing(18)
            .padding(iced::padding::right(12)),
        )
        .height(iced::Length::Fixed(460.0)),
        row![
            location.width(Fill),
            primary_button(tr("close"), Message::CloseDialog, colors)
        ]
        .spacing(8)
        .align_y(Alignment::Center),
    ]
    .spacing(16)
    .width(560)
    .into()
}

/// A labelled pick list over a few values shown through `label`.
fn choice<'a, T: Copy + PartialEq + 'a>(
    title: &'a str,
    options: &[T],
    selected: T,
    label: fn(T) -> &'static str,
    on_select: impl Fn(T) -> Message + 'a,
) -> Element<'a, Message> {
    let entries: Vec<Labelled<T>> = options
        .iter()
        .map(|value| Labelled {
            value: *value,
            label: label(*value),
        })
        .collect();
    let current = Labelled {
        value: selected,
        label: label(selected),
    };
    row![
        text(title).size(14).width(Fill),
        pick_list(entries, Some(current), move |entry: Labelled<T>| on_select(entry.value)).width(180),
    ]
    .align_y(Alignment::Center)
    .into()
}

#[derive(Debug, Clone, Copy)]
struct Labelled<T> {
    value: T,
    label: &'static str,
}

impl<T: PartialEq> PartialEq for Labelled<T> {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl<T> std::fmt::Display for Labelled<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.label)
    }
}
