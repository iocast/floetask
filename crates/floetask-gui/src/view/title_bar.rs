//! The custom title bar, like Firefox and Zed: file-drawer toggle and logo
//! on the left, the search field centred, the filter drawer and settings on
//! the right next to the window buttons. Empty space drags the window; a
//! double click maximises it.

use iced::widget::{button, center, container, mouse_area, responsive, row, rule, space, stack};
use iced::{Alignment, Element, Fill};

use floetask_application::MainView;

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

    let view_button = |view: MainView, glyph: Icon, hint: &'static str| {
        icon_button(
            glyph,
            tr(hint),
            has_file.then_some(Message::ShowMainView(view)),
            colors,
            has_file && app.state.main_view == view,
        )
    };
    let left = row![
        icon_button(
            Icon::Sidebar,
            tr("toggle_files"),
            Some(Message::ToggleFilesDrawer),
            colors,
            app.shown_panels().files || app.panel_overlay == Some(crate::layout::Panel::Files),
        ),
        container(crate::logo::view(22.0)).padding([0, 6]),
    ]
    .spacing(4)
    .padding([0, 8])
    .align_y(Alignment::Center);

    let right = row![
        view_button(MainView::List, Icon::List, "show_list"),
        view_button(MainView::Board, Icon::Board, "show_board"),
        view_button(MainView::Calendar, Icon::Calendar, "show_calendar"),
        icon_button(
            Icon::Filter,
            tr("toggle_drawer"),
            has_file.then_some(Message::ToggleDrawer),
            colors,
            app.shown_panels().filters
                || app.panel_overlay == Some(crate::layout::Panel::Filters)
                || app.state.view.has_active_filters(),
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

    let middle: Element<'_, Message> = if search::field(app).is_some() {
        responsive(move |size| search_slot(app, size.width)).into()
    } else {
        space().into()
    };

    container(stack![bar, middle])
        .width(Fill)
        .height(HEIGHT)
        .style(theme::app(colors))
        .into()
}

/// Room the controls take on each side: the file-drawer toggle and logo on
/// the left; the view switch, filter, settings and window buttons on the
/// right.
const LEFT_CONTROLS: f32 = 92.0;
const RIGHT_CONTROLS: f32 = 340.0;
/// Space kept between the search field and the controls.
const SEARCH_MARGIN: f32 = 12.0;

/// The search field, centred on the whole window while it fits between the
/// controls. On narrower windows it moves into the gap between them and
/// shrinks, so it never covers a button.
fn search_slot(app: &Floetask, width: f32) -> Element<'_, Message> {
    let Some(field) = search::field(app) else {
        return space().into();
    };
    let gap_start = LEFT_CONTROLS + SEARCH_MARGIN;
    let gap = (width - gap_start - RIGHT_CONTROLS - SEARCH_MARGIN).max(0.0);
    let centred_x = (width - SEARCH_WIDTH) / 2.0;
    let (x, field_width) =
        if centred_x >= gap_start && centred_x + SEARCH_WIDTH <= width - RIGHT_CONTROLS - SEARCH_MARGIN {
            (centred_x, SEARCH_WIDTH)
        } else {
            (gap_start, gap.min(SEARCH_WIDTH))
        };
    row![space().width(x), container(field).width(field_width).center_y(HEIGHT)]
        .height(HEIGHT)
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
