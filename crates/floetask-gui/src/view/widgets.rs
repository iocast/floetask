//! Small building blocks shared by the views.

use iced::widget::{button, center, container, mouse_area, opaque, row, text, toggler, tooltip};
use iced::{Element, Fill, alignment};

use super::icons::{Icon, icon};
use crate::app::Message;
use crate::theme::{self, Colors};

/// A layer showing `content` centred over a dimmed backdrop, to stack on
/// top of the page. Clicking the backdrop closes the dialog.
pub fn modal<'a>(content: Element<'a, Message>, colors: Colors) -> Element<'a, Message> {
    let card = container(content).padding(24).max_width(680).style(theme::card(colors));
    opaque(mouse_area(center(opaque(card)).padding(24).style(theme::backdrop(colors))).on_press(Message::CloseDialog))
}

/// Margin around a full-window dialog: about a thumb's width, so the app
/// behind stays visible as a frame.
const FULL_MODAL_MARGIN: f32 = 40.0;

/// Like [`modal`], but the card fills the window minus a thumb-width
/// margin, for dialogs with a lot of content such as Settings.
pub fn full_modal<'a>(content: Element<'a, Message>, colors: Colors) -> Element<'a, Message> {
    let card = container(content)
        .padding(24)
        .width(Fill)
        .height(Fill)
        .style(theme::card(colors));
    opaque(
        mouse_area(
            container(opaque(card))
                .padding(FULL_MODAL_MARGIN)
                .width(Fill)
                .height(Fill)
                .style(theme::backdrop(colors)),
        )
        .on_press(Message::CloseDialog),
    )
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
