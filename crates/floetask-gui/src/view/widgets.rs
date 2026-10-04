//! Small building blocks shared by the views.

use iced::widget::{button, center, container, mouse_area, opaque, row, stack, text, toggler, tooltip};
use iced::{Element, Fill, alignment};

use crate::app::Message;
use crate::theme::{self, Colors};

/// Shows `content` centred over a dimmed `base`. Clicking the backdrop
/// closes the dialog.
pub fn modal<'a>(base: Element<'a, Message>, content: Element<'a, Message>, colors: Colors) -> Element<'a, Message> {
    let card = container(content).padding(20).max_width(640).style(theme::card(colors));
    stack![
        base,
        opaque(mouse_area(center(opaque(card)).padding(24).style(theme::backdrop)).on_press(Message::CloseDialog))
    ]
    .into()
}

/// A square icon button with a tooltip.
pub fn icon_button<'a>(symbol: &'a str, hint: &'a str, message: Option<Message>) -> Element<'a, Message> {
    let content = button(text(symbol).size(18).align_x(alignment::Horizontal::Center).width(Fill))
        .width(40)
        .height(40)
        .style(button::text)
        .on_press_maybe(message);
    tooltip(
        content,
        container(text(hint).size(13)).padding(6).style(container::rounded_box),
        tooltip::Position::Right,
    )
    .into()
}

/// A label with a switch, for settings and view toggles.
pub fn switch<'a>(label: &'a str, value: bool, on_toggle: impl Fn(bool) -> Message + 'a) -> Element<'a, Message> {
    row![text(label).width(Fill), toggler(value).on_toggle(on_toggle)]
        .spacing(12)
        .align_y(alignment::Vertical::Center)
        .into()
}

/// A small text button.
pub fn link_button<'a>(label: impl text::IntoFragment<'a>, message: Message) -> Element<'a, Message> {
    button(text(label).size(13))
        .padding([2, 6])
        .style(button::text)
        .on_press(message)
        .into()
}
