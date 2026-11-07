//! The sync status next to a linked file, the share dialog and the
//! conflict dialog.

use iced::widget::text::Span;
use iced::widget::{Column, button, column, container, rich_text, row, scrollable, span, text, text_input};
use iced::{Alignment, Element, Fill, Font};

use floetask_application::SyncStatus;
use floetask_domain::merge::{Choice, Conflict, changed_words};
use floetask_domain::todo::MULTILINE_SEPARATOR;

use super::{ConflictsDialog, ShareDialog};
use crate::app::{Floetask, Message};
use crate::i18n::{self, tr, trf};
use crate::sync;
use crate::theme::{self, Colors};
use crate::view::dialogs::{buttons, raw_text};
use crate::view::icons::{Icon, icon};
use crate::view::widgets::{caption, primary_button, secondary_button, title, with_tooltip};

/// A small icon after a linked file in the drawer: in sync, syncing,
/// offline, conflicts, or the shared copy missing. Its tooltip names the
/// shared copy; a click syncs now, or opens the conflicts, or shares the
/// file again. `None` for files that are not linked.
pub fn status_icon(app: &Floetask, index: usize, colors: Colors) -> Option<Element<'_, Message>> {
    let entry = &app.state.files[index];
    let link = app.sync.link(&entry.path)?;
    let state = app.sync.status(&entry.path);
    let sync_now = Some(Message::Sync(sync::Message::SyncNow(index)));
    let (glyph, tint, label, message) = match state.and_then(|state| state.status.as_ref()) {
        _ if state.is_some_and(|state| state.running) => {
            (Icon::Sync, colors.primary, tr("sync_syncing").to_owned(), None)
        }
        None => (Icon::Sync, colors.muted, tr("sync_waiting").to_owned(), sync_now),
        Some(SyncStatus::InSync) => (Icon::Cloud, colors.muted, tr("sync_in_sync").to_owned(), sync_now),
        Some(SyncStatus::Conflict(count)) => (
            Icon::Alert,
            colors.danger,
            trf("sync_conflict", &[("count", (*count).into())]),
            Some(Message::Sync(sync::Message::ShowConflicts(entry.path.clone()))),
        ),
        Some(SyncStatus::Offline(error)) => (
            Icon::CloudOff,
            colors.muted,
            trf("sync_offline", &[("error", i18n::error_text(error).into())]),
            sync_now,
        ),
        Some(SyncStatus::SharedMissing) => (
            Icon::Alert,
            colors.danger,
            tr("sync_missing").to_owned(),
            Some(Message::Sync(sync::Message::ShareAgain(index))),
        ),
    };
    let hint = format!(
        "{label}\n{}",
        trf("sync_shared_copy", &[("location", link.shared_location().into())])
    );
    let control = button(icon(glyph, 14.0, tint))
        .padding(4)
        .style(theme::ghost(colors, false))
        .on_press_maybe(message);
    Some(with_tooltip(control, hint, colors))
}

/// Where the shared copy of a file goes: a folder, where a copy named like
/// the file with `-share` added is created, or an existing file to share
/// with. When the shared copy exists, sharing joins it.
pub fn share_dialog<'a>(app: &'a Floetask, dialog: &'a ShareDialog) -> Element<'a, Message> {
    let colors = app.colors();
    let file = dialog
        .file
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_default();
    let label = |key: &'static str| text(tr(key)).size(13).color(colors.muted);
    let location: Element<'a, Message> = match dialog.shared_path() {
        Some(path) => raw_text(path.display().to_string(), colors),
        None => text(tr("share_choose_hint")).size(13).into(),
    };
    let choose = row![
        secondary_button(
            tr("share_choose_folder"),
            Message::Sync(sync::Message::ShareChooseFolder),
            colors
        ),
        secondary_button(
            tr("share_choose_file"),
            Message::Sync(sync::Message::ShareChooseFile),
            colors
        ),
    ]
    .spacing(8);
    let mut content = column![
        title(trf("share_title", &[("file", file.into())])),
        text(tr("share_hint")).size(13).color(colors.muted),
        column![label("share_shared_copy"), location, choose].spacing(8),
    ]
    .spacing(16)
    .width(560);
    if dialog.folder.is_some() {
        let field = text_input("", &dialog.name)
            .on_input(|name| Message::Sync(sync::Message::ShareName(name)))
            .on_submit(Message::ConfirmDialog)
            .padding(10)
            .style(theme::input(colors));
        content = content.push(column![label("share_name"), field].spacing(6));
    }
    if dialog.is_same_file() {
        content = content.push(text(tr("share_same_file")).size(13).color(colors.danger));
    } else if dialog.exists {
        let name = dialog.name.trim().to_owned();
        content = content.push(
            text(trf("share_exists", &[("name", name.into())]))
                .size(13)
                .color(colors.warning),
        );
    }
    let label = if dialog.exists { tr("join") } else { tr("share") };
    let action = button(text(label).size(14))
        .padding([8, 16])
        .style(theme::primary(colors))
        .on_press_maybe(dialog.is_ready().then_some(Message::ConfirmDialog));
    content.push(buttons(action.into(), colors)).into()
}

