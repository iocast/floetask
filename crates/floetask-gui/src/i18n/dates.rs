//! Dates spelled out in the current language: calendar titles and day
//! names. todo.txt dates stay ISO; this is only how they are shown.

use floetask_domain::Date;
use floetask_domain::date::{Datelike, Weekday};

use super::{tr, trf};

/// "October 2026".
pub fn month_year(date: Date) -> String {
    capitalized(trf(
        "date_month_year",
        &[("month", month(date).into()), ("year", year(date).into())],
    ))
}

/// "Tuesday, 6 October 2026".
pub fn long_date(date: Date) -> String {
    capitalized(trf(
        "date_long",
        &[
            (
                "weekday",
                tr(&format!("weekday_{}", weekday_key(date.weekday()))).into(),
            ),
            ("day", date.day().into()),
            ("month", month(date).into()),
            ("year", year(date).into()),
        ],
    ))
}

/// A span of days, as short as it can be: "5 – 11 Oct 2026",
/// "28 Sep – 4 Oct 2026" or "28 Dec 2026 – 3 Jan 2027".
pub fn date_range(first: Date, last: Date) -> String {
    let days = [("first_day", first.day().into()), ("last_day", last.day().into())];
    if first.year() != last.year() {
        let rest = [
            ("first_month", short_month(first).into()),
            ("first_year", year(first).into()),
            ("last_month", short_month(last).into()),
            ("last_year", year(last).into()),
        ];
        trf("date_range", &[days.as_slice(), &rest].concat())
    } else if first.month() != last.month() {
        let rest = [
            ("first_month", short_month(first).into()),
            ("last_month", short_month(last).into()),
            ("year", year(last).into()),
        ];
        trf("date_range_in_year", &[days.as_slice(), &rest].concat())
    } else {
        let rest = [("month", short_month(last).into()), ("year", year(last).into())];
        trf("date_range_in_month", &[days.as_slice(), &rest].concat())
    }
}

/// "Tue", next to the day number in the week view.
pub fn weekday_abbr(date: Date) -> String {
    tr(&format!("weekday_abbr_{}", weekday_key(date.weekday()))).to_owned()
}

fn month(date: Date) -> String {
    tr(&format!("month_{}", date.month())).to_owned()
}

fn short_month(date: Date) -> String {
    tr(&format!("month_short_{}", date.month())).to_owned()
}

/// The year as text: it names a year, it does not count anything.
fn year(date: Date) -> String {
    date.year().to_string()
}

fn weekday_key(weekday: Weekday) -> &'static str {
    match weekday {
        Weekday::Mon => "mon",
        Weekday::Tue => "tue",
        Weekday::Wed => "wed",
        Weekday::Thu => "thu",
        Weekday::Fri => "fri",
        Weekday::Sat => "sat",
        Weekday::Sun => "sun",
    }
}

/// Titles start with a capital, also where month and day names are
/// lowercase (French, Spanish, Italian).
fn capitalized(text: String) -> String {
    let mut chars = text.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().chain(chars).collect(),
        None => text,
    }
}
