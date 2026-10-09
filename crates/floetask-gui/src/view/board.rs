//! The status board: one column per status, todos as cards that can be
//! dragged to another column. When the list is grouped, every group gets its
//! own board, stacked like swimlanes under the group's header.
//!
//! iced has no drag and drop, so it is built from mouse areas: pressing a
//! card starts a drag, each column reports when the cursor enters or leaves
//! it, and releasing the button anywhere on the board drops the card on the
//! column under the cursor.

use iced::widget::{
    Column, button, checkbox, column, container, mouse_area, responsive, row, scrollable, space, text, toggler,
};
use iced::{Alignment, Color, Element, Fill, Length, mouse};

use floetask_application::TodoRef;
use floetask_domain::board::Column as BoardColumn;
use floetask_domain::listing::{BoardGroup, Lane, LaneId, ListedTodo, board_grouping};

use super::icons::{Icon, icon};
use super::list::{body, collapsible, group_header, meta, status_label};
use super::widgets::{caption, with_tooltip};
use crate::app::{Floetask, Message};
use crate::i18n::{tr, trf};
use crate::theme::{self, Colors};

pub fn view(app: &Floetask) -> Element<'_, Message> {
    let colors = app.colors();
    // Grouping only applies when the list is grouped by something other
    // than status; otherwise the switch is shown disabled.
    let grouping = board_grouping(&app.state.sorting);
    let group_label = match grouping {
        Some(attribute) => trf("board_group_by", &[&tr(attribute.key()).to_lowercase()]),
        None => tr("board_group").to_owned(),
    };
    let group_switch: iced::widget::Row<'_, Message> = row![
        text(group_label).size(13),
        toggler(app.state.board_grouped && grouping.is_some())
            .on_toggle_maybe(grouping.map(|_| Message::BoardGrouped as fn(bool) -> Message))
            .size(18),
    ]
    .spacing(8)
    .align_y(Alignment::Center);
    let group_switch = with_tooltip(group_switch, tr("board_group_hint"), colors);
    let toolbar = row![
        space().width(Fill),
        group_switch,
        button(
            row![icon(Icon::Board, 14.0, colors.text), text(tr("board_columns")).size(13)]
                .spacing(6)
                .align_y(Alignment::Center)
        )
        .padding([6, 10])
        .style(theme::ghost(colors, false))
        .on_press(Message::OpenBoardColumns),
    ]
    .spacing(12)
    .align_y(Alignment::Center);
    // Columns share the width of the board area, so opening the drawer
    // narrows them. Below a readable width they keep that width and the
    // board scrolls sideways instead.
    let boards = responsive(move |size| boards(app, size.width));
    let mut board = mouse_area(column![toolbar, boards].spacing(8).height(Fill)).on_release(Message::BoardRelease);
    if app.drag.is_some() {
        board = board.interaction(mouse::Interaction::Grabbing);
    }
    board.into()
}

/// Narrowest a column gets before the board scrolls sideways.
const MIN_LANE_WIDTH: f32 = 200.0;
const LANE_SPACING: f32 = 12.0;

/// The boards for the available `width`: one row of columns, or one per
/// group stacked like swimlanes.
fn boards(app: &Floetask, width: f32) -> Element<'_, Message> {
    let colors = app.colors();
    let scrollbar = || scrollable::Scrollbar::new().width(4).scroller_width(4);
    let count = app.board.groups.first().map_or(0, |group| group.lanes.len()) as f32;
    // Padding plus room for a vertical scrollbar.
    let needed = count * MIN_LANE_WIDTH + (count - 1.0).max(0.0) * LANE_SPACING + 24.0;
    let fits = width >= needed;
    let lane_width = if fits { Fill } else { Length::Fixed(MIN_LANE_WIDTH) };

    if app.board.is_grouped() {
        // Swimlanes: the page scrolls down; columns grow with their cards.
        let groups = Column::with_children(
            app.board
                .groups
                .iter()
                .enumerate()
                .map(|(index, group)| swimlane(app, index, group, lane_width)),
        )
        .spacing(20);
        let direction = if fits {
            scrollable::Direction::Vertical(scrollbar())
        } else {
            scrollable::Direction::Both {
                vertical: scrollbar(),
                horizontal: scrollbar(),
            }
        };
        scrollable(container(groups).padding(iced::padding::bottom(10).right(10)))
            .direction(direction)
            .style(theme::slim_scroller(colors))
            .width(Fill)
            .height(Fill)
            .into()
    } else {
        // One board: columns fill the height and scroll on their own.
        let lanes = app
            .board
            .groups
            .first()
            .map(|group| lanes(app, 0, group, true, lane_width))
            .unwrap_or_else(|| space().into());
        let lanes = container(lanes).padding(iced::padding::bottom(10)).height(Fill);
        if fits {
            lanes.width(Fill).into()
        } else {
            scrollable(lanes)
                .direction(scrollable::Direction::Horizontal(scrollbar()))
                .style(theme::slim_scroller(colors))
                .width(Fill)
                .height(Fill)
                .into()
        }
    }
}

