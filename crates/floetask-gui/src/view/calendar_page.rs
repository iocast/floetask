//! The calendar view: the active file's todos on a day, week or month
//! calendar by due date (`features/todo-notes/18-calendar.md`).
//!
//! Dragging works like on the board: pressing a todo starts a drag, every
//! day reports when the cursor enters or leaves it, and releasing anywhere
//! on the calendar drops the todo on the day under the cursor.

use iced::widget::{
    Column, Row, button, checkbox, column, container, mouse_area, responsive, row, scrollable, space, text,
};
use iced::{Alignment, Color, Element, Fill, Length, mouse};

use floetask_application::TodoRef;
use floetask_domain::Date;
use floetask_domain::calendar::{CalendarMode, CalendarSpan, todos_by_day, undated};
use floetask_domain::date::{Datelike, first_of_month};
use floetask_domain::listing::ListedTodo;

use super::icons::{Icon, icon};
use super::list::{body, meta};
use super::widgets::{icon_button, with_tooltip};
use crate::app::{DateKey, Floetask, Message};
use crate::i18n::{tr, trf};
use crate::theme::{self, Colors};

/// Todos a month cell shows before "+N more".
const MONTH_CELL_TODOS: usize = 3;
const WEEKDAYS: [&str; 7] = ["mon", "tue", "wed", "thu", "fri", "sat", "sun"];

pub fn view(app: &Floetask) -> Element<'_, Message> {
    let mode = app.state.calendar_mode;
    let span = CalendarSpan::new(mode, app.calendar_anchor, app.settings.week_start);
    let days = todos_by_day(app.listing.todos(), &span);
    let body: Element<'_, Message> = match mode {
        CalendarMode::Month => month(app, &span, &days),
        CalendarMode::Week => week(app, &span, &days),
        CalendarMode::Day => day(app, app.calendar_anchor, days.get(&app.calendar_anchor)),
    };
    let body: Element<'_, Message> = if app.shown_panels().undated {
        row![container(body).width(Fill), undated_panel(app)]
            .spacing(12)
            .height(Fill)
            .into()
    } else {
        body
    };
    let mut calendar =
        mouse_area(column![header(app, &span), body].spacing(12).height(Fill)).on_release(Message::CalendarRelease);
    if app.calendar_drag.is_some() {
        calendar = calendar.interaction(mouse::Interaction::Grabbing);
    }
    calendar.into()
}

/// Previous / today / next, the period, how many todos have no date, and
/// the mode switch.
fn header<'a>(app: &'a Floetask, span: &CalendarSpan) -> Element<'a, Message> {
    let colors = app.colors();
    let mode = app.state.calendar_mode;
    let navigation = row![
        icon_button(
            Icon::ChevronLeft,
            tr("calendar_previous"),
            Some(Message::CalendarStep(-1)),
            colors,
            false
        ),
        button(text(tr("calendar_today")).size(13))
            .padding([6, 12])
            .style(theme::secondary(colors))
            .on_press(Message::CalendarToday),
        icon_button(
            Icon::ChevronRight,
            tr("calendar_next"),
            Some(Message::CalendarStep(1)),
            colors,
            false
        ),
        text(period_title(mode, app.calendar_anchor, span)).size(18),
    ]
    .spacing(8)
    .align_y(Alignment::Center);

    // Opens the panel of todos without a due date, to plan them.
    let undated = undated(app.listing.todos()).len();
    let note: Element<'a, Message> = if undated > 0 || app.calendar_undated_open {
        button(
            row![
                icon(Icon::Calendar, 14.0, colors.muted),
                text(trf("calendar_undated", &[&undated])).size(12)
            ]
            .spacing(6)
            .align_y(Alignment::Center),
        )
        .padding([6, 10])
        .style(theme::ghost(colors, app.calendar_undated_open))
        .on_press(Message::CalendarToggleUndated)
        .into()
    } else {
        space().into()
    };

    let modes = Row::with_children(CalendarMode::ALL.into_iter().map(|option| {
        let key = match option {
            CalendarMode::Day => "calendar_day",
            CalendarMode::Week => "calendar_week",
            CalendarMode::Month => "calendar_month",
        };
        button(text(tr(key)).size(13))
            .padding([6, 12])
            .style(theme::ghost(colors, option == mode))
            .on_press(Message::CalendarMode(option))
            .into()
    }))
    .spacing(2);

    row![navigation, space().width(Fill), note, modes]
        .spacing(16)
        .align_y(Alignment::Center)
        .into()
}

