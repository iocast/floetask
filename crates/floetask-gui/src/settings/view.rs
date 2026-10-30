//! The settings dialog: a sidebar of sections on the left, the settings of
//! the chosen section on the right. Every setting has a short grey
//! description under its name.

use iced::widget::{
    Column, button, checkbox, column, container, pick_list, row, rule, scrollable, slider, space, text, toggler,
};
use iced::{Alignment, Element, Fill};

use floetask_application::ports::PublishedRelease;
use floetask_application::updates::rollback_targets;
use floetask_application::{MonthOverflow, Settings, ThemePreference, UpdateChannel};
use floetask_domain::WeekStart;
use floetask_domain::status::{StatusSet, is_valid_status, key_from_label};

use super::{SettingChange, SettingsDialog, SettingsSection, UpdateState};
use crate::app::{Floetask, Message};
use crate::i18n::{self, Language, tr, trf};
use crate::list::status_label;
use crate::settings;
use crate::theme::{self, Colors};
use crate::view::icons::Icon;
use crate::view::widgets::{icon_button, primary_button, status_name_field, title};

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
            .on_press(Message::Settings(settings::Message::Section(section)))
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
    let control = toggler(value)
        .on_toggle(move |on| Message::Settings(settings::Message::Change(change(on))))
        .into();
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
        |day| Message::Settings(settings::Message::Change(SettingChange::WeekStart(day))),
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
                enabled.then_some(Message::Settings(settings::Message::Change(change))),
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
                .on_toggle(move |on| {
                    Message::Settings(settings::Message::Change(SettingChange::StatusHidden(name.clone(), on)))
                })
                .text_size(13)
                .width(84)
                .into()
        };
        row![
            column![
                row![
                    text(status_label(app, status)).size(14),
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
    let key = key_from_label(input);
    let valid = is_valid_status(&key) && !set.names().contains(&key);
    let add = status_name_field(
        tr("status_placeholder"),
        input,
        &key,
        |text| Message::Settings(settings::Message::StatusInput(text)),
        valid.then(|| Message::Settings(settings::Message::Change(SettingChange::StatusAdd(input.to_owned())))),
        tr("add_status"),
        colors,
    );
    page(
        "section_statuses",
        vec![
            text(tr("statuses_desc")).size(12).color(colors.muted).into(),
            Column::with_children(rows).spacing(14).into(),
            add,
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
        |theme| Message::Settings(settings::Message::Change(SettingChange::Theme(theme))),
    );
    let languages = i18n::language_choices();
    let language = choice(
        &languages,
        languages.iter().copied().find(|l| *l == s.language).unwrap_or("system"),
        // Each language by its own name, so users find theirs whatever the
        // interface shows.
        |code| match Language::from_code(code) {
            Some(language) => language.native_name(),
            None => tr("language_system"),
        },
        |language| Message::Settings(settings::Message::Change(SettingChange::Language(language.to_owned()))),
    );
    let month_overflow = choice(
        &[MonthOverflow::More, MonthOverflow::Scroll],
        s.month_overflow,
        |overflow| {
            tr(match overflow {
                MonthOverflow::More => "month_overflow_more",
                MonthOverflow::Scroll => "month_overflow_scroll",
            })
        },
        |overflow| Message::Settings(settings::Message::Change(SettingChange::MonthOverflow(overflow))),
    );
    page(
        "section_appearance",
        vec![
            setting("theme", tr("theme").to_owned(), theme_choice, colors),
            setting("language", tr("language").to_owned(), language, colors),
            switch_setting("compact", s.compact, SettingChange::Compact, colors),
            setting(
                "month_overflow",
                tr("month_overflow").to_owned(),
                month_overflow,
                colors,
            ),
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
        |days| Message::Settings(settings::Message::Change(SettingChange::NotificationThreshold(days))),
    )
    .width(CONTROL_WIDTH)
    .into();
    page(
        "section_notifications",
        vec![
            switch_setting("notifications", s.notifications, SettingChange::Notifications, colors),
            setting(
                "notification_threshold",
                trf(
                    "notification_threshold",
                    &[("days", s.notification_threshold_days.into())],
                ),
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
            text(trf("about_version", &[("version", env!("CARGO_PKG_VERSION").into())]))
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
            UpdateState::Idle => row![primary_button(
                tr("check_now"),
                Message::Settings(settings::Message::CheckForUpdates),
                colors
            )]
            .into(),
            UpdateState::Checking => line(tr("update_checking").to_owned(), colors.muted).into(),
            UpdateState::UpToDate => {
                // A preview build on the stable channel waits for the next
                // release, which is newer than any of its candidates.
                let (message, color) = if running_preview() && app.settings.update_channel == UpdateChannel::Stable {
                    (tr("update_preview_on_stable"), colors.muted)
                } else {
                    (tr("update_up_to_date"), colors.success)
                };
                row![
                    line(message.to_owned(), color).width(Fill),
                    primary_button(
                        tr("check_now"),
                        Message::Settings(settings::Message::CheckForUpdates),
                        colors
                    ),
                ]
                .spacing(16)
                .align_y(Alignment::Center)
                .into()
            }
            UpdateState::Available(release) => {
                let mut details = column![line(
                    trf("update_available", &[("version", version_label(release).into())]),
                    colors.primary
                )]
                .spacing(6);
                if let Some(notes) = &release.notes {
                    details = details.push(text(notes.clone()).size(12).color(colors.muted));
                }
                row![
                    details.width(Fill),
                    primary_button(
                        tr("update_install"),
                        Message::Settings(settings::Message::InstallRelease(release.version.clone())),
                        colors
                    )
                ]
                .spacing(16)
                .align_y(Alignment::Center)
                .into()
            }
            UpdateState::Installing(version) => line(
                trf("update_installing", &[("version", version.as_str().into())]),
                colors.muted,
            )
            .into(),
            UpdateState::Failed(error) => row![
                line(trf("update_failed", &[("error", error.as_str().into())]), colors.danger).width(Fill),
                primary_button(
                    tr("check_now"),
                    Message::Settings(settings::Message::CheckForUpdates),
                    colors
                ),
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
        let channel = choice(
            &[UpdateChannel::Stable, UpdateChannel::Preview],
            app.settings.update_channel,
            |channel| {
                tr(match channel {
                    UpdateChannel::Stable => "update_channel_stable",
                    UpdateChannel::Preview => "update_channel_preview",
                })
            },
            |channel| Message::Settings(settings::Message::Change(SettingChange::UpdateChannel(channel))),
        );
        items.push(setting(
            "update_channel",
            tr("update_channel").to_owned(),
            channel,
            colors,
        ));
        items.push(rollback(app, colors));
    }
    page("section_about", items)
}

/// Installs an earlier release over this one. The list fills with the
/// releases of the channel once a check has found them.
fn rollback(app: &Floetask, colors: Colors) -> Element<'_, Message> {
    let targets: Vec<RollbackChoice> = rollback_targets(&app.published, app.settings.update_channel)
        .into_iter()
        .map(|release| RollbackChoice {
            version: release.version.clone(),
            label: version_label(release),
        })
        .collect();
    let busy = matches!(app.update, UpdateState::Checking | UpdateState::Installing(_));
    let control: Element<'_, Message> = if targets.is_empty() {
        text(tr(if app.published.is_empty() {
            "rollback_check_first"
        } else {
            "rollback_none"
        }))
        .size(13)
        .color(colors.muted)
        .into()
    } else {
        let selected = app
            .rollback_to
            .as_ref()
            .and_then(|version| targets.iter().find(|choice| &choice.version == version).cloned());
        let install = selected
            .as_ref()
            .filter(|_| !busy)
            .map(|choice| Message::Settings(settings::Message::InstallRelease(choice.version.clone())));
        row![
            pick_list(targets, selected, |choice: RollbackChoice| Message::Settings(
                settings::Message::RollbackChoice(choice.version)
            ))
            .placeholder(tr("rollback_pick"))
            .width(CONTROL_WIDTH),
            button(text(tr("rollback_install")).size(14))
                .padding([8, 16])
                .style(theme::secondary(colors))
                .on_press_maybe(install),
        ]
        .spacing(8)
        .align_y(Alignment::Center)
        .into()
    };
    setting("rollback", tr("rollback").to_owned(), control, colors)
}

/// Whether this build is a release candidate.
fn running_preview() -> bool {
    env!("CARGO_PKG_VERSION").contains('-')
}

/// A version as offered: release candidates are marked as previews.
fn version_label(release: &PublishedRelease) -> String {
    if release.prerelease {
        trf(
            "update_preview_version",
            &[("version", release.version.as_str().into())],
        )
    } else {
        release.version.clone()
    }
}

/// An earlier version in the rollback list.
#[derive(Debug, Clone, PartialEq, Eq)]
struct RollbackChoice {
    version: String,
    label: String,
}

impl std::fmt::Display for RollbackChoice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.label)
    }
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
