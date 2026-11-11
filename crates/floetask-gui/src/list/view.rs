//! The grouped todo list. The overview groups it by file instead.
//!
//! Each todo is a card: a priority accent, a round checkbox, the text with
//! its attributes underneath, and quick actions that appear on hover.

use std::path::Path;

use iced::widget::{
    Column, Row, button, checkbox, column, container, markdown, mouse_area, pick_list, rich_text, row, rule,
    scrollable, space, span, stack, text,
};
use iced::{Alignment, Color, Element, Fill, mouse};

use floetask_application::{NoteService, NoteStatus};
use floetask_domain::board::BoardColumns;
use floetask_domain::date::format_iso;
use floetask_domain::human_date::display_bucket;
use floetask_domain::listing::{Attribute, ListedTodo};
use floetask_domain::status::{StatusSet, label_from_key};
use floetask_domain::todo::TextSegment;
use floetask_domain::{Date, NoteLink, Todo};

use crate::app::{Floetask, Message, Target};
use crate::editor::DateKey;
use crate::i18n::{self, tr, trf};
use crate::theme::{self, Colors};
use crate::view::LIST_ID;
use crate::view::dialogs::{buttons, raw_text};
use crate::view::icons::{Icon, icon};
use crate::view::popover::popover;
use crate::view::widgets::{
    danger_button, file_choices, icon_button, primary_button, secondary_button, title, with_tooltip,
};
use crate::{drawer, list};

pub fn view(app: &Floetask) -> Element<'_, Message> {
    let colors = app.colors();
    let compact = app.settings.compact;
    let mut index = 0;
    let mut groups = Column::new().spacing(if compact { 12 } else { 20 }).padding([4, 4]);
    for group in &app.listing.groups {
        let mut rows = Column::new().spacing(if compact { 4 } else { 6 });
        let key = app.list_group_key(group);
        let collapsed = app.state.collapsed_list_groups.contains(&key);
        let header = match group.file {
            Some(file) => Some(file_header(app, file, group.todos.len())),
            None => group_header(app, group.attribute, &group.values, group.todos.len()),
        };
        if let Some(header) = header {
            rows = rows.push(collapsible(
                header,
                collapsed,
                Message::List(list::Message::ToggleGroup(key)),
                app.colors(),
            ));
        }
        if collapsed {
            groups = groups.push(rows);
            continue;
        }
        for entry in &group.todos {
            rows = rows.push(todo_row(app, entry, index, group.attribute));
            index += 1;
        }
        groups = groups.push(rows);
    }
    let list = scrollable(container(groups).padding(iced::padding::right(8).bottom(16)))
        .id(LIST_ID)
        .direction(theme::thin_scrollbar())
        .style(theme::slim_scroller(colors))
        .height(Fill)
        .width(Fill);
    // A row press opens the todo on release, or starts dragging it to a
    // file in the file drawer.
    let mut list = mouse_area(list).on_release(Message::List(list::Message::Release));
    if app.list_drag.as_ref().is_some_and(|drag| drag.moved) {
        list = list.interaction(mouse::Interaction::Grabbing);
    }
    list.into()
}

/// Group title with its todo count, for the list and the grouped board.
/// Project and context names can be right-clicked to rename or remove them
/// across the file.
pub(crate) fn group_header<'a>(
    app: &'a Floetask,
    attribute: Option<Attribute>,
    values: &'a [String],
    count: usize,
) -> Option<Element<'a, Message>> {
    let attribute = attribute?;
    let colors = app.colors();
    let mut label = Row::new().spacing(8).align_y(Alignment::Center);
    if values.is_empty() {
        // Only completed todos have no status: done is not a status.
        let empty_label = if attribute == Attribute::Status {
            tr("status_done").to_owned()
        } else {
            trf("no_value", &[("attribute", tr(attribute.key()).to_lowercase().into())])
        };
        label = label.push(group_title(&empty_label, colors));
    }
    for value in values {
        let shown = group_title(&attribute_label(app, attribute, value), colors);
        label = label.push(if attribute.is_renamable() {
            mouse_area(shown)
                .on_right_press(Message::Drawer(drawer::Message::AskRename(attribute, value.clone())))
                .into()
        } else {
            shown
        });
    }
    label = label.push(text(count.to_string()).size(13).color(colors.muted));
    Some(container(label).padding([6, 6]).into())
}