/// One group's header above its own board.
/// A click on the header collapses the group's board to just its header.
fn swimlane<'a>(app: &'a Floetask, index: usize, group: &'a BoardGroup, width: Length) -> Element<'a, Message> {
    let colors = app.colors();
    let count = group.lanes.iter().map(|lane| lane.todos.len()).sum();
    let key = group.key();
    let collapsed = app.state.collapsed_board_groups.contains(&key);
    let mut content = column![].spacing(8);
    if let Some(header) = group_header(app, group.attribute, &group.values, count) {
        content = content.push(collapsible(header, collapsed, Message::ToggleBoardGroup(key), colors));
    }
    if !collapsed {
        content = content.push(lanes(app, index, group, false, width));
    }
    content.into()
}

/// The columns of one board, each `width` wide. `fill` stretches them to
/// the window height with their own scrolling; otherwise they grow with
/// their cards.
fn lanes<'a>(
    app: &'a Floetask,
    group: usize,
    board: &'a BoardGroup,
    fill: bool,
    width: Length,
) -> Element<'a, Message> {
    let mut lanes = row(board.lanes.iter().enumerate().map(|(lane_index, l)| {
        lane(
            app,
            LaneId {
                group,
                lane: lane_index,
            },
            l,
            fill,
            width,
        )
    }))
    .spacing(LANE_SPACING);
    if fill {
        lanes = lanes.height(Fill);
    }
    lanes.into()
}

fn lane<'a>(app: &'a Floetask, id: LaneId, lane: &'a Lane, fill: bool, width: Length) -> Element<'a, Message> {
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
    let cards = Column::with_children(lane.todos.iter().map(|entry| card(app, id, entry, colors)))
        .spacing(8)
        .padding(iced::padding::right(6))
        .width(Fill);
    let cards: Element<'a, Message> = if fill {
        scrollable(cards)
            .direction(theme::thin_scrollbar())
            .style(theme::slim_scroller(colors))
            .width(Fill)
            .height(Fill)
            .into()
    } else {
        // Room below the cards, so an empty column is still a drop target.
        column![cards, space().height(28)].width(Fill).into()
    };
    let content = column![container(header).padding([4, 6]), cards].spacing(8).width(Fill);

    let target = app.drag.as_ref().is_some_and(|drag| drag.from != id) && app.board_hover == Some(id);
    let droppable = lane.column.is_some();
    let mut frame = container(content)
        .padding(10)
        .width(width)
        .style(theme::lane(colors, target && droppable))
        .clip(true);
    if fill {
        frame = frame.height(Fill);
    }
    mouse_area(frame)
        .on_enter(Message::BoardHover(id, true))
        .on_exit(Message::BoardHover(id, false))
        .into()
}

fn card<'a>(app: &'a Floetask, lane: LaneId, entry: &'a ListedTodo, colors: Colors) -> Element<'a, Message> {
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

    // The column already names the status, so its chip is left out.
    let meta = meta(app, todo, &target, None, false, colors);
    let compact = app.settings.compact;
    let content: Element<'a, Message> = if compact {
        // The attributes go under the checkbox too, using the card's full
        // width, so a card needs fewer lines.
        let line = row![accent, done, container(body(app, todo, colors)).width(Fill)]
            .spacing(8)
            .align_y(Alignment::Start);
        column![line].extend(meta).spacing(6).into()
    } else {
        let details = column![body(app, todo, colors)].extend(meta).spacing(6).width(Fill);
        row![accent, done, details].spacing(10).align_y(Alignment::Start).into()
    };
    mouse_area(
        container(content)
            .padding(if compact { [6, 10] } else { [10, 12] })
            .width(Fill)
            .style(theme::board_card(colors, dragged))
            .clip(true),
    )
    .on_press(Message::BoardPress(target, lane))
    .on_exit(Message::BoardCardLeft)
    .interaction(mouse::Interaction::Grab)
    .into()
}
