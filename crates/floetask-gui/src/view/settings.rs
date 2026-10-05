//! The settings dialog: a sidebar of sections on the left, the settings of
//! the chosen section on the right. Every setting has a short grey
//! description under its name.

use iced::widget::{
    Column, button, checkbox, column, container, pick_list, row, rule, scrollable, slider, space, text, text_input,
    toggler,
};
use iced::{Alignment, Element, Fill};

use floetask_application::{Settings, ThemePreference};
use floetask_domain::WeekStart;
use floetask_domain::status::{StatusSet, is_valid_status};

use super::icons::Icon;
use super::list::status_label;
use super::widgets::{icon_button, primary_button, title};
use crate::app::{Floetask, Message, SettingChange, SettingsDialog, SettingsSection, UpdateState};
use crate::i18n::{LANGUAGES, tr, trf};
use crate::theme::{self, Colors};

const SIDEBAR_WIDTH: f32 = 180.0;
const CONTROL_WIDTH: f32 = 180.0;

pub fn view<'a>(app: &'a Floetask, dialog: &'a SettingsDialog) -> Element<'a, Message> {
    let colors = app.colors();
    let content = match dialog.section {
        SettingsSection::Todos => todos(app, colors),
        SettingsSection::Dates => dates(app, colors),
        SettingsSection::Statuses => statuses(app, &dialog.status_input, colors),
        SettingsSection::Appearance => appearance(app, colors),
        SettingsSection::Notifications => notifications(app, colors),
        SettingsSection::Files => files(app, colors),
        SettingsSection::About => about(app, colors),
    };
    let body = row![
        sidebar(dialog.section, colors),
        rule::vertical(1).style(move |_| rule::Style {
            color: colors.border,
            radius: 0.0.into(),
            fill_mode: rule::FillMode::Full,
            snap: true,
        }),
        scrollable(container(content).padding(iced::padding::right(14).left(6)))
            .direction(theme::thin_scrollbar())
            .style(theme::slim_scroller(colors))
            .width(Fill)
            .height(Fill),
    ]
    .spacing(16)
    .height(Fill);

    column![
        title(tr("settings_title")),
        body,
        container(primary_button(tr("close"), Message::CloseDialog, colors)).align_right(Fill),
    ]
    .spacing(16)
    .width(Fill)
    .height(Fill)
    .into()
}

fn sidebar(current: SettingsSection, colors: Colors) -> Element<'static, Message> {
    Column::with_children(SettingsSection::ALL.into_iter().map(|section| {
        button(text(tr(section.key())).size(14))
            .width(Fill)
            .padding([8, 12])
            .style(theme::ghost(colors, section == current))
            .on_press(Message::SettingsSection(section))
            .into()
    }))
    .spacing(2)
    .width(SIDEBAR_WIDTH)
    .into()
}

/// One setting: its name with a grey description underneath on the left,
/// the control on the right. The description key is `<key>_desc`.
fn setting<'a>(key: &str, label: String, control: Element<'a, Message>, colors: Colors) -> Element<'a, Message> {
    row![
        column![
            text(label).size(14),
            text(tr(&format!("{key}_desc")).to_owned()).size(12).color(colors.muted),
        ]
        .spacing(3)
        .width(Fill),
        control,
    ]
    .spacing(16)
    .align_y(Alignment::Center)
    .into()
}

fn switch_setting<'a>(
    key: &'static str,
    value: bool,
    change: fn(bool) -> SettingChange,
    colors: Colors,
) -> Element<'a, Message> {
    let control = toggler(value).on_toggle(move |on| Message::Setting(change(on))).into();
    setting(key, tr(key).to_owned(), control, colors)
}

/// The settings of one section, under a heading.
fn page<'a>(heading: &'static str, items: Vec<Element<'a, Message>>) -> Element<'a, Message> {
    column![text(tr(heading)).size(18), Column::with_children(items).spacing(20)]
        .spacing(18)
        .into()
}

fn todos(app: &Floetask, colors: Colors) -> Element<'_, Message> {
    let s = &app.settings;
    page(
        "section_todos",
        vec![
            switch_setting(
                "append_creation_date",
                s.append_creation_date,
                SettingChange::AppendCreationDate,
                colors,
            ),
            switch_setting("bulk_creation", s.bulk_creation, SettingChange::BulkCreation, colors),
        ],
    )
}

