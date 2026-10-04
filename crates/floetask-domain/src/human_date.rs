//! Human-friendly date buckets (`today`, `next week`, `overdue`, ...).

use chrono::Datelike;

use crate::date::{self, Date, WeekStart};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum DateBucket {
    /// A past due date.
    Overdue,
    /// A past date that is not a due date.
    Elapsed,
    LastWeek,
    Today,
    Tomorrow,
    ThisWeek,
    NextWeek,
    ThisMonth,
    NextMonth,
}

impl DateBucket {
    /// Stable identifier, also used as the translation key.
    pub fn key(self) -> &'static str {
        match self {
            DateBucket::Overdue => "overdue",
            DateBucket::Elapsed => "elapsed",
            DateBucket::LastWeek => "last week",
            DateBucket::Today => "today",
            DateBucket::Tomorrow => "tomorrow",
            DateBucket::ThisWeek => "this week",
            DateBucket::NextWeek => "next week",
            DateBucket::ThisMonth => "this month",
            DateBucket::NextMonth => "next month",
        }
    }
}

/// Every bucket `date` falls in, most specific first. Used by the drawer,
/// where a date counts towards each of its buckets.
pub fn buckets(date: Date, today: Date, week_start: WeekStart, is_due: bool) -> Vec<DateBucket> {
    let week = date::start_of_week(today, week_start);
    let next_week = date::add_days(week, 7);
    let last_week = date::add_days(week, -7);
    let same_month = |a: Date, b: Date| a.year() == b.year() && a.month() == b.month();

    let mut result = Vec::new();
    if date < today {
        result.push(if is_due {
            DateBucket::Overdue
        } else {
            DateBucket::Elapsed
        });
        if date >= last_week && date < week {
            result.push(DateBucket::LastWeek);
        }
    }
    if date == today {
        result.push(DateBucket::Today);
    }
    if date == date::add_days(today, 1) {
        result.push(DateBucket::Tomorrow);
    }
    if date >= week && date < next_week && date >= today {
        result.push(DateBucket::ThisWeek);
    }
    if date >= next_week && date < date::add_days(next_week, 7) {
        result.push(DateBucket::NextWeek);
    }
    if same_month(date, today) && date >= today {
        result.push(DateBucket::ThisMonth);
    }
    if same_month(date, date::add_months(today, 1)) {
        result.push(DateBucket::NextMonth);
    }
    result
}

/// The single bucket shown next to a todo, or `None` to show the ISO date.
pub fn display_bucket(date: Date, today: Date, week_start: WeekStart, is_due: bool) -> Option<DateBucket> {
    const ORDER: [DateBucket; 8] = [
        DateBucket::Today,
        DateBucket::Tomorrow,
        DateBucket::Overdue,
        DateBucket::LastWeek,
        DateBucket::Elapsed,
        DateBucket::NextWeek,
        DateBucket::ThisMonth,
        DateBucket::NextMonth,
    ];
    let found = buckets(date, today, week_start, is_due);
    ORDER.into_iter().find(|bucket| found.contains(bucket))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::date::parse_iso;

    fn d(s: &str) -> Date {
        parse_iso(s).unwrap()
    }

    // 2024-03-13 is a Wednesday.
    const TODAY: &str = "2024-03-13";

    #[test]
    fn today_belongs_to_several_buckets() {
        let found = buckets(d(TODAY), d(TODAY), WeekStart::Monday, true);
        assert_eq!(
            found,
            vec![DateBucket::Today, DateBucket::ThisWeek, DateBucket::ThisMonth]
        );
    }

    #[test]
    fn display_picks_most_specific() {
        let show = |s, due| display_bucket(d(s), d(TODAY), WeekStart::Monday, due);
        assert_eq!(show("2024-03-14", true), Some(DateBucket::Tomorrow));
        assert_eq!(show("2024-03-01", true), Some(DateBucket::Overdue));
        assert_eq!(show("2024-03-05", false), Some(DateBucket::LastWeek));
        assert_eq!(show("2024-01-05", false), Some(DateBucket::Elapsed));
        assert_eq!(show("2024-03-19", true), Some(DateBucket::NextWeek));
        assert_eq!(show("2024-03-29", true), Some(DateBucket::ThisMonth));
        assert_eq!(show("2024-04-29", true), Some(DateBucket::NextMonth));
        assert_eq!(show("2024-08-29", true), None);
    }
}
