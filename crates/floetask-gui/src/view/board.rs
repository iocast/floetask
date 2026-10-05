//! The status board: one column per status, todos as cards that can be
//! dragged to another column.
//!
//! iced has no drag and drop, so it is built from mouse areas: pressing a
//! card starts a drag, each column reports when the cursor enters or leaves
//! it, and releasing the button anywhere on the board drops the card on the
//! column under the cursor.

use iced::widget::{Column, button, checkbox, column, container, mouse_area, row, scrollable, space, text};
use iced::{Alignment, Color, Element, Fill, mouse};

use floetask_application::TodoRef;
use floetask_domain::board::Column as BoardColumn;
use floetask_domain::listing::{Lane, ListedTodo};

use super::icons::{Icon, icon};
use super::list::{body, meta, status_label};
use super::widgets::caption;
use crate::app::{Floetask, Message};
use crate::i18n::tr;
use crate::theme::{self, Colors};

/// Width of one column.
const LANE_WIDTH: f32 = 290.0;

pub fn view(app: &Floetask) -> Element<'_, Message> {
    let colors = app.colors();
    let toolbar = row![
        space().width(Fill),
        button(
            row![icon(Icon::Board, 14.0, colors.text), text(tr("board_columns")).size(13)]
                .spacing(6)
                .align_y(Alignment::Center)
        )
        .padding([6, 10])
        .style(theme::ghost(colors, false))
        .on_press(Message::OpenBoardColumns),
    ];
    let lanes = row(app.board.lanes.iter().enumerate().map(|(index, l)| lane(app, index, l)))
        .spacing(12)
        .height(Fill);
    let lanes = scrollable(container(lanes).padding(iced::padding::bottom(10)))
        .direction(scrollable::Direction::Horizontal(
            scrollable::Scrollbar::new().width(4).scroller_width(4),
        ))
        .style(theme::slim_scroller(colors))
        .width(Fill)
        .height(Fill);
    let mut board = mouse_area(column![toolbar, lanes].spacing(8).height(Fill)).on_release(Message::BoardRelease);
    if app.drag.is_some() {
        board = board.interaction(mouse::Interaction::Grabbing);
    }
    board.into()
}

fn lane<'a>(app: &'a Floetask, index: usize, lane: &'a Lane) -> Element<'a, Message> {
    let colors = app.colors();
    let title = match &lane.column {
        Some(BoardColumn::Done) => tr("status_done").to_owned(),
        Some(BoardColumn::Status(status)) => status_label(status),
        None => tr("board_other").to_owned(),
    };
    let header = row![
        caption(&title, colors),
        text(lane.todos.len().to_string()).size(11).color(colors.muted)
    ]
    .spacing(8)
    .align_y(Alignment::Center);
    let cards = Column::with_children(lane.todos.iter().map(|entry| card(app, index, entry, colors)))
        .spacing(8)
        .padding(iced::padding::right(6));
    let content = column![
        container(header).padding([4, 6]),
        scrollable(cards)
            .direction(theme::thin_scrollbar())
            .style(theme::slim_scroller(colors))
            .height(Fill),
    ]
    .spacing(8);

    let target = app.drag.as_ref().is_some_and(|drag| drag.from != index) && app.board_hover == Some(index);
    let droppable = lane.column.is_some();
    mouse_area(
        container(content)
            .padding(10)
            .width(LANE_WIDTH)
            .height(Fill)
            .style(theme::lane(colors, target && droppable)),
    )
    .on_enter(Message::BoardHover(index, true))
    .on_exit(Message::BoardHover(index, false))
    .into()
}

fn card<'a>(app: &'a Floetask, lane: usize, entry: &'a ListedTodo, colors: Colors) -> Element<'a, Message> {
    let todo = &entry.todo;
    let target = TodoRef::new(entry.line, todo);
    let dragged = app.drag.as_ref().is_some_and(|drag| drag.target.line == entry.line);

    let accent_color = match todo.priority() {
        Some(priority) if !todo.is_complete() => colors.priority(priority),
        _ => Color::TRANSPARENT,
    };
    let accent = container(space())
        .width(3)
        .height(20)
        .style(theme::accent(accent_color));
    let toggle_target = target.clone();
    let done = checkbox(todo.is_complete())
        .on_toggle(move |_| Message::ToggleComplete(toggle_target.clone()))
        .size(16)
        .style(theme::round_checkbox(colors));

    let mut details = column![body(app, todo, colors)].spacing(6).width(Fill);
    // The column already names the status, so its chip is left out.
    if let Some(meta) = meta(app, todo, &target, None, false, colors) {
        details = details.push(meta);
    }
    let content = row![accent, done, details].spacing(10).align_y(Alignment::Start);
    mouse_area(
        container(content)
            .padding([10, 12])
            .width(Fill)
            .style(theme::board_card(colors, dragged)),
    )
    .on_press(Message::BoardPress(target, lane))
    .interaction(mouse::Interaction::Grab)
    .into()
}