/// A group header with a chevron; a click collapses or expands the group.
pub(crate) fn collapsible<'a>(
    header: Element<'a, Message>,
    collapsed: bool,
    on_toggle: Message,
    colors: Colors,
) -> Element<'a, Message> {
    let chevron = icon(
        if collapsed {
            Icon::ChevronRight
        } else {
            Icon::ChevronDown
        },
        16.0,
        colors.muted,
    );
    mouse_area(row![chevron, header].spacing(2).align_y(Alignment::Center))
        .on_press(on_toggle)
        .interaction(iced::mouse::Interaction::Pointer)
        .into()
}

/// The header of a file's group in the overview: its name, its folder and
/// how many of its todos are shown. Shared with the overview's board.
pub(crate) fn file_header<'a>(app: &'a Floetask, file: usize, count: usize) -> Element<'a, Message> {
    let colors = app.colors();
    let path = app.listed_file(file);
    let name = path
        .and_then(|path| path.file_name())
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_default();
    let folder = path
        .and_then(|path| path.parent())
        .map(|parent| parent.display().to_string())
        .unwrap_or_default();
    container(
        row![
            icon(Icon::File, 15.0, colors.primary),
            group_title(&name, colors),
            text(folder).size(12).color(colors.muted),
            text(count.to_string()).size(13).color(colors.muted),
        ]
        .spacing(8)
        .align_y(Alignment::Center),
    )
    .padding([6, 6])
    .into()
}

/// A group's name in its header: larger than a caption so groups stand out
/// in a long list.
fn group_title<'a>(label: &str, colors: Colors) -> Element<'a, Message> {
    text(label.to_owned())
        .size(15)
        .color(colors.text)
        .font(iced::Font {
            weight: iced::font::Weight::Semibold,
            ..iced::Font::default()
        })
        .into()
}

fn todo_row<'a>(
    app: &'a Floetask,
    entry: &'a ListedTodo,
    index: usize,
    grouped_by: Option<Attribute>,
) -> Element<'a, Message> {
    let colors = app.colors();
    let todo = &entry.todo;
    let target = app.target(entry);
    let compact = app.settings.compact;
    let hovered = app.hovered == Some(entry.id()) || app.row_menu == Some(entry.id());

    let accent_color = match todo.priority() {
        Some(priority) if !todo.is_complete() => colors.priority(priority),
        _ => Color::TRANSPARENT,
    };
    let accent = container(space())
        .width(3)
        .height(if compact { 18 } else { 22 })
        .style(theme::accent(accent_color));

    let toggle_target = target.clone();
    let done = checkbox(todo.is_complete())
        .on_toggle(move |_| Message::List(list::Message::ToggleComplete(toggle_target.clone())))
        .size(18)
        .style(theme::round_checkbox(colors));

    let mut details = column![body(app, todo, colors)].spacing(6).width(Fill);
    if let Some(meta) = meta(app, todo, &target, grouped_by, true, colors) {
        details = details.push(meta);
    }

    let line = row![accent, done, details].spacing(12).align_y(Alignment::Start);

    let dragged = app
        .list_drag
        .as_ref()
        .is_some_and(|drag| drag.moved && drag.target == target);
    let selected = app.selected == Some(index) || dragged;
    let card = container(line)
        .padding(if compact { [6, 12] } else { [10, 14] })
        .width(Fill)
        .style(theme::row(colors, hovered, selected));
    // The quick actions float over the row's right edge instead of taking
    // room in it. A row that changed size on hover would shift the list
    // while it scrolls under a still cursor, and the list would jitter.
    let actions: Element<'a, Message> = if hovered {
        container(container(quick_actions(todo, &target, colors)).style(theme::row_actions(colors)))
            .padding(iced::padding::right(6))
            .align_right(Fill)
            .center_y(Fill)
            .into()
    } else {
        space().into()
    };
    let mut row = mouse_area(stack![card, actions]).on_press(Message::List(list::Message::Press(target)));
    // While a todo is dragged, rows the cursor passes stay as they are:
    // a hover message for each would rebuild the whole window.
    if app.dragged_todo().is_none() {
        row = row
            .on_enter(Message::List(list::Message::Hover(Some(entry.id()))))
            .on_exit(Message::List(list::Message::Hover(None)));
    }
    card_menu(app, entry, row)
}

