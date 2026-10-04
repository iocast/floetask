//! The custom title bar, like Firefox and Zed: file-drawer toggle and logo
//! on the left, the search field centred, the filter drawer and settings on
//! the right next to the window buttons. Empty space drags the window; a
//! double click maximises it.

use iced::widget::{button, center, center_x, container, mouse_area, row, rule, space, stack};
use iced::{Alignment, Element, Fill};

use super::icons::{Icon, icon};
use super::search;
use super::widgets::icon_button;
use crate::app::{Floetask, Message};
use crate::i18n::tr;
use crate::theme::{self, Colors};

pub const HEIGHT: f32 = 48.0;
const SEARCH_WIDTH: f32 = 520.0;

pub fn view(app: &Floetask) -> Element<'_, Message> {
    let colors = app.colors();
    let has_file = app.active_document().is_some();

    let left = row![
        icon_button(
            Icon::Sidebar,
            tr("toggle_files"),
            Some(Message::ToggleFilesDrawer),
            colors,
            app.state.files_drawer_open,
        ),
        container(crate::logo::view(22.0)).padding([0, 6]),
    ]
    .spacing(4)
    .padding([0, 8])
    .align_y(Alignment::Center);

    let right = row![
        icon_button(
            Icon::Filter,
            tr("toggle_drawer"),
            has_file.then_some(Message::ToggleDrawer),
            colors,
            app.state.drawer_open || app.state.view.has_active_filters(),
        ),
        icon_button(
            Icon::Settings,
            tr("settings"),
            Some(Message::OpenSettings),
            colors,
            false
        ),
        rule::vertical(1).style(move |_| rule::Style {
            color: colors.border,
            radius: 0.0.into(),
            fill_mode: rule::FillMode::Padded(12),
            snap: true,
        }),
        window_controls(colors),
    ]
    .spacing(4)
    .align_y(Alignment::Center);

    // The drag area fills everything that is not a control.
    let drag = mouse_area(space().width(Fill).height(HEIGHT))
        .on_press(Message::WindowDrag)
        .on_double_click(Message::WindowToggleMaximize);
    let bar = row![left, drag, right].height(HEIGHT).align_y(Alignment::Center);

    // The search field is centred on the whole window, not on the gap.
    let middle: Element<'_, Message> = match search::field(app) {
        Some(field) => center_x(container(field).max_width(SEARCH_WIDTH).center_y(HEIGHT)).into(),
        None => space().into(),
    };

    container(stack![bar, middle])
        .width(Fill)
        .height(HEIGHT)
        .style(theme::app(colors))
        .into()
}

fn window_controls(colors: Colors) -> Element<'static, Message> {
    let control = |glyph: Icon, message: Message, close: bool| {
        button(center(icon(glyph, 16.0, colors.text)))
            .width(46)
            .height(HEIGHT)
            .padding(0)
            .style(theme::window_control(colors, close))
            .on_press(message)
    };
    row![
        control(Icon::Minimize, Message::WindowMinimize, false),
        control(Icon::Maximize, Message::WindowToggleMaximize, false),
        control(Icon::Close, Message::Quit, true),
    ]
    .into()
}