/// "October 2026", "5 – 11 Oct 2026" or "Tuesday, 6 October 2026".
fn period_title(mode: CalendarMode, anchor: Date, span: &CalendarSpan) -> String {
    match mode {
        CalendarMode::Month => first_of_month(anchor).format("%B %Y").to_string(),
        CalendarMode::Week if span.first.month() == span.last.month() => {
            format!("{} – {}", span.first.format("%-d"), span.last.format("%-d %b %Y"))
        }
        CalendarMode::Week if span.first.year() == span.last.year() => {
            format!("{} – {}", span.first.format("%-d %b"), span.last.format("%-d %b %Y"))
        }
        CalendarMode::Week => format!("{} – {}", span.first.format("%-d %b %Y"), span.last.format("%-d %b %Y")),
        CalendarMode::Day => anchor.format("%A, %-d %B %Y").to_string(),
    }
}

type Days<'a> = std::collections::BTreeMap<Date, Vec<&'a ListedTodo>>;

/// Whole weeks covering the month, one cell per day.
fn month<'a>(app: &'a Floetask, span: &CalendarSpan, days: &Days<'a>) -> Element<'a, Message> {
    let colors = app.colors();
    let month = app.calendar_anchor.month();
    let names = weekday_names(app, colors);
    let all: Vec<Date> = span.days().collect();
    let weeks = Column::with_children(all.chunks(7).map(|week| {
        Row::with_children(week.iter().map(|&date| {
            let todos = days.get(&date).map(Vec::as_slice).unwrap_or_default();
            month_cell(app, date, date.month() == month, todos)
        }))
        .spacing(6)
        .height(Fill)
        .into()
    }))
    .spacing(6)
    .height(Fill);
    column![names, weeks].spacing(6).height(Fill).into()
}

fn weekday_names<'a>(app: &Floetask, colors: Colors) -> Element<'a, Message> {
    let first = app.settings.week_start.weekday().num_days_from_monday() as usize;
    Row::with_children((0..7).map(|offset| {
        text(tr(WEEKDAYS[(first + offset) % 7]))
            .size(12)
            .color(colors.muted)
            .width(Fill)
            .align_x(Alignment::Center)
            .into()
    }))
    .spacing(6)
    .into()
}

