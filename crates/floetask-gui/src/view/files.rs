//! The collapsible file drawer on the left. Each todo file is listed with
//! its archive (done) file directly underneath.

use iced::widget::{Column, button, column, container, row, rule, scrollable, text};
use iced::{Alignment, Element, Fill};

use super::icons::{Icon, icon};
use super::popover::popover;
use super::widgets::{caption, icon_button};
use crate::app::{Floetask, Message};
use crate::i18n::tr;
use crate::theme::{self, Colors};

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
    .on_press_maybe(has_file.then_some(Message::NewTodo));

    let mut files = Column::new().spacing(2);
    for index in 0..app.state.files.len() {
        if is_archive_of_another(app, index) {
            continue;
        }
        files = files.push(file_entry(app, index, colors));
        if let Some(archive) = archive_entry(app, index, colors) {
            files = files.push(archive);
        }
    }

    let footer = column![
        wide_button(Icon::Folder, tr("open_file"), Message::OpenFileDialog, colors),
        wide_button(Icon::FilePlus, tr("create_file"), Message::CreateFileDialog, colors),
    ]
    .spacing(2);

    container(
        column![
            new_todo,
            caption(tr("files"), colors),
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

fn file_entry(app: &Floetask, index: usize, colors: Colors) -> Element<'_, Message> {
    let entry = &app.state.files[index];
    let active = index == app.state.active_file;
    let open = app.file_menu == Some(index);
    let folder = entry
        .path
        .parent()
        .and_then(|parent| parent.file_name())
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_default();
    let label = button(
        row![
            icon(Icon::File, 16.0, if active { colors.primary } else { colors.muted }),
            column![
                text(entry.file_name()).size(14),
                text(folder).size(11).color(colors.muted)
            ]
            .spacing(1),
        ]
        .spacing(10)
        .align_y(Alignment::Center),
    )
    .width(Fill)
    .padding([6, 8])
    .style(theme::ghost(colors, active))
    .on_press(Message::SelectFile(index));
    let menu = popover(
        icon_button(
            Icon::DotsVertical,
            tr("file_actions"),
            Some(Message::FileMenu(if open { None } else { Some(index) })),
            colors,
            open,
        ),
        open.then(|| file_menu(app, index, colors)),
        Message::FileMenu(None),
    );
    row![label, menu].spacing(2).align_y(Alignment::Center).into()
}

/// The ⋮ menu of a file: archive file settings, reveal, close.
fn file_menu(app: &Floetask, index: usize, colors: Colors) -> Element<'_, Message> {
    let entry = &app.state.files[index];
    let has_done = entry.done_path.is_some();
    let can_archive = index == app.state.active_file && has_done && app.listing.counts.completed > 0;
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
                Some(Message::ChangeDoneFile(index)),
                false
            ),
            item(
                Icon::File,
                "open_archive_file",
                has_done.then_some(Message::OpenDoneFile(index)),
                false
            ),
            item(
                Icon::Archive,
                "archive",
                can_archive.then_some(Message::ArchiveCompleted),
                false
            ),
            item(
                Icon::Folder,
                "reveal_todo_file",
                Some(Message::RevealTodoFile(index)),
                false
            ),
            rule::horizontal(1),
            item(Icon::Close, "close_file", Some(Message::AskRemoveFile(index)), true),
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
    let active = registered == Some(app.state.active_file);
    let name = done
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    let open = match registered {
        Some(position) => Message::SelectFile(position),
        None => Message::OpenDoneFile(index),
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

/// Archive files that are open are listed under their todo file, not on
/// their own.
fn is_archive_of_another(app: &Floetask, index: usize) -> bool {
    let path = &app.state.files[index].path;
    app.state
        .files
        .iter()
        .enumerate()
        .any(|(other, entry)| other != index && entry.done_path.as_ref() == Some(path))
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