/// Extra space around an inline context pill.
const PILL_GAP: &str = "\u{2002}";

/// The todo text with its contexts in place. Markdown when it uses Markdown
/// syntax; otherwise plain text with each context as a green oval that
/// filters the list when clicked. Completed todos are struck through and
/// muted.
pub(crate) fn body<'a>(app: &'a Floetask, todo: &'a Todo, colors: Colors) -> Element<'a, Message> {
    let size = if app.settings.compact { 14 } else { 15 };
    if !todo.is_complete()
        && let Some(content) = app.markdown.get(&todo.display_text())
    {
        let mut style = markdown::Style::from_palette(app.theme().palette());
        style.inline_code_color = colors.text;
        style.inline_code_highlight.background = colors.hover.into();
        return markdown::view(content.items(), markdown::Settings::with_text_size(size, style)).map(Message::OpenLink);
    }
    let done = todo.is_complete();
    if !done && todo.contexts().is_empty() {
        return text(todo.body()).size(size).into();
    }
    let spans: Vec<_> = todo
        .text_segments()
        .into_iter()
        .flat_map(|segment| match segment {
            TextSegment::Text(words) => vec![
                span(words)
                    .strikethrough(done)
                    .color_maybe(done.then_some(colors.muted)),
            ],
            TextSegment::Context(name) => vec![
                // The pill's padding is drawn over the neighbouring spaces;
                // an en space on each side keeps a visible gap.
                span(PILL_GAP),
                span(name.clone())
                    .color(if done { colors.muted } else { colors.success })
                    .strikethrough(done)
                    .background(colors.tint(colors.success, 0.15))
                    .border(iced::border::rounded(10))
                    .padding([1, 8])
                    .link(Message::List(list::Message::ChipFilter(Attribute::Contexts, name))),
                span(PILL_GAP),
            ],
        })
        .collect();
    rich_text(spans).size(size).on_link_click(|message| message).into()
}