fn month_cell<'a>(app: &'a Floetask, date: Date, in_month: bool, todos: &[&'a ListedTodo]) -> Element<'a, Message> {
    let colors = app.colors();
    let mut entries = Column::new().spacing(4).push(day_number(app, date, in_month, colors));
    for entry in todos.iter().take(MONTH_CELL_TODOS) {
        entries = entries.push(compact_card(app, date, entry, colors));
    }
    if todos.len() > MONTH_CELL_TODOS {
        entries = entries.push(
            button(text(trf("calendar_more", &[&(todos.len() - MONTH_CELL_TODOS)])).size(11))
                .padding([1, 6])
                .style(theme::ghost(colors, false))
                .on_press(Message::CalendarOpenDay(date)),
        );
    }
    drop_zone(
        app,
        date,
        container(entries).padding(6).width(Fill).height(Fill).clip(true),
        in_month,
    )
}

/// The day's number; today is a filled circle. A click opens the day.
fn day_number<'a>(app: &Floetask, date: Date, in_month: bool, colors: Colors) -> Element<'a, Message> {
    let today = date == app.today;
    let color = if today {
        colors.surface
    } else if in_month {
        colors.text
    } else {
        colors.muted
    };
    button(
        text(date.day().to_string())
            .size(12)
            .color(color)
            .align_x(Alignment::Center)
            .width(Fill),
    )
    .width(24)
    .padding([2, 0])
    .style(move |theme, status| {
        if today {
            theme::primary(colors)(theme, status)
        } else {
            theme::ghost(colors, false)(theme, status)
        }
    })
    .on_press(Message::CalendarOpenDay(date))
    .into()
}

const DAY_SPACING: f32 = 6.0;
/// Narrowest a week column gets: the columns share the week's minimum width.
const MIN_DAY_WIDTH: f32 = (crate::layout::MAIN_MIN_WIDTH - 6.0 * DAY_SPACING) / 7.0;

/// Seven day columns listing every todo of the day, with board-size cards.
fn week<'a>(app: &'a Floetask, span: &CalendarSpan, days: &Days<'a>) -> Element<'a, Message> {
    let span = *span;
    let days = days.clone();
    responsive(move |size| {
        // Columns shrink with the window down to the week's minimum width;
        // side panels close before that (see `layout`), so the week only
        // scrolls sideways in a window too small for it alone.
        if size.width >= crate::layout::MAIN_MIN_WIDTH {
            week_row(app, &span, &days, Length::Fill)
        } else {
            let colors = app.colors();
            scrollable(
                container(week_row(app, &span, &days, Length::Fixed(MIN_DAY_WIDTH))).padding(iced::padding::bottom(8)),
            )
            .direction(scrollable::Direction::Horizontal(
                scrollable::Scrollbar::new().width(4).scroller_width(4),
            ))
            .style(theme::slim_scroller(colors))
            .width(Fill)
            .height(Fill)
            .into()
        }
    })
    .into()
}

fn week_row<'a>(app: &'a Floetask, span: &CalendarSpan, days: &Days<'a>, width: Length) -> Element<'a, Message> {
    let colors = app.colors();
    Row::with_children(span.days().map(|date| {
        let today = date == app.today;
        let title = button(
            row![
                text(date.format("%a").to_string()).size(12).color(colors.muted),
                text(date.day().to_string())
                    .size(15)
                    .color(if today { colors.primary } else { colors.text }),
            ]
            .spacing(6)
            .align_y(Alignment::Center),
        )
        .padding([2, 6])
        .style(theme::ghost(colors, false))
        .on_press(Message::CalendarOpenDay(date));
        let todos = days.get(&date).map(Vec::as_slice).unwrap_or_default();
        let entries = Column::with_children(todos.iter().map(|entry| card(app, Some(date), entry, colors)))
            .spacing(8)
            .padding(iced::padding::right(6))
            .width(Fill);
        let content = column![
            title,
            scrollable(entries)
                .direction(theme::thin_scrollbar())
                .style(theme::slim_scroller(colors))
                .height(Fill)
        ]
        .spacing(8);
        drop_zone(app, date, container(content).padding(8).width(width).height(Fill), true)
    }))
    .spacing(DAY_SPACING)
    .height(Fill)
    .into()
}

/// One day with full todo cards.
fn day<'a>(app: &'a Floetask, date: Date, todos: Option<&Vec<&'a ListedTodo>>) -> Element<'a, Message> {
    let colors = app.colors();
    let todos = todos.map(Vec::as_slice).unwrap_or_default();
    let content: Element<'a, Message> = if todos.is_empty() {
        container(text(tr("calendar_empty_day")).size(14).color(colors.muted))
            .padding(20)
            .into()
    } else {
        scrollable(
            Column::with_children(todos.iter().map(|entry| card(app, Some(date), entry, colors)))
                .spacing(8)
                .padding(iced::padding::right(8)),
        )
        .direction(theme::thin_scrollbar())
        .style(theme::slim_scroller(colors))
        .height(Fill)
        .into()
    };
    drop_zone(app, date, container(content).padding(10).width(Fill).height(Fill), true)
}

/// A day that takes dropped todos, outlined while one is dragged over it.
fn drop_zone<'a>(
    app: &Floetask,
    date: Date,
    content: container::Container<'a, Message>,
    in_month: bool,
) -> Element<'a, Message> {
    let colors = app.colors();
    let target =
        app.calendar_drag.as_ref().is_some_and(|(_, from)| *from != Some(date)) && app.calendar_hover == Some(date);
    mouse_area(content.style(theme::calendar_day(colors, in_month, target)))
        .on_enter(Message::CalendarHover(date, true))
        .on_exit(Message::CalendarHover(date, false))
        .into()
}

/// A todo in a month cell: a smaller version of the board card on one line,
/// with the priority accent and the todo text cut at the cell's edge. The
/// whole text shows in a tooltip.
fn compact_card<'a>(app: &'a Floetask, date: Date, entry: &'a ListedTodo, colors: Colors) -> Element<'a, Message> {
    let from = Some(date);
    let todo = &entry.todo;
    let target = TodoRef::new(entry.line, todo);
    let accent = match todo.priority() {
        Some(priority) if !todo.is_complete() => colors.priority(priority),
        _ => Color::TRANSPARENT,
    };
    let label = text(todo.body().lines().next().unwrap_or_default().to_owned())
        .size(13)
        .wrapping(text::Wrapping::None)
        .color(if todo.is_complete() { colors.muted } else { colors.text });
    let content = row![
        container(space()).width(3).height(16).style(theme::accent(accent)),
        complete_box(entry, 13.0, colors),
        label
    ]
    .spacing(8)
    .align_y(Alignment::Center);
    let card = mouse_area(
        container(content)
            .padding([6, 8])
            .width(Fill)
            .clip(true)
            .style(theme::board_card(colors, is_dragged(app, entry))),
    )
    .on_press(Message::CalendarPress(target, from))
    .on_exit(Message::CalendarCardLeft)
    .interaction(mouse::Interaction::Grab);
    with_tooltip(card, todo.display_text(), colors)
}

