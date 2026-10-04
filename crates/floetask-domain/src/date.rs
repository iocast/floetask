//! Calendar dates as used by todo.txt (`YYYY-MM-DD`).

use chrono::{Days, Months, NaiveDate};

pub use chrono::{Datelike, Weekday};

/// A calendar date without time zone.
pub type Date = NaiveDate;

/// First day of the week, used by natural-language and human-friendly dates.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum WeekStart {
    #[default]
    Monday,
    Saturday,
    Sunday,
}

impl WeekStart {
    pub fn weekday(self) -> Weekday {
        match self {
            WeekStart::Monday => Weekday::Mon,
            WeekStart::Saturday => Weekday::Sat,
            WeekStart::Sunday => Weekday::Sun,
        }
    }
}

/// Parses a strict ISO date (`2024-03-09`).
pub fn parse_iso(text: &str) -> Option<Date> {
    let bytes = text.as_bytes();
    let well_formed = bytes.len() == 10
        && bytes[4] == b'-'
        && bytes[7] == b'-'
        && bytes
            .iter()
            .enumerate()
            .all(|(i, b)| i == 4 || i == 7 || b.is_ascii_digit());
    if !well_formed {
        return None;
    }
    NaiveDate::parse_from_str(text, "%Y-%m-%d").ok()
}

/// Formats a date as ISO (`2024-03-09`).
pub fn format_iso(date: Date) -> String {
    date.format("%Y-%m-%d").to_string()
}

pub fn add_days(date: Date, days: i64) -> Date {
    if days >= 0 {
        date.checked_add_days(Days::new(days as u64))
    } else {
        date.checked_sub_days(Days::new(days.unsigned_abs()))
    }
    .unwrap_or(date)
}

/// Adds (or subtracts) months, clamping the day to the target month's end.
pub fn add_months(date: Date, months: i64) -> Date {
    let months_abs = Months::new(months.unsigned_abs() as u32);
    if months >= 0 {
        date.checked_add_months(months_abs)
    } else {
        date.checked_sub_months(months_abs)
    }
    .unwrap_or(date)
}

/// Adds business days (Monday to Friday), skipping weekends.
pub fn add_business_days(date: Date, days: i64) -> Date {
    let step = if days >= 0 { 1 } else { -1 };
    let mut remaining = days.abs();
    let mut current = date;
    while remaining > 0 {
        current = add_days(current, step);
        if !is_weekend(current) {
            remaining -= 1;
        }
    }
    current
}

pub fn is_weekend(date: Date) -> bool {
    matches!(date.weekday(), Weekday::Sat | Weekday::Sun)
}

/// Start of the week containing `date`.
pub fn start_of_week(date: Date, week_start: WeekStart) -> Date {
    let offset = days_since(date.weekday(), week_start.weekday());
    add_days(date, -offset)
}

/// First day of the month containing `date`.
pub fn first_of_month(date: Date) -> Date {
    date.with_day(1).unwrap_or(date)
}

/// Last day of the month containing `date`.
pub fn end_of_month(date: Date) -> Date {
    add_days(add_months(first_of_month(date), 1), -1)
}

/// Number of days from `from` forward to `to` (0..=6).
pub fn days_since(to: Weekday, from: Weekday) -> i64 {
    (to.num_days_from_monday() as i64 - from.num_days_from_monday() as i64).rem_euclid(7)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(s: &str) -> Date {
        parse_iso(s).unwrap()
    }

    #[test]
    fn parses_only_strict_iso() {
        assert_eq!(parse_iso("2024-02-29"), NaiveDate::from_ymd_opt(2024, 2, 29));
        assert_eq!(parse_iso("2023-02-29"), None);
        assert_eq!(parse_iso("2024-2-9"), None);
        assert_eq!(parse_iso("2024-02-09x"), None);
    }

    #[test]
    fn month_addition_clamps() {
        assert_eq!(add_months(d("2024-01-31"), 1), d("2024-02-29"));
        assert_eq!(add_months(d("2024-03-31"), -1), d("2024-02-29"));
    }

    #[test]
    fn business_days_skip_weekends() {
        // 2024-03-08 is a Friday.
        assert_eq!(add_business_days(d("2024-03-08"), 1), d("2024-03-11"));
        assert_eq!(add_business_days(d("2024-03-11"), -1), d("2024-03-08"));
        assert_eq!(add_business_days(d("2024-03-09"), 1), d("2024-03-11"));
    }

    #[test]
    fn week_and_month_bounds() {
        // 2024-03-13 is a Wednesday.
        assert_eq!(start_of_week(d("2024-03-13"), WeekStart::Monday), d("2024-03-11"));
        assert_eq!(start_of_week(d("2024-03-13"), WeekStart::Sunday), d("2024-03-10"));
        assert_eq!(start_of_week(d("2024-03-13"), WeekStart::Saturday), d("2024-03-09"));
        assert_eq!(end_of_month(d("2024-02-10")), d("2024-02-29"));
    }
}