/// Attributes under the text: projects, contexts, dates, recurrence,
/// pomodoros and links, plus the status unless `show_status` is off (the
/// board column already names it). Chips filter the list when clicked.
pub(crate) fn meta<'a>(
    app: &'a Floetask,
    todo: &'a Todo,
    target: &Target,
    grouped_by: Option<Attribute>,
    show_status: bool,
    colors: Colors,
) -> Option<Element<'a, Message>> {
    let mut chips = Row::new().spacing(6).align_y(Alignment::Center);
    let mut empty = true;
    let chip = |attribute: Attribute, value: String, glyph: Option<Icon>, label: String, color: Color| {
        let active = app.state.view.filter_state(attribute, &value) == Some(false);
        let tint = if active { colors.surface } else { color };
        let mut content = Row::new().spacing(4).align_y(Alignment::Center);
        if let Some(glyph) = glyph {
            content = content.push(icon(glyph, 12.0, tint));
        }
        button(content.push(text(label).size(12)))
            .padding([2, 8])
            .style(theme::chip(color, colors, active))
            .on_press(Message::List(list::Message::ChipFilter(attribute, value)))
    };

    if let Some(priority) = todo.priority()
        && grouped_by != Some(Attribute::Priority)
    {
        chips = chips.push(chip(
            Attribute::Priority,
            priority.to_string(),
            None,
            priority.to_string(),
            colors.priority(priority),
        ));
        empty = false;
    }
    if let Some(status) = todo.status_tag()
        && !todo.is_complete()
        && show_status
    {
        let (glyph, color) = status_style(status, colors);
        chips = chips.push(chip(
            Attribute::Status,
            status.to_owned(),
            Some(glyph),
            status_label(app, status),
            color,
        ));
        empty = false;
    }
    for project in todo.projects() {
        chips = chips.push(chip(
            Attribute::Projects,
            project.clone(),
            None,
            project.clone(),
            colors.project,
        ));
        empty = false;
    }
    // Contexts have no chip here: they are highlighted inside the text.
    for (key, attribute, date) in [
        (DateKey::Due, Attribute::Due, todo.due()),
        (DateKey::Threshold, Attribute::Threshold, todo.threshold()),
    ] {
        let Some(date) = date else { continue };
        let urgent = key == DateKey::Due && date <= app.today && !todo.is_complete();
        let color = if urgent { colors.danger } else { colors.muted };
        let value = date_label(app, date, key == DateKey::Due);
        let filter_value = attribute
            .values(todo, &app.date_context())
            .into_iter()
            .next()
            .unwrap_or_default();
        let active = app.state.view.filter_state(attribute, &filter_value) == Some(false);
        let tint = if active { colors.surface } else { color };
        // Two-part pill, "due | 2026-10-04", so the date says what it is.
        let labelled = button(
            row![
                text(tr(key.label_key())).size(12).color(tint).font(iced::Font {
                    weight: iced::font::Weight::Semibold,
                    ..iced::Font::default()
                }),
                container(space())
                    .width(1)
                    .height(12)
                    .style(theme::accent(colors.tint(tint, 0.4))),
                icon(Icon::Calendar, 12.0, tint),
                text(value).size(12),
            ]
            .spacing(6)
            .align_y(Alignment::Center),
        )
        .padding([2, 8])
        .style(theme::chip(color, colors, active))
        .on_press(Message::List(list::Message::ChipFilter(attribute, filter_value)));
        chips = chips.push(
            row![
                labelled,
                button(icon(Icon::ChevronDown, 12.0, colors.muted))
                    .padding(2)
                    .style(theme::ghost(colors, false))
                    .on_press(Message::List(list::Message::OpenDatePicker(target.clone(), key))),
            ]
            .spacing(0)
            .align_y(Alignment::Center),
        );
        empty = false;
    }
    if let Some(rec) = todo.recurrence() {
        chips = chips.push(chip(
            Attribute::Recurrence,
            rec.to_string(),
            Some(Icon::Repeat),
            rec.to_string(),
            colors.muted,
        ));
        empty = false;
    }
    if let Some(pm) = todo.pomodoros() {
        chips = chips.push(chip(
            Attribute::Pomodoro,
            pm.to_string(),
            Some(Icon::Timer),
            pm.to_string(),
            colors.muted,
        ));
        empty = false;
    }
    if let Some(note) = todo.note() {
        chips = chips.push(note_button(app, note, target, colors));
        empty = false;
    }
    // Bare links get an explicit open button, so a click on the row never
    // opens a link by accident.
    for url in urls(todo.body()) {
        chips = chips.push(
            button(
                row![icon(Icon::Link, 12.0, colors.primary), text(short_url(&url)).size(12)]
                    .spacing(4)
                    .align_y(Alignment::Center),
            )
            .padding([2, 8])
            .style(theme::chip(colors.primary, colors, false))
            .on_press(Message::OpenLink(url)),
        );
        empty = false;
    }
    if let Some(completed) = todo.completed() {
        chips = chips.push(
            text(trf("completed_on", &[("date", format_iso(completed).into())]))
                .size(12)
                .color(colors.muted),
        );
        empty = false;
    } else if let Some(created) = todo.created() {
        chips = chips.push(
            text(trf("created_on", &[("date", format_iso(created).into())]))
                .size(12)
                .color(colors.muted),
        );
        empty = false;
    }
    (!empty).then(|| chips.wrap().into())
}