fn dates(app: &Floetask, colors: Colors) -> Element<'_, Message> {
    let s = &app.settings;
    let week_start = choice(
        &[WeekStart::Monday, WeekStart::Saturday, WeekStart::Sunday],
        s.week_start,
        |day| {
            tr(match day {
                WeekStart::Monday => "monday",
                WeekStart::Saturday => "saturday",
                WeekStart::Sunday => "sunday",
            })
        },
        |day| Message::Setting(SettingChange::WeekStart(day)),
    );
    page(
        "section_dates",
        vec![
            switch_setting(
                "convert_relative_dates",
                s.convert_relative_dates,
                SettingChange::ConvertRelativeDates,
                colors,
            ),
            switch_setting(
                "human_friendly_dates",
                s.human_friendly_dates,
                SettingChange::HumanFriendlyDates,
                colors,
            ),
            setting("week_start", tr("week_start").to_owned(), week_start, colors),
        ],
    )
}

/// Global status management: order, hidden by default, custom statuses.
fn statuses<'a>(app: &'a Floetask, input: &'a str, colors: Colors) -> Element<'a, Message> {
    let set = &app.settings.statuses;
    let last = set.names().len().saturating_sub(1);
    let rows = set.names().iter().enumerate().map(|(index, status)| {
        let built_in = StatusSet::is_built_in(status);
        let description = if built_in {
            tr(&format!("status_{status}_desc")).to_owned()
        } else {
            tr("status_custom_desc").to_owned()
        };
        let step = |glyph: Icon, label: &'static str, change: SettingChange, enabled: bool| {
            icon_button(
                glyph,
                tr(label),
                enabled.then_some(Message::Setting(change)),
                colors,
                false,
            )
        };
        // `todo` is what a todo without a tag has, so it cannot be hidden.
        let hidden: Element<'a, Message> = if status == "todo" {
            space().width(84).into()
        } else {
            let name = status.clone();
            checkbox(set.is_hidden(status))
                .label(tr("status_hidden"))
                .on_toggle(move |on| Message::Setting(SettingChange::StatusHidden(name.clone(), on)))
                .text_size(13)
                .width(84)
                .into()
        };
        row![
            column![
                row![
                    text(status_label(status)).size(14),
                    text(status.as_str())
                        .size(12)
                        .color(colors.muted)
                        .font(iced::Font::MONOSPACE),
                ]
                .spacing(8)
                .align_y(Alignment::Center),
                text(description).size(12).color(colors.muted),
            ]
            .spacing(3)
            .width(Fill),
            hidden,
            step(
                Icon::ArrowUp,
                "move_up",
                SettingChange::StatusMove(index, -1),
                index > 0
            ),
            step(
                Icon::ArrowDown,
                "move_down",
                SettingChange::StatusMove(index, 1),
                index < last
            ),
            // Built-in statuses can be reordered and hidden, not removed.
            if built_in {
                space().width(34).into()
            } else {
                step(
                    Icon::Trash,
                    "remove_status",
                    SettingChange::StatusRemove(status.clone()),
                    true,
                )
            },
        ]
        .spacing(4)
        .align_y(Alignment::Center)
        .into()
    });
    let valid = is_valid_status(input) && !set.names().iter().any(|name| name == input);
    let add_message = Message::Setting(SettingChange::StatusAdd(input.to_owned()));
    let add = row![
        text_input(tr("column_placeholder"), input)
            .on_input(Message::SettingsStatusInput)
            .on_submit_maybe(valid.then(|| add_message.clone()))
            .padding(8)
            .size(13)
            .style(theme::input(colors)),
        button(text(tr("add_status")).size(14))
            .padding([8, 14])
            .style(theme::secondary(colors))
            .on_press_maybe(valid.then_some(add_message)),
    ]
    .spacing(8)
    .align_y(Alignment::Center);
    page(
        "section_statuses",
        vec![
            text(tr("statuses_desc")).size(12).color(colors.muted).into(),
            Column::with_children(rows).spacing(14).into(),
            add.into(),
        ],
    )
}

fn appearance(app: &Floetask, colors: Colors) -> Element<'_, Message> {
    let s = &app.settings;
    let theme_choice = choice(
        &[ThemePreference::System, ThemePreference::Light, ThemePreference::Dark],
        s.theme,
        |theme| {
            tr(match theme {
                ThemePreference::System => "theme_system",
                ThemePreference::Light => "theme_light",
                ThemePreference::Dark => "theme_dark",
            })
        },
        |theme| Message::Setting(SettingChange::Theme(theme)),
    );
    let language = choice(
        &LANGUAGES,
        LANGUAGES.into_iter().find(|l| *l == s.language).unwrap_or("system"),
        |language| {
            tr(if language == "system" {
                "language_system"
            } else {
                "language_en"
            })
        },
        |language| Message::Setting(SettingChange::Language(language.to_owned())),
    );
    let zoom = slider(Settings::ZOOM_RANGE, s.zoom_percent, |percent| {
        Message::Setting(SettingChange::Zoom(percent))
    })
    .step(10u16)
    .width(CONTROL_WIDTH)
    .into();
    page(
        "section_appearance",
        vec![
            setting("theme", tr("theme").to_owned(), theme_choice, colors),
            setting("language", tr("language").to_owned(), language, colors),
            setting("zoom", trf("zoom", &[&s.zoom_percent]), zoom, colors),
            switch_setting("compact", s.compact, SettingChange::Compact, colors),
            switch_setting(
                "disable_animations",
                s.disable_animations,
                SettingChange::DisableAnimations,
                colors,
            ),
        ],
    )
}

