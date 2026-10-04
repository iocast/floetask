//! The grouped todo list.

use iced::widget::{
    Column, Row, button, checkbox, column, container, markdown, mouse_area, rich_text, row, scrollable, span, text,
    tooltip,
};
use iced::{Alignment, Color, Element, Fill, Font};

use floetask_application::TodoRef;
use floetask_domain::date::format_iso;
use floetask_domain::human_date::display_bucket;
use floetask_domain::listing::{Attribute, Group, ListedTodo};
use floetask_domain::{Date, Todo};

use super::LIST_ID;
use super::widgets::link_button;
use crate::app::{DateKey, Floetask, Message};
use crate::i18n::{tr, trf};
use crate::theme::{self, Colors};

pub fn view(app: &Floetask) -> Element<'_, Message> {
    let mut index = 0;
    let mut groups = Column::new()
        .spacing(if app.settings.compact { 6 } else { 12 })
        .padding([4, 16]);
    for group in &app.listing.groups {
        let mut rows = Column::new().spacing(if app.settings.compact { 0 } else { 2 });
        if let Some(header) = group_header(app, group) {
            rows = rows.push(header);
        }
        for entry in &group.todos {
            rows = rows.push(todo_row(app, entry, index, group.attribute));
            index += 1;
        }
        groups = groups.push(rows);
    }
    scrollable(groups).id(LIST_ID).height(Fill).width(Fill).into()
}

/// Group title. Project and context names can be right-clicked to rename or
/// remove them across the file.
fn group_header<'a>(app: &'a Floetask, group: &'a Group) -> Option<Element<'a, Message>> {
    let attribute = group.attribute?;
    let colors = app.colors();
    let label: Element<'a, Message> = if group.values.is_empty() {
        text(trf("no_value", &[&tr(attribute.key()).to_lowercase()]))
            .size(13)
            .into()
    } else {
        row(group.values.iter().map(|value| {
            let shown = text(attribute_label(attribute, value)).size(13).font(Font {
                weight: iced::font::Weight::Bold,
                ..Font::default()
            });
            if attribute.is_renamable() {
                mouse_area(shown)
                    .on_right_press(Message::AskRename(attribute, value.clone()))
                    .into()
            } else {
                shown.into()
            }
        }))
        .spacing(8)
        .into()
    };
    Some(
        container(label)
            .padding([6, 4])
            .style(theme::group_header(colors))
            .into(),
    )
}

fn todo_row<'a>(
    app: &'a Floetask,
    entry: &'a ListedTodo,
    index: usize,
    grouped_by: Option<Attribute>,
) -> Element<'a, Message> {
    let colors = app.colors();
    let todo = &entry.todo;
    let target = TodoRef::new(entry.line, todo);
    let compact = app.settings.compact;

    let mut line = Row::new().spacing(8).align_y(Alignment::Center);
    let toggle_target = target.clone();
    line = line.push(checkbox(todo.is_complete()).on_toggle(move |_| Message::ToggleComplete(toggle_target.clone())));

    if let Some(priority) = todo.priority()
        && grouped_by != Some(Attribute::Priority)
    {
        line = line.push(
            container(text(priority.to_string()).size(12))
                .padding([1, 6])
                .style(theme::badge(colors.priority(priority))),
        );
    }

    line = line.push(container(body(app, todo, colors)).width(Fill));
    line = line.push(chips(app, todo, &target, colors));
    line = line.push(date_marks(todo, colors));

    let selected = app.selected == Some(index);
    let mut content = column![line].spacing(4);
    if app.row_menu == Some(entry.line) {
        content = content.push(row_menu(todo, &target));
    }
    let row_box = container(content)
        .padding(if compact { [2, 6] } else { [6, 8] })
        .width(Fill)
        .style(theme::row(colors, selected));
    mouse_area(row_box)
        .on_press(Message::OpenTodo(target))
        .on_right_press(Message::RowMenu(Some(entry.line)))
        .into()
}

/// The description: Markdown when it uses Markdown syntax, plain text
/// otherwise. Completed todos are struck through and muted.
fn body<'a>(app: &'a Floetask, todo: &'a Todo, colors: Colors) -> Element<'a, Message> {
    let size = if app.settings.compact { 14 } else { 15 };
    if todo.is_complete() {
        return rich_text([span(todo.body()).strikethrough(true).color(colors.muted)])
            .size(size)
            .on_link_click(Message::OpenLink)
            .into();
    }
    let main: Element<'a, Message> = match app.markdown.get(todo.body()) {
        Some(content) => {
            let style = markdown::Style::from_palette(app.theme().palette());
            markdown::view(content.items(), markdown::Settings::with_text_size(size, style)).map(Message::OpenLink)
        }
        None => text(todo.body()).size(size).into(),
    };
    let links = urls(todo.body());
    if links.is_empty() {
        return main;
    }
    // Bare links get an explicit open button, so a click on the row never
    // opens a link by accident.
    let mut buttons = Row::new().spacing(4);
    for url in links {
        buttons = buttons.push(
            button(text(format!("↗ {}", short_url(&url))).size(12))
                .style(theme::chip(colors.primary, colors, false))
                .padding([1, 6])
                .on_press(Message::OpenLink(url)),
        );
    }
    column![main, buttons].spacing(2).into()
}

