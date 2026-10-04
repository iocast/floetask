//! A month calendar for picking dates.

use iced::widget::{Column, Row, button, column, row, text};
use iced::{Alignment, Element, Length};

use floetask_domain::date::{self, Datelike, first_of_month};
use floetask_domain::{Date, WeekStart};

use super::icons::Icon;
use super::widgets::{icon_button, secondary_button};
use crate::app::{Calendar, Message};
use crate::i18n::tr;
use crate::theme::{self, Colors};

const CELL: f32 = 38.0;
const WEEKDAYS: [&str; 7] = ["mon", "tue", "wed", "thu", "fri", "sat", "sun"];

pub fn view(calendar: &Calendar, today: Date, week_start: WeekStart, colors: Colors) -> Element<'_, Message> {
    let month = first_of_month(calendar.month);
    let header = row![
        icon_button(
            Icon::ChevronLeft,
            tr("previous_month"),
            Some(Message::CalendarMonth(-1)),
            colors,
            false
        ),
        text(month.format("%B %Y").to_string())
            .size(15)
            .width(Length::Fill)
            .align_x(Alignment::Center),
        icon_button(
            Icon::ChevronRight,
            tr("next_month"),
            Some(Message::CalendarMonth(1)),
            colors,
            false
        ),
    ]
    .align_y(Alignment::Center)
    .width(CELL * 7.0);

    let first_weekday = week_start.weekday().num_days_from_monday() as usize;
    let names = Row::with_children((0..7).map(|offset| {
        text(tr(WEEKDAYS[(first_weekday + offset) % 7]))
            .size(11)
            .color(colors.muted)
            .width(CELL)
            .align_x(Alignment::Center)
            .into()
    }));

    // Start the grid on the first week-start day on or before the 1st.
    let start = date::start_of_week(month, week_start);
    let mut weeks = Column::new().spacing(2);
    for week in 0..6 {
        let days = Row::with_children((0..7).map(|weekday| {
            let day = date::add_days(start, week * 7 + weekday);
            let in_month = day.month() == month.month();
            let selected = Some(day) == calendar.selected;
            let color = if selected {
                colors.surface
            } else if !in_month {
                colors.muted
            } else if day == today {
                colors.primary
            } else {
                colors.text
            };
            let label = text(day.day().to_string())
                .size(13)
                .color(color)
                .width(Length::Fill)
                .align_x(Alignment::Center);
            let button = button(label)
                .width(CELL)
                .padding(6)
                .on_press(Message::CalendarPick(Some(day)));
            if selected {
                button.style(theme::primary(colors)).into()
            } else {
                button.style(theme::ghost(colors, false)).into()
            }
        }));
        weeks = weeks.push(days);
    }

    let footer = row![
        secondary_button(tr("today"), Message::CalendarPick(Some(today)), colors),
        secondary_button(tr("clear"), Message::CalendarPick(None), colors),
    ]
    .spacing(8);

    column![header, names, weeks, footer].spacing(8).into()
}