/// Opens the linked note. A note whose file does not exist yet shows as a
/// broken link and is created on click; an invalid name cannot be opened.
fn note_button<'a>(app: &Floetask, note: &'a str, target: &Target, colors: Colors) -> Element<'a, Message> {
    let status = app
        .note_statuses
        .get(&target.file)
        .and_then(|statuses| statuses.get(note))
        .copied()
        .unwrap_or(NoteStatus::Missing);
    // A note kept elsewhere shows its file name; the tooltip has the path.
    let (label, location) = match NoteLink::parse(note) {
        Ok(link @ NoteLink::External(_)) => (link.file_name().to_owned(), link.to_string()),
        _ => (note.to_owned(), note.to_owned()),
    };
    let (glyph, color, hint) = match status {
        NoteStatus::Present => (Icon::Note, colors.primary, tr("open_note").to_owned()),
        NoteStatus::Missing => (
            Icon::NoteBroken,
            colors.muted,
            trf("note_missing", &[("note", location.as_str().into())]),
        ),
        NoteStatus::Invalid => (
            Icon::NoteBroken,
            colors.danger,
            trf("note_invalid", &[("note", location.as_str().into())]),
        ),
    };
    let chip = button(
        row![icon(glyph, 12.0, color), text(label).size(12)]
            .spacing(4)
            .align_y(Alignment::Center),
    )
    .padding([2, 8])
    .style(theme::chip(color, colors, false))
    .on_press_maybe((status != NoteStatus::Invalid).then(|| Message::List(list::Message::OpenNote(target.clone()))));
    with_tooltip(chip, hint, colors)
}

/// Edit, copy, archive and delete, shown while the row is hovered.
fn quick_actions<'a>(todo: &'a Todo, target: &Target, colors: Colors) -> Element<'a, Message> {
    row![
        icon_button(
            Icon::Pencil,
            tr("edit"),
            Some(Message::List(list::Message::OpenTodo(target.clone()))),
            colors,
            false
        ),
        icon_button(
            Icon::Copy,
            tr("copy"),
            Some(Message::List(list::Message::Copy(todo.raw().to_owned()))),
            colors,
            false
        ),
        icon_button(
            Icon::Archive,
            tr("archive_one"),
            Some(Message::List(list::Message::ArchiveOne(target.clone()))),
            colors,
            false
        ),
        icon_button(
            Icon::Trash,
            tr("delete"),
            Some(Message::List(list::Message::AskDelete(target.clone()))),
            colors,
            false
        ),
    ]
    .spacing(0)
    .into()
}

/// Wraps a list row, board card or calendar card so a right-click opens a
/// menu at the cursor: copy, move to done file, move to another file and
/// delete.
pub(crate) fn card_menu<'a>(
    app: &Floetask,
    entry: &'a ListedTodo,
    card: impl Into<Element<'a, Message>>,
) -> Element<'a, Message> {
    let open = app.row_menu == Some(entry.id());
    let anchor = mouse_area(card).on_right_press(Message::List(list::Message::RowMenu(Some(entry.id()))));
    let menu = open.then(|| row_menu(app, entry, app.colors()));
    popover(anchor, menu, Message::List(list::Message::RowMenu(None)))
        .at_cursor()
        .into()
}

