//! The note window: the note's name and save state above the Markdown editor.

use iced::widget::{button, column, container, row, space, text};
use iced::{Alignment, Element, Fill};
use iced_markdown_editor::markdown_editor;

use floetask_domain::NoteLink;

use super::icons::{Icon, icon};
use super::widgets::with_tooltip;
use crate::app::{Floetask, Message, NoteWindow};
use crate::i18n::tr;
use crate::theme;

pub fn view<'a>(app: &Floetask, window: &'a NoteWindow) -> Element<'a, Message> {
    let colors = app.colors();
    let (status, status_color) = match (&window.error, window.note.changed()) {
        (Some(error), _) => (error.clone(), colors.danger),
        (None, true) => (tr("note_unsaved").to_owned(), colors.muted),
        (None, false) => (tr("note_saved").to_owned(), colors.muted),
    };
    // A note kept elsewhere shows where it lives.
    let mut name = column![text(window.title()).size(15)].spacing(2);
    if let NoteLink::External(path) = &window.link {
        name = name.push(text(path.clone()).size(12).color(colors.muted));
    }
    let open_externally = button(icon(Icon::Link, 16.0, colors.text))
        .padding(6)
        .style(theme::ghost(colors, false))
        .on_press(Message::NoteWindowOpenExternally);
    let header = row![
        icon(Icon::Note, 16.0, colors.muted),
        name,
        space::horizontal(),
        text(status).size(12).color(status_color),
        with_tooltip(open_externally, tr("open_in_default_app"), colors),
    ]
    .spacing(10)
    .align_y(Alignment::Center);
    let editor = markdown_editor(window.note.state())
        .view()
        .map(Message::NoteWindowAction);
    column![
        container(header)
            .padding([10, 16])
            .width(Fill)
            .style(theme::bar(colors)),
        container(editor).width(Fill).height(Fill),
    ]
    .into()
}
