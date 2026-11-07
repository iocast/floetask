//! Small building blocks shared by the views.

use iced::widget::{
    button, center, column, container, mouse_area, opaque, row, space, stack, text, text_input, toggler, tooltip,
};
use iced::{Alignment, Element, Fill, alignment};

use floetask_application::TodoFileEntry;

use super::icons::{Icon, icon};
use crate::app::Message;
use crate::theme::{self, Colors};

/// A layer showing `content` centred over a dimmed backdrop, to stack on
/// top of the page. Clicking the backdrop closes the dialog.
pub fn modal<'a>(content: Element<'a, Message>, colors: Colors) -> Element<'a, Message> {
    let card = container(content).padding(24).max_width(680).style(theme::card(colors));
    over_page(center(opaque(card)).padding(24).into(), colors)
}

/// Margin a large dialog keeps from the window edge when the window is
/// small: about a thumb's width, so the app behind stays visible as a frame.
const LARGE_MODAL_MARGIN: f32 = 40.0;
/// The most a large dialog grows to, so it stays compact on a big window.
const LARGE_MODAL_WIDTH: f32 = 820.0;
const LARGE_MODAL_HEIGHT: f32 = 580.0;

/// Like [`modal`], but the card has a fixed size, shrinking with a small
/// window, for dialogs with a sidebar and scrolled pages such as Settings.
pub fn large_modal<'a>(content: Element<'a, Message>, colors: Colors) -> Element<'a, Message> {
    let card = container(content)
        .padding(24)
        .width(Fill)
        .height(Fill)
        .max_width(LARGE_MODAL_WIDTH)
        .max_height(LARGE_MODAL_HEIGHT)
        .style(theme::card(colors));
    over_page(center(opaque(card)).padding(LARGE_MODAL_MARGIN).into(), colors)
}

/// Puts a dialog's card layer over the dimmed backdrop. The title bar's
/// strip still drags the window, so a dialog never pins it in place; the
/// card sits above the strip where they overlap.
fn over_page<'a>(card: Element<'a, Message>, colors: Colors) -> Element<'a, Message> {
    let backdrop = mouse_area(
        container(space().width(Fill).height(Fill))
            .width(Fill)
            .height(Fill)
            .style(theme::backdrop(colors)),
    )
    .on_press(Message::CloseDialog);
    opaque(stack![backdrop, super::title_bar::drag_area(), card])
}

/// A square icon button with a tooltip. `active` highlights toggled tools;
/// without a message the button is disabled and its icon greyed out.
pub fn icon_button<'a>(
    glyph: Icon,
    hint: &'a str,
    message: Option<Message>,
    colors: Colors,
    active: bool,
) -> Element<'a, Message> {
    let tint = match (&message, active) {
        (None, _) => colors.muted,
        (Some(_), true) => colors.primary,
        (Some(_), false) => colors.text,
    };
    let content = button(center(icon(glyph, 18.0, tint)))
        .width(34)
        .height(34)
        .padding(0)
        .style(theme::ghost(colors, active))
        .on_press_maybe(message);
    with_tooltip(content, hint, colors)
}

/// Wraps `content` in a tooltip shown below it.
pub fn with_tooltip<'a>(
    content: impl Into<Element<'a, Message>>,
    hint: impl iced::widget::text::IntoFragment<'a>,
    colors: Colors,
) -> Element<'a, Message> {
    tooltip(
        content,
        container(text(hint).size(12))
            .padding([4, 8])
            .style(theme::tooltip(colors)),
        tooltip::Position::Bottom,
    )
    .gap(4)
    .into()
}

/// A label with a switch, for settings and view toggles.
pub fn switch<'a>(
    label: impl text::IntoFragment<'a>,
    value: bool,
    on_toggle: impl Fn(bool) -> Message + 'a,
) -> Element<'a, Message> {
    row![text(label).size(14).width(Fill), toggler(value).on_toggle(on_toggle)]
        .spacing(12)
        .align_y(alignment::Vertical::Center)
        .into()
}

pub fn primary_button<'a>(
    label: impl text::IntoFragment<'a>,
    message: Message,
    colors: Colors,
) -> Element<'a, Message> {
    button(text(label).size(14))
        .padding([8, 16])
        .style(theme::primary(colors))
        .on_press(message)
        .into()
}

pub fn secondary_button<'a>(
    label: impl text::IntoFragment<'a>,
    message: Message,
    colors: Colors,
) -> Element<'a, Message> {
    button(text(label).size(14))
        .padding([8, 16])
        .style(theme::secondary(colors))
        .on_press(message)
        .into()
}

pub fn danger_button<'a>(label: impl text::IntoFragment<'a>, message: Message, colors: Colors) -> Element<'a, Message> {
    button(text(label).size(14))
        .padding([8, 16])
        .style(theme::danger(colors))
        .on_press(message)
        .into()
}

/// Dialog title.
pub fn title<'a>(label: impl text::IntoFragment<'a>) -> Element<'a, Message> {
    text(label)
        .size(20)
        .font(iced::Font {
            weight: iced::font::Weight::Semibold,
            ..iced::Font::default()
        })
        .into()
}

/// Small uppercase label above a group of controls.
pub fn caption<'a>(label: &str, colors: Colors) -> Element<'a, Message> {
    text(label.to_uppercase())
        .size(11)
        .color(colors.muted)
        .font(iced::Font {
            weight: iced::font::Weight::Semibold,
            ..iced::Font::default()
        })
        .into()
}

/// A field for naming a new status: the user types the display name, and
/// the key written to todos (`status:in-review`) shows read-only below it,
/// since it cannot change once todos use it. Enter and the button send
/// `submit`; without it, both are disabled.
pub fn status_name_field<'a>(
    placeholder: &'a str,
    name: &'a str,
    key: &str,
    on_input: fn(String) -> Message,
    submit: Option<Message>,
    action: &'a str,
    colors: Colors,
) -> Element<'a, Message> {
    let field = row![
        text_input(placeholder, name)
            .on_input(on_input)
            .on_submit_maybe(submit.clone())
            .padding(8)
            .size(13)
            .style(theme::input(colors)),
        button(text(action).size(14))
            .padding([8, 14])
            .style(theme::secondary(colors))
            .on_press_maybe(submit),
    ]
    .spacing(8)
    .align_y(Alignment::Center);
    // Empty until something is typed, so the dialog does not jump.
    let hint = if key.is_empty() {
        String::new()
    } else {
        format!("status:{key}")
    };
    column![
        field,
        container(text(hint).size(12).color(colors.muted).font(iced::Font::MONOSPACE)).padding([0, 9]),
    ]
    .spacing(4)
    .into()
}

/// A file in a file pick list: its label and its path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct FileChoice(pub String, pub std::path::PathBuf);

impl std::fmt::Display for FileChoice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Pick list choices for these files, in the given order. A name that
/// several files share gets its folder added, so each choice is clear.
pub(crate) fn file_choices(files: &[&TodoFileEntry]) -> Vec<FileChoice> {
    let names: Vec<String> = files.iter().map(|entry| entry.file_name()).collect();
    files
        .iter()
        .zip(&names)
        .map(|(entry, name)| {
            let shared = names.iter().filter(|other| *other == name).count() > 1;
            let label = match entry.path.parent() {
                Some(folder) if shared => format!("{name} ({})", folder.display()),
                _ => name.clone(),
            };
            FileChoice(label, entry.path.clone())
        })
        .collect()
}