/// Attribute chips. Clicking one filters the list by it.
fn chips<'a>(app: &'a Floetask, todo: &'a Todo, target: &TodoRef, colors: Colors) -> Element<'a, Message> {
    let mut chips = Row::new().spacing(4).align_y(Alignment::Center);
    let chip = |attribute: Attribute, value: String, label: String, color: Color| {
        let active = app.state.view.filter_state(attribute, &value) == Some(false);
        button(text(label).size(12))
            .padding([1, 8])
            .style(theme::chip(color, colors, active))
            .on_press(Message::ChipFilter(attribute, value))
    };
    for project in todo.projects() {
        chips = chips.push(chip(
            Attribute::Projects,
            project.clone(),
            format!("+{project}"),
            colors.primary,
        ));
    }
    for context in todo.contexts() {
        chips = chips.push(chip(
            Attribute::Contexts,
            context.clone(),
            format!("@{context}"),
            colors.success,
        ));
    }
    for (key, attribute, date) in [
        (DateKey::Due, Attribute::Due, todo.due()),
        (DateKey::Threshold, Attribute::Threshold, todo.threshold()),
    ] {
        let Some(date) = date else { continue };
        let urgent = key == DateKey::Due && date <= app.today && !todo.is_complete();
        let color = if urgent { colors.danger } else { colors.muted };
        let marker = if urgent { "● " } else { "" };
        let label = format!(
            "{marker}{}: {}",
            tr(key.key()),
            date_label(app, date, key == DateKey::Due)
        );
        let filter_value = attribute
            .values(todo, &app.date_context())
            .into_iter()
            .next()
            .unwrap_or_default();
        chips = chips.push(
            row![
                chip(attribute, filter_value, label, color),
                button(text("▾").size(11))
                    .padding([1, 4])
                    .style(button::text)
                    .on_press(Message::OpenRowDatePicker(target.clone(), key)),
            ]
            .align_y(Alignment::Center),
        );
    }
    if let Some(rec) = todo.recurrence() {
        chips = chips.push(chip(
            Attribute::Recurrence,
            rec.to_string(),
            format!("↻ {rec}"),
            colors.muted,
        ));
    }
    if let Some(pm) = todo.pomodoros() {
        chips = chips.push(chip(
            Attribute::Pomodoro,
            pm.to_string(),
            format!("◔ {pm}"),
            colors.muted,
        ));
    }
    chips.into()
}

/// Small markers for creation and completion dates, with the date as tooltip.
fn date_marks(todo: &Todo, colors: Colors) -> Element<'_, Message> {
    let mut marks = Row::new().spacing(4);
    let mark = |symbol: &'static str, key: &str, date: Date| {
        tooltip(
            text(symbol).size(12).color(colors.muted),
            container(text(trf(key, &[&format_iso(date)])).size(12))
                .padding(6)
                .style(container::rounded_box),
            tooltip::Position::Left,
        )
    };
    if let Some(created) = todo.created() {
        marks = marks.push(mark("◷", "created_on", created));
    }
    if let Some(completed) = todo.completed() {
        marks = marks.push(mark("✓", "completed_on", completed));
    }
    marks.into()
}

fn row_menu<'a>(todo: &'a Todo, target: &TodoRef) -> Element<'a, Message> {
    row![
        link_button(tr("edit"), Message::OpenTodo(target.clone())),
        link_button(tr("copy"), Message::CopyTodo(todo.raw().to_owned())),
        link_button(tr("archive"), Message::ArchiveOne(target.clone())),
        link_button(tr("delete"), Message::AskDelete(target.clone())),
    ]
    .spacing(6)
    .into()
}

pub(crate) fn date_label(app: &Floetask, date: Date, is_due: bool) -> String {
    if app.settings.human_friendly_dates
        && let Some(bucket) = display_bucket(date, app.today, app.settings.week_start, is_due)
    {
        return tr(bucket.key()).to_owned();
    }
    format_iso(date)
}

/// How an attribute value is shown in headers and the drawer.
pub(crate) fn attribute_label(attribute: Attribute, value: &str) -> String {
    match attribute {
        Attribute::Projects => format!("+{value}"),
        Attribute::Contexts => format!("@{value}"),
        Attribute::Priority => format!("({value})"),
        _ => tr(value).to_owned(),
    }
}

/// Whether a body is worth rendering as Markdown.
pub fn looks_like_markdown(body: &str) -> bool {
    body.contains(['*', '_', '`', '[', '#', '>', '|', '~']) || body.contains('\n')
}

/// Bare URLs in a body (any `scheme://` token).
fn urls(body: &str) -> Vec<String> {
    body.split_whitespace()
        .map(|word| {
            word.trim_start_matches(['(', '[', '<'])
                .trim_end_matches(['.', ',', ')', ']', '>'])
        })
        .filter(|word| {
            word.split_once("://").is_some_and(|(scheme, rest)| {
                !scheme.is_empty()
                    && !rest.is_empty()
                    && scheme.chars().all(|c| c.is_ascii_alphanumeric() || "+-.".contains(c))
            })
        })
        .map(str::to_owned)
        .collect()
}

fn short_url(url: &str) -> String {
    let rest = url.split_once("://").map(|(_, rest)| rest).unwrap_or(url);
    let host = rest.split('/').next().unwrap_or(rest);
    if host.is_empty() {
        url.to_owned()
    } else {
        host.to_owned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_urls_with_any_scheme() {
        assert_eq!(
            urls("see https://example.com/a, and joplin://x/1 or (file:///tmp/a)"),
            vec!["https://example.com/a", "joplin://x/1", "file:///tmp/a"]
        );
        assert!(urls("no links: here").is_empty());
        assert_eq!(short_url("https://example.com/a"), "example.com");
    }
}
