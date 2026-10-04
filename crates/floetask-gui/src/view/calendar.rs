//! A month calendar for picking dates.

use iced::widget::{Column, Row, button, column, row, text};
use iced::{Alignment, Element, Length};

use floetask_domain::date::{self, Datelike, first_of_month};
use floetask_domain::{Date, WeekStart};

use crate::app::{Calendar, Message};
use crate::i18n::tr;

const CELL: f32 = 36.0;
const WEEKDAYS: [&str; 7] = ["mon", "tue", "wed", "thu", "fri", "sat", "sun"];

pub fn view(calendar: &Calendar, today: Date, week_start: WeekStart) -> Element<'_, Message> {
    let month = first_of_month(calendar.month);
    let header = row![
        button(text("‹"))
            .style(button::text)
            .on_press(Message::CalendarMonth(-1)),
        text(month.format("%B %Y").to_string())
            .width(Length::Fill)
            .align_x(Alignment::Center),
        button(text("›"))
            .style(button::text)
            .on_press(Message::CalendarMonth(1)),
    ]
    .align_y(Alignment::Center)
    .width(CELL * 7.0);

    let first_weekday = week_start.weekday().num_days_from_monday() as usize;
    let names = Row::with_children((0..7).map(|offset| {
        text(tr(WEEKDAYS[(first_weekday + offset) % 7]))
            .size(12)
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
            let style = if Some(day) == calendar.selected {
                button::primary
            } else if day == today {
                button::secondary
            } else {
                button::text
            };
            let label = text(day.day().to_string())
                .size(13)
                .width(Length::Fill)
                .align_x(Alignment::Center);
            let label = if in_month {
                label
            } else {
                label.color(iced::Color::from_rgb(0.55, 0.55, 0.6))
            };
            button(label)
                .width(CELL)
                .padding(4)
                .style(style)
                .on_press(Message::CalendarPick(Some(day)))
                .into()
        }))
        .spacing(0);
        weeks = weeks.push(days);
    }

    let footer = row![
        button(text(tr("today")).size(13))
            .style(button::secondary)
            .on_press(Message::CalendarPick(Some(today))),
        button(text(tr("clear")).size(13))
            .style(button::secondary)
            .on_press(Message::CalendarPick(None)),
    ]
    .spacing(8);

    column![header, names, weeks, footer].spacing(6).into()
}