/// Wraps a calendar day or a board lane so a click on its free space selects
/// it (`select`), a double-click adds a todo there (`new`), and a right-click
/// opens a menu at the cursor offering the same. Cards inside keep their own
/// clicks and menu.
pub(crate) fn add_area<'a>(
    app: &Floetask,
    area: impl Into<Element<'a, Message>>,
    menu_open: bool,
    open_menu: Message,
    close_menu: Message,
    select: Message,
    new: Message,
) -> Element<'a, Message> {
    let colors = app.colors();
    let anchor = mouse_area(area)
        .on_press(select)
        .on_double_click(new.clone())
        .on_right_press(open_menu);
    let menu = menu_open.then(|| {
        container(
            button(
                row![
                    icon(Icon::Plus, 15.0, colors.text),
                    text(tr("new_todo")).size(13).color(colors.text)
                ]
                .spacing(10)
                .align_y(Alignment::Center),
            )
            .width(Fill)
            .padding([7, 10])
            .style(theme::ghost(colors, false))
            .on_press(new),
        )
        .padding(6)
        .width(220)
        .style(theme::card(colors))
        .into()
    });
    popover(anchor, menu, close_menu).at_cursor().into()
}

fn row_menu<'a>(app: &Floetask, entry: &'a ListedTodo, colors: Colors) -> Element<'a, Message> {
    let target = app.target(entry);
    let item = |glyph: Icon, label: &'static str, message: list::Message, danger: bool| {
        let tint = if danger { colors.danger } else { colors.text };
        button(
            row![icon(glyph, 15.0, tint), text(tr(label)).size(13).color(tint)]
                .spacing(10)
                .align_y(Alignment::Center),
        )
        .width(Fill)
        .padding([7, 10])
        .style(theme::ghost(colors, false))
        .on_press(Message::List(message))
    };
    let mut items = column![
        item(
            Icon::Copy,
            "copy",
            list::Message::Copy(entry.todo.raw().to_owned()),
            false
        ),
        item(
            Icon::Archive,
            "archive_one",
            list::Message::ArchiveOne(target.clone()),
            false
        ),
    ]
    .spacing(2);
    // A dialog picks the file, so the menu stays short however many files
    // are open.
    if !app.move_targets(&target.file).is_empty() {
        items = items.push(item(
            Icon::File,
            "move_to_file",
            list::Message::AskMoveToFile(target.clone()),
            false,
        ));
    }
    items = items
        .push(rule::horizontal(1))
        .push(item(Icon::Trash, "delete", list::Message::AskDelete(target), true));
    container(items).padding(6).width(220).style(theme::card(colors)).into()
}

/// The todo following the cursor while it is dragged: its priority accent
/// and text on one short line. Plain containers and text only, so it takes
/// no mouse input and the drop target under it still sees the cursor.
pub(crate) fn drag_ghost<'a>(app: &Floetask, target: &Target) -> Element<'a, Message> {
    let colors = app.colors();
    let todo = Todo::parse(&target.todo.raw);
    let accent_color = match todo.priority() {
        Some(priority) if !todo.is_complete() => colors.priority(priority),
        _ => colors.primary,
    };
    let accent = container(space())
        .width(3)
        .height(18)
        .style(theme::accent(accent_color));
    container(
        row![accent, text(ghost_label(&todo.display_text())).size(14)]
            .spacing(10)
            .align_y(Alignment::Center),
    )
    .padding([8, 12])
    .max_width(320)
    .style(theme::drag_ghost(colors))
    .into()
}

/// A todo's text cut to fit the drag ghost's single line.
fn ghost_label(text: &str) -> String {
    const MAX: usize = 40;
    if text.chars().count() <= MAX {
        return text.to_owned();
    }
    let cut: String = text.chars().take(MAX - 1).collect();
    format!("{}…", cut.trim_end())
}

/// Moves a todo to another file picked from a dropdown of the files in the
/// file drawer.
pub(crate) fn move_dialog<'a>(app: &'a Floetask, target: &'a Target, to: &'a Path) -> Element<'a, Message> {
    let colors = app.colors();
    let line = Todo::parse(&target.todo.raw).editable_text();
    let choices = file_choices(&app.move_targets(&target.file));
    let current = choices.iter().find(|choice| choice.1 == to).cloned();
    column![
        title(tr("move_to_file_title")),
        raw_text(line, colors),
        row![
            icon(Icon::File, 14.0, colors.muted),
            text(tr("move_to_file_label")).size(13).color(colors.muted),
            pick_list(choices, current, |choice| Message::List(
                list::Message::MoveToFileChoice(choice.1)
            ))
            .text_size(13)
            .width(Fill),
        ]
        .spacing(8)
        .align_y(Alignment::Center),
        buttons(primary_button(tr("move"), Message::ConfirmDialog, colors), colors),
    ]
    .spacing(16)
    .width(460)
    .into()
}

