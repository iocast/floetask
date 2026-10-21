//! The collapsible file drawer on the left. With two files or more, an
//! overview of all of them comes first. Each todo file is listed with its
//! archive (done) file directly underneath, sorted by the drawer's sort
//! menu or dragged into a custom order. An archive file always moves with
//! its todo file.
//!
//! iced has no drag and drop, so it is built from mouse areas like the
//! board's: pressing a file starts a drag, each file reports when the cursor
//! enters or leaves it, and releasing over another file moves the dragged
//! one there.

use iced::widget::{Column, button, column, container, mouse_area, row, rule, scrollable, space, text};
use iced::{Alignment, Color, Element, Fill, mouse};

use floetask_application::FileSort;

use crate::app::{Floetask, Message};
use crate::i18n::tr;
use crate::theme::{self, Colors};
use crate::view::icons::{Icon, icon};
use crate::view::popover::popover;
use crate::view::widgets::{caption, icon_button, with_tooltip};
use crate::{editor, files};

const WIDTH: f32 = crate::layout::FILES_DRAWER_WIDTH;

/// The left drawer: new todo, the registered files with their menus, and
/// open / create at the bottom.
pub fn drawer(app: &Floetask) -> Element<'_, Message> {
    let colors = app.colors();
    let has_file = app.active_document().is_some();

    let new_todo = button(
        row![
            icon(Icon::Plus, 16.0, on_primary(colors)),
            text(tr("new_todo")).size(14)
        ]
        .spacing(8)
        .align_y(Alignment::Center),
    )
    .width(Fill)
    .padding([9, 12])
    .style(theme::primary(colors))
    .on_press_maybe(has_file.then_some(Message::Editor(editor::Message::New)));

    let order = app.state.drawer_order();
    let drop_line = drop_line(app, &order);
    let mut files = Column::new();
    if order.len() > 1 {
        files = files.push(overview_entry(app, colors));
    }
    for (position, &index) in order.iter().enumerate() {
        files = files.push(line(drop_line == Some(position), colors));
        let mut group = column![file_entry(app, index, colors)];
        if let Some(archive) = archive_entry(app, index, colors) {
            group = group.push(archive);
        }
        files = files.push(
            mouse_area(group)
                .on_enter(Message::Files(files::Message::Hover(index, true)))
                .on_exit(Message::Files(files::Message::Hover(index, false))),
        );
    }
    files = files.push(line(drop_line == Some(order.len()), colors));
    let mut files = mouse_area(files)
        .on_release(Message::Files(files::Message::Release))
        .on_exit(Message::Files(files::Message::ListLeft));
    if app.file_drag.is_some_and(|drag| drag.moved) {
        files = files.interaction(mouse::Interaction::Grabbing);
    }

    let footer = column![
        wide_button(
            Icon::Folder,
            tr("open_file"),
            Message::Files(files::Message::OpenDialog),
            colors
        ),
        wide_button(
            Icon::FilePlus,
            tr("create_file"),
            Message::Files(files::Message::CreateDialog),
            colors
        ),
    ]
    .spacing(2);

    container(
        column![
            new_todo,
            row![
                caption(tr("files"), colors),
                space::horizontal(),
                sort_menu(app, colors)
            ]
            .align_y(Alignment::Center),
            scrollable(files)
                .height(Fill)
                .direction(theme::thin_scrollbar())
                .style(theme::slim_scroller(colors)),
            rule::horizontal(1),
            footer,
        ]
        .spacing(12),
    )
    .width(WIDTH)
    .height(Fill)
    .padding(12)
    .style(theme::panel(colors))
    .into()
}

/// The overview of every file, above the files themselves.
fn overview_entry(app: &Floetask, colors: Colors) -> Element<'_, Message> {
    let active = app.state.shows_overview();
    let content = row![
        icon(Icon::Files, 16.0, if active { colors.primary } else { colors.muted }),
        column![
            text(tr("overview")).size(14),
            text(tr("overview_hint")).size(11).color(colors.muted)
        ]
        .spacing(1),
    ]
    .spacing(10)
    .align_y(Alignment::Center);
    button(container(content).width(Fill).padding([6, 8]))
        .width(Fill)
        .padding(0)
        .style(theme::ghost(colors, active))
        .on_press(Message::Files(files::Message::SelectOverview))
        .into()
}