/// A full todo card for the Week and Day views and the undated panel, the
/// size of a board card. `from` is the day it sits on, `None` in the panel.
fn card<'a>(app: &'a Floetask, from: Option<Date>, entry: &'a ListedTodo, colors: Colors) -> Element<'a, Message> {
    let todo = &entry.todo;
    let target = TodoRef::new(entry.line, todo);
    let accent = match todo.priority() {
        Some(priority) if !todo.is_complete() => colors.priority(priority),
        _ => Color::TRANSPARENT,
    };
    let mut extras: Vec<Element<'a, Message>> = meta(app, todo, &target, None, true, colors).into_iter().collect();
    if from.is_none() {
        extras.push(
            button(
                row![
                    icon(Icon::Calendar, 12.0, colors.primary),
                    text(tr("calendar_set_due")).size(12)
                ]
                .spacing(4)
                .align_y(Alignment::Center),
            )
            .padding([2, 8])
            .style(theme::chip(colors.primary, colors, false))
            .on_press(Message::OpenRowDatePicker(target.clone(), DateKey::Due))
            .into(),
        );
    }
    let accent = container(space()).width(3).height(20).style(theme::accent(accent));
    let compact = app.settings.compact;
    // Compact cards put the attributes under the checkbox, like the board.
    let content: Element<'a, Message> = if compact {
        let line = row![
            accent,
            complete_box(entry, 16.0, colors),
            container(body(app, todo, colors)).width(Fill)
        ]
        .spacing(8);
        column![line].extend(extras).spacing(6).into()
    } else {
        let details = column![body(app, todo, colors)].extend(extras).spacing(6).width(Fill);
        row![accent, complete_box(entry, 16.0, colors), details]
            .spacing(10)
            .into()
    };
    mouse_area(
        container(content)
            .padding(if compact { [6, 10] } else { [10, 12] })
            .width(Fill)
            .clip(true)
            .style(theme::board_card(colors, is_dragged(app, entry))),
    )
    .on_press(Message::CalendarPress(target, from))
    .on_exit(Message::CalendarCardLeft)
    .interaction(mouse::Interaction::Grab)
    .into()
}

/// Width of the panel listing todos without a due date.
const UNDATED_WIDTH: f32 = crate::layout::UNDATED_PANEL_WIDTH;

/// The todos without a due date, as cards to drag onto a day or to give a
/// date with the picker.
fn undated_panel(app: &Floetask) -> Element<'_, Message> {
    let colors = app.colors();
    let todos = undated(app.listing.todos());
    let header = row![
        text(tr("calendar_undated_title")).size(15).width(Fill),
        text(todos.len().to_string()).size(12).color(colors.muted),
        icon_button(
            Icon::Close,
            tr("close"),
            Some(Message::CalendarToggleUndated),
            colors,
            false
        ),
    ]
    .spacing(8)
    .align_y(Alignment::Center);
    let cards = Column::with_children(todos.into_iter().map(|entry| card(app, None, entry, colors)))
        .spacing(8)
        .padding(iced::padding::right(6));
    let content = column![
        header,
        text(tr("calendar_undated_hint")).size(12).color(colors.muted),
        scrollable(cards)
            .direction(theme::thin_scrollbar())
            .style(theme::slim_scroller(colors))
            .height(Fill),
    ]
    .spacing(10);
    mouse_area(
        container(content)
            .padding(12)
            .width(UNDATED_WIDTH)
            .height(Fill)
            .style(theme::lane(colors, false)),
    )
    .on_enter(Message::CalendarHoverUndated(true))
    .on_exit(Message::CalendarHoverUndated(false))
    .into()
}

/// The round checkbox that completes a todo, as in the list and on the board.
/// It takes the click, so ticking it does not start a drag.
fn complete_box<'a>(entry: &ListedTodo, size: f32, colors: Colors) -> Element<'a, Message> {
    let target = TodoRef::new(entry.line, &entry.todo);
    checkbox(entry.todo.is_complete())
        .on_toggle(move |_| Message::ToggleComplete(target.clone()))
        .size(size)
        .style(theme::round_checkbox(colors))
        .into()
}

fn is_dragged(app: &Floetask, entry: &ListedTodo) -> bool {
    app.calendar_drag
        .as_ref()
        .is_some_and(|(target, _)| target.line == entry.line)
}