/// Confirms deleting a todo whose note no other todo uses and asks whether
/// the note goes too. Keeping it is the default (`Enter`).
pub(crate) fn delete_dialog<'a>(app: &'a Floetask, target: &'a Target, note: &'a NoteLink) -> Element<'a, Message> {
    let colors = app.colors();
    let line = Todo::parse(&target.todo.raw).editable_text();
    let note = match note {
        NoteLink::Folder(name) => format!("{}/{name}", NoteService::folder_name(&target.file)),
        NoteLink::External(path) => path.clone(),
    };
    let actions = row![
        secondary_button(tr("delete_todo_only"), Message::ConfirmDialog, colors),
        danger_button(
            tr("delete_todo_and_note"),
            Message::List(list::Message::DeleteWithNote),
            colors
        ),
    ]
    .spacing(8);
    column![
        title(tr("delete_confirm")),
        raw_text(line, colors),
        text(trf("delete_note_question", &[("note", note.into())])).size(14),
        buttons(actions.into(), colors),
    ]
    .spacing(16)
    .width(460)
    .into()
}

pub(crate) fn date_label(app: &Floetask, date: Date, is_due: bool) -> String {
    if app.settings.human_friendly_dates
        && let Some(bucket) = display_bucket(date, app.today, app.settings.week_start, is_due)
    {
        return bucket_label(bucket.key()).unwrap_or(bucket.key()).to_owned();
    }
    format_iso(date)
}

/// How an attribute value is shown in headers and the drawer.
pub(crate) fn attribute_label(app: &Floetask, attribute: Attribute, value: &str) -> String {
    match attribute {
        // Shown without `+` and `@`: the colour already says which it is.
        Attribute::Projects | Attribute::Contexts => value.to_owned(),
        Attribute::Priority => format!("({value})"),
        Attribute::Status => status_label(app, value),
        // Date groups are named by their bucket, such as `next week`.
        _ => bucket_label(value).unwrap_or(value).to_owned(),
    }
}

/// A human-friendly date bucket (`next week`) in the user's language.
fn bucket_label(key: &str) -> Option<&'static str> {
    i18n::tr_known(&format!("bucket_{}", key.replace(' ', "_")))
}

/// A status name for display, with the active file's board labels.
pub(crate) fn status_label(app: &Floetask, status: &str) -> String {
    let board = app
        .state
        .active_entry()
        .and_then(|entry| app.settings.boards.get(&entry.path));
    status_name(&app.settings.statuses, board, status)
}

/// A status name for display: the label from settings, else the one a
/// board gives a status only it knows, else the translation of a built-in
/// status, else a name read from the key (`in-review` is "In review").
pub(crate) fn status_name(statuses: &StatusSet, board: Option<&BoardColumns>, status: &str) -> String {
    if let Some(label) = statuses.label(status).or_else(|| board.and_then(|b| b.label(status))) {
        return label.to_owned();
    }
    match i18n::tr_known(&format!("status_{status}")) {
        Some(translated) => translated.to_owned(),
        None => label_from_key(status),
    }
}

/// Icon and colour of a status chip. `waiting` stands out, since the todo
/// is blocked on someone else.
fn status_style(status: &str, colors: Colors) -> (Icon, Color) {
    match status {
        "doing" => (Icon::Play, colors.primary),
        "waiting" => (Icon::Hourglass, colors.warning),
        _ => (Icon::Status, colors.muted),
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