/// Whether the drawer marks the file at `index` as the one shown.
fn is_shown(app: &Floetask, index: usize) -> bool {
    index == app.state.active_file && !app.state.shows_overview()
}

fn file_entry(app: &Floetask, index: usize, colors: Colors) -> Element<'_, Message> {
    let entry = &app.state.files[index];
    let active = is_shown(app, index);
    let open = app.file_menu == Some(index);
    let folder = entry
        .path
        .parent()
        .and_then(|parent| parent.file_name())
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_default();
    let content = row![
        icon(Icon::File, 16.0, if active { colors.primary } else { colors.muted }),
        column![
            text(entry.file_name()).size(14),
            text(folder).size(11).color(colors.muted)
        ]
        .spacing(1),
    ]
    .spacing(10)
    .align_y(Alignment::Center);
    // The press goes to the mouse area, which starts a drag (a release on
    // the same file selects it); the button around it only draws the hover.
    let label = button(
        mouse_area(container(content).width(Fill).padding([6, 8]))
            .on_press(Message::Files(files::Message::Press(index)))
            .on_exit(Message::Files(files::Message::EntryLeft)),
    )
    .width(Fill)
    .padding(0)
    .style(theme::ghost(colors, active))
    .on_press(Message::Files(files::Message::Select(index)));
    let menu = popover(
        icon_button(
            Icon::DotsVertical,
            tr("file_actions"),
            Some(Message::Files(files::Message::Menu(if open {
                None
            } else {
                Some(index)
            }))),
            colors,
            open,
        ),
        open.then(|| file_menu(app, index, colors)),
        Message::Files(files::Message::Menu(None)),
    );
    row![label, menu].spacing(2).align_y(Alignment::Center).into()
}

/// The ⋮ menu of a file: archive file settings, reveal, close.
fn file_menu(app: &Floetask, index: usize, colors: Colors) -> Element<'_, Message> {
    let entry = &app.state.files[index];
    let has_done = entry.done_path.is_some();
    let can_archive = is_shown(app, index) && has_done && app.listing.counts.completed > 0;
    let item = |glyph: Icon, label: &'static str, message: Option<Message>, danger: bool| {
        let enabled = message.is_some();
        let tint = match (enabled, danger) {
            (false, _) => colors.border,
            (true, true) => colors.danger,
            (true, false) => colors.text,
        };
        button(
            row![icon(glyph, 15.0, tint), text(tr(label)).size(13).color(tint)]
                .spacing(10)
                .align_y(Alignment::Center),
        )
        .width(Fill)
        .padding([7, 10])
        .style(theme::ghost(colors, false))
        .on_press_maybe(message)
    };
    container(
        column![
            item(
                Icon::Archive,
                "set_archive_file",
                Some(Message::Files(files::Message::ChangeDoneFile(index))),
                false
            ),
            item(
                Icon::File,
                "open_archive_file",
                has_done.then_some(Message::Files(files::Message::OpenDoneFile(index))),
                false
            ),
            item(
                Icon::Archive,
                "archive",
                can_archive.then_some(Message::Files(files::Message::ArchiveCompleted)),
                false
            ),
            item(
                Icon::Folder,
                "reveal_todo_file",
                Some(Message::Files(files::Message::Reveal(index))),
                false
            ),
            rule::horizontal(1),
            item(
                Icon::Close,
                "close_file",
                Some(Message::Files(files::Message::AskRemove(index))),
                true
            ),
        ]
        .spacing(2),
    )
    .padding(6)
    .width(240)
    .style(theme::card(colors))
    .into()
}