/// Every conflict of a linked file and its notes: both versions side by
/// side, the words that differ highlighted, and the choice for each.
pub fn conflicts_dialog<'a>(app: &'a Floetask, dialog: &'a ConflictsDialog) -> Element<'a, Message> {
    let colors = app.colors();
    let file = dialog
        .file
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_default();
    let items = dialog.items.iter().enumerate().map(|(index, item)| {
        let heading = if item.item.is_empty() {
            tr("conflicts_todo_file").to_owned()
        } else {
            trf("conflicts_note", &[("name", item.item.clone().into())])
        };
        let conflicts = item.conflicts.iter().enumerate().map(|(position, conflict)| {
            let choice = dialog.choices[index][position];
            conflict_entry(conflict, choice, index, position, colors)
        });
        column![caption(&heading, colors), Column::with_children(conflicts).spacing(14)]
            .spacing(8)
            .into()
    });
    column![
        title(trf("conflicts_title", &[("file", file.into())])),
        text(tr("conflicts_hint")).size(13).color(colors.muted),
        scrollable(container(Column::with_children(items).spacing(20)).padding(iced::padding::right(14)))
            .direction(theme::thin_scrollbar())
            .style(theme::slim_scroller(colors))
            .height(Fill),
        buttons(
            primary_button(tr("conflicts_apply"), Message::ConfirmDialog, colors),
            colors
        ),
    ]
    .spacing(16)
    .width(Fill)
    .height(Fill)
    .into()
}

fn conflict_entry<'a>(
    conflict: &'a Conflict,
    choice: Choice,
    item: usize,
    position: usize,
    colors: Colors,
) -> Element<'a, Message> {
    let choose = |value: Choice, key: &'static str| {
        button(text(tr(key)).size(13))
            .padding([5, 12])
            .style(theme::ghost(colors, choice == value))
            .on_press(Message::Sync(sync::Message::Choose(item, position, value)))
    };
    column![
        row![
            side(tr("conflict_mine"), &conflict.local, &conflict.shared, colors),
            side(tr("conflict_theirs"), &conflict.shared, &conflict.local, colors),
        ]
        .spacing(12),
        row![
            choose(Choice::Mine, "conflict_choose_mine"),
            choose(Choice::Theirs, "conflict_choose_theirs"),
            choose(Choice::Both, "conflict_choose_both"),
        ]
        .spacing(6)
        .align_y(Alignment::Center),
    ]
    .spacing(8)
    .into()
}

/// One side of a conflict, each line with the words the other side does
/// not have highlighted. A side without lines removed them.
fn side<'a>(label: &str, lines: &'a [String], other: &'a [String], colors: Colors) -> Element<'a, Message> {
    let other = other.join(" ");
    let body: Element<'a, Message> = if lines.is_empty() {
        text(tr("conflict_removed"))
            .size(13)
            .color(colors.muted)
            .font(Font {
                style: iced::font::Style::Italic,
                ..Font::MONOSPACE
            })
            .into()
    } else {
        Column::with_children(lines.iter().map(|line| {
            let spans: Vec<Span<'a, Message, Font>> = changed_words(&other, line)
                .into_iter()
                .map(|(word, changed)| {
                    let word = word.replace(MULTILINE_SEPARATOR, "\n");
                    let piece = span(word).font(Font::MONOSPACE);
                    if changed {
                        piece.color(colors.text).background(colors.tint(colors.warning, 0.3))
                    } else {
                        piece
                    }
                })
                .collect();
            rich_text(spans).size(13).into()
        }))
        .spacing(4)
        .into()
    };
    container(column![caption(label, colors), body].spacing(6))
        .padding(12)
        .width(Fill)
        .style(theme::code(colors))
        .into()
}