fn notifications(app: &Floetask, colors: Colors) -> Element<'_, Message> {
    let s = &app.settings;
    let threshold = slider(
        0..=Settings::NOTIFICATION_THRESHOLD_MAX,
        s.notification_threshold_days,
        |days| Message::Setting(SettingChange::NotificationThreshold(days)),
    )
    .width(CONTROL_WIDTH)
    .into();
    page(
        "section_notifications",
        vec![
            switch_setting("notifications", s.notifications, SettingChange::Notifications, colors),
            setting(
                "notification_threshold",
                trf("notification_threshold", &[&s.notification_threshold_days]),
                threshold,
                colors,
            ),
        ],
    )
}

fn files(app: &Floetask, colors: Colors) -> Element<'_, Message> {
    let location = app.services.ports.settings.location().display().to_string();
    let location = text(location)
        .size(12)
        .font(iced::Font::MONOSPACE)
        .color(colors.muted)
        .into();
    page(
        "section_files",
        vec![
            switch_setting(
                "safe_writes",
                app.settings.safe_writes,
                SettingChange::SafeWrites,
                colors,
            ),
            setting("config_file", tr("config_file").to_owned(), location, colors),
        ],
    )
}

/// Name, version and the update check.
fn about(app: &Floetask, colors: Colors) -> Element<'_, Message> {
    let identity = row![
        crate::logo::view(72.0),
        column![
            text("floetask").size(26),
            text(trf("about_version", &[&env!("CARGO_PKG_VERSION")]))
                .size(14)
                .color(colors.muted),
            text(tr("about_tagline")).size(14),
        ]
        .spacing(4),
    ]
    .spacing(20)
    .align_y(Alignment::Center);

    let configured = app.services.ports.updater.is_configured();
    let status: Element<'_, Message> = if !configured {
        text(tr("updates_not_configured")).size(13).color(colors.muted).into()
    } else {
        let line = |label: String, color| text(label).size(13).color(color);
        match &app.update {
            UpdateState::Idle => row![primary_button(tr("check_now"), Message::CheckForUpdates, colors)].into(),
            UpdateState::Checking => line(tr("update_checking").to_owned(), colors.muted).into(),
            UpdateState::UpToDate => row![
                line(tr("update_up_to_date").to_owned(), colors.success),
                space::horizontal(),
                primary_button(tr("check_now"), Message::CheckForUpdates, colors),
            ]
            .align_y(Alignment::Center)
            .into(),
            UpdateState::Available(update) => {
                let mut details = column![line(trf("update_available", &[&update.version]), colors.primary)].spacing(6);
                if let Some(notes) = &update.notes {
                    details = details.push(text(notes.clone()).size(12).color(colors.muted));
                }
                row![
                    details.width(Fill),
                    primary_button(tr("update_install"), Message::InstallUpdate, colors)
                ]
                .spacing(16)
                .align_y(Alignment::Center)
                .into()
            }
            UpdateState::Installing => line(tr("update_installing").to_owned(), colors.muted).into(),
            UpdateState::Failed(error) => row![
                line(trf("update_failed", &[error]), colors.danger).width(Fill),
                primary_button(tr("check_now"), Message::CheckForUpdates, colors),
            ]
            .spacing(16)
            .align_y(Alignment::Center)
            .into(),
        }
    };

    let mut items = vec![identity.into(), status];
    if configured {
        items.push(switch_setting(
            "check_for_updates",
            app.settings.check_for_updates,
            SettingChange::CheckForUpdates,
            colors,
        ));
    }
    page("section_about", items)
}

/// A pick list over a few values shown through `label`.
fn choice<'a, T: Copy + PartialEq + 'a>(
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
    pick_list(entries, Some(current), move |entry: Labelled<T>| on_select(entry.value))
        .width(CONTROL_WIDTH)
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