/// The archive file of the todo file at `index`, indented under it.
fn archive_entry(app: &Floetask, index: usize, colors: Colors) -> Option<Element<'_, Message>> {
    let done = app.state.files[index].done_path.as_ref()?;
    let registered = app.state.files.iter().position(|entry| &entry.path == done);
    let active = registered.is_some_and(|index| is_shown(app, index));
    let name = done
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    let open = match registered {
        Some(position) => Message::Files(files::Message::Select(position)),
        None => Message::Files(files::Message::OpenDoneFile(index)),
    };
    Some(
        button(
            row![
                icon(Icon::Archive, 14.0, if active { colors.primary } else { colors.muted }),
                text(name)
                    .size(13)
                    .color(if active { colors.primary } else { colors.muted }),
            ]
            .spacing(8)
            .align_y(Alignment::Center),
        )
        .width(Fill)
        .padding(iced::padding::left(34).right(8).top(4).bottom(4))
        .style(theme::ghost(colors, active))
        .on_press(open)
        .into(),
    )
}

/// Where the dragged file would land: the gap before the file at this
/// position, or after the last one. Above the hovered file when moving up,
/// below it when moving down.
fn drop_line(app: &Floetask, order: &[usize]) -> Option<usize> {
    let drag = app.file_drag?;
    let target = app.file_hover.filter(|&target| target != drag.from)?;
    let from = order.iter().position(|&index| index == drag.from)?;
    let to = order.iter().position(|&index| index == target)?;
    Some(if to < from { to } else { to + 1 })
}

/// The gap between two files, drawn as a line where a dragged file would
/// land. It is always there, so the list does not shift during a drag.
fn line<'a>(shown: bool, colors: Colors) -> Element<'a, Message> {
    container(space())
        .width(Fill)
        .height(2)
        .style(theme::accent(if shown { colors.primary } else { Color::TRANSPARENT }))
        .into()
}

/// The sort button next to the "Files" caption and its menu.
fn sort_menu(app: &Floetask, colors: Colors) -> Element<'_, Message> {
    let open = app.file_sort_menu;
    let anchor = with_tooltip(
        button(icon(Icon::Sort, 14.0, if open { colors.primary } else { colors.muted }))
            .padding(4)
            .style(theme::ghost(colors, open))
            .on_press(Message::Files(files::Message::SortMenu(!open))),
        tr("sort_files"),
        colors,
    );
    let item = |sort: FileSort, label: &'static str, hint: Option<&'static str>| {
        let selected = app.state.file_sort == sort;
        let label = text(tr(label))
            .size(13)
            .color(if selected { colors.primary } else { colors.text });
        button(
            column![label]
                .push(hint.map(|hint| text(tr(hint)).size(11).color(colors.muted)))
                .spacing(2),
        )
        .width(Fill)
        .padding([7, 10])
        .style(theme::ghost(colors, selected))
        .on_press(Message::Files(files::Message::Sort(sort)))
    };
    let menu = container(
        column![
            item(FileSort::Custom, "file_sort_custom", Some("file_sort_custom_hint")),
            item(FileSort::Name, "file_sort_name", None),
            item(FileSort::Folder, "file_sort_folder", None),
        ]
        .spacing(2),
    )
    .padding(6)
    .width(240)
    .style(theme::card(colors));
    popover(
        anchor,
        open.then(|| menu.into()),
        Message::Files(files::Message::SortMenu(false)),
    )
    .into()
}

fn wide_button<'a>(glyph: Icon, label: &'a str, message: Message, colors: Colors) -> Element<'a, Message> {
    button(
        row![icon(glyph, 16.0, colors.muted), text(label).size(13)]
            .spacing(10)
            .align_y(Alignment::Center),
    )
    .width(Fill)
    .padding([7, 8])
    .style(theme::ghost(colors, false))
    .on_press(message)
    .into()
}

/// Text colour on a primary button.
fn on_primary(colors: Colors) -> iced::Color {
    if colors.dark {
        colors.background
    } else {
        iced::Color::WHITE
    }
}
