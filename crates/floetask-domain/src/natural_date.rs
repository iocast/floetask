//! Natural-language dates: `due:tomorrow`, `t:in one week`,
//! `due:next tuesday`, `due:end of february`, ...
//!
//! Covers the common English phrases people write in due and threshold
//! dates. A complete natural-language grammar is a deliberate non-goal.

use chrono::{Datelike, Month, Weekday};

use crate::date::{self, Date, WeekStart, format_iso, parse_iso};
use crate::recurrence::RecurrenceUnit;

/// Keys whose values are rewritten from relative to absolute dates.
const DATE_KEYS: [&str; 2] = ["due", "t"];

/// The longest phrase we try after a key, in words.
const MAX_PHRASE_WORDS: usize = 4;

/// Parses a relative date phrase.
pub fn parse_phrase(phrase: &str, today: Date, week_start: WeekStart) -> Option<Date> {
    let lowered = phrase.trim().to_lowercase();
    let words: Vec<&str> = lowered.split_whitespace().collect();
    fixed_phrase(&words, today, week_start)
        .or_else(|| offset_phrase(&words, today))
        .or_else(|| weekday_phrase(&words, today, week_start))
        .or_else(|| month_phrase(&words, today))
}

/// Phrases made only of keywords.
fn fixed_phrase(words: &[&str], today: Date, week_start: WeekStart) -> Option<Date> {
    let week_start_date = date::start_of_week(today, week_start);
    match words {
        ["today"] | ["now"] => Some(today),
        ["tomorrow"] => Some(date::add_days(today, 1)),
        ["yesterday"] => Some(date::add_days(today, -1)),
        ["day", "after", "tomorrow"] => Some(date::add_days(today, 2)),
        ["next", "week"] => Some(date::add_days(week_start_date, 7)),
        ["next", "month"] => Some(first_of_month(date::add_months(today, 1))),
        ["next", "year"] => Date::from_ymd_opt(today.year() + 1, 1, 1),
        ["end", "of", "week"] | ["end", "of", "the", "week"] => Some(date::add_days(week_start_date, 6)),
        ["end", "of", "month"] | ["end", "of", "the", "month"] => Some(date::end_of_month(today)),
        ["end", "of", "next", "month"] => Some(date::end_of_month(date::add_months(today, 1))),
        ["end", "of", "year"] | ["end", "of", "the", "year"] => Date::from_ymd_opt(today.year(), 12, 31),
        _ => None,
    }
}

/// `in 3 days`, `2 weeks`, `one month ago`, `3d`.
fn offset_phrase(words: &[&str], today: Date) -> Option<Date> {
    match words {
        ["in", amount, unit] => offset(today, amount, unit, 1),
        [amount, unit, "ago"] => offset(today, amount, unit, -1),
        [amount, unit] => offset(today, amount, unit, 1),
        [shorthand] => shorthand_offset(today, shorthand),
        _ => None,
    }
}

/// `friday`, `this friday`, `next tuesday`.
fn weekday_phrase(words: &[&str], today: Date, week_start: WeekStart) -> Option<Date> {
    match words {
        ["next", day] => {
            let weekday = parse_weekday(day)?;
            let next_week = date::add_days(date::start_of_week(today, week_start), 7);
            Some(date::add_days(next_week, date::days_since(weekday, week_start.weekday())))
        }
        ["this", day] | [day] => {
            let weekday = parse_weekday(day)?;
            Some(date::add_days(today, date::days_since(weekday, today.weekday())))
        }
        _ => None,
    }
}

/// `end of february`, `april 2`, `2 april`.
fn month_phrase(words: &[&str], today: Date) -> Option<Date> {
    match words {
        ["end", "of", month] => {
            let month = parse_month(month)?;
            Some(date::end_of_month(upcoming_month_start(today, month)?))
        }
        [first, second] => {
            let (month, day) = match parse_month(first) {
                Some(month) => (month, second),
                None => (parse_month(second)?, first),
            };
            month_day(today, month, day.parse().ok()?)
        }
        _ => None,
    }
}

/// Rewrites relative values of `due:` and `t:` in a todo line to ISO dates.
/// Text that does not parse as a date is left as is.
pub fn rewrite_relative_dates(text: &str, today: Date, week_start: WeekStart) -> String {
    text.split('\n')
        .map(|line| rewrite_line(line, today, week_start))
        .collect::<Vec<_>>()
        .join("\n")
}

fn rewrite_line(line: &str, today: Date, week_start: WeekStart) -> String {
    let words: Vec<&str> = line.split(' ').collect();
    let mut output: Vec<String> = Vec::with_capacity(words.len());
    let mut index = 0;
    while index < words.len() {
        let word = words[index];
        let date_key = word
            .split_once(':')
            .filter(|(key, value)| DATE_KEYS.contains(key) && !value.is_empty() && parse_iso(value).is_none());
        if let Some((key, first)) = date_key {
            let longest = (1..=MAX_PHRASE_WORDS.min(words.len() - index)).rev().find_map(|length| {
                let mut phrase = vec![first];
                phrase.extend(&words[index + 1..index + length]);
                parse_phrase(&phrase.join(" "), today, week_start).map(|date| (length, date))
            });
            if let Some((length, date)) = longest {
                output.push(format!("{key}:{}", format_iso(date)));
                index += length;
                continue;
            }
        }
        output.push(word.to_owned());
        index += 1;
    }
    output.join(" ")
}

fn offset(today: Date, amount: &str, unit: &str, sign: i64) -> Option<Date> {
    let amount = parse_amount(amount)? * sign;
    let unit = unit.strip_suffix('s').unwrap_or(unit);
    Some(match unit {
        "day" => date::add_days(today, amount),
        "business day" | "weekday" => date::add_business_days(today, amount),
        "week" => date::add_days(today, amount * 7),
        "month" => date::add_months(today, amount),
        "year" => date::add_months(today, amount * 12),
        _ => return None,
    })
}

/// `3d`, `+2w`, `1m`, `1y`, `5b`.
fn shorthand_offset(today: Date, text: &str) -> Option<Date> {
    let text = text.strip_prefix('+').unwrap_or(text);
    let symbol = text.chars().last()?;
    let unit = RecurrenceUnit::from_symbol(symbol)?;
    let amount: i64 = text[..text.len() - symbol.len_utf8()].parse().ok()?;
    Some(unit.shift(today, amount))
}

fn parse_amount(text: &str) -> Option<i64> {
    const WORDS: [&str; 13] = [
        "zero", "one", "two", "three", "four", "five", "six", "seven", "eight", "nine", "ten",
        "eleven", "twelve",
    ];
    match text {
        "a" | "an" => Some(1),
        _ => text
            .parse()
            .ok()
            .or_else(|| WORDS.iter().position(|w| *w == text).map(|n| n as i64)),
    }
}

fn parse_weekday(text: &str) -> Option<Weekday> {
    let weekday = match text {
        "monday" | "mon" => Weekday::Mon,
        "tuesday" | "tue" | "tues" => Weekday::Tue,
        "wednesday" | "wed" => Weekday::Wed,
        "thursday" | "thu" | "thurs" => Weekday::Thu,
        "friday" | "fri" => Weekday::Fri,
        "saturday" | "sat" => Weekday::Sat,
        "sunday" | "sun" => Weekday::Sun,
        _ => return None,
    };
    Some(weekday)
}

fn parse_month(text: &str) -> Option<Month> {
    if text.len() < 3 {
        return None;
    }
    (1..=12)
        .filter_map(|n| Month::try_from(n as u8).ok())
        .find(|month| month.name().to_lowercase().starts_with(text))
}

fn first_of_month(date: Date) -> Date {
    date.with_day(1).unwrap_or(date)
}

/// First day of the next occurrence of `month`, this year or next.
fn upcoming_month_start(today: Date, month: Month) -> Option<Date> {
    let number = month.number_from_month();
    let year = if number < today.month() { today.year() + 1 } else { today.year() };
    Date::from_ymd_opt(year, number, 1)
}

fn month_day(today: Date, month: Month, day: u32) -> Option<Date> {
    let this_year = Date::from_ymd_opt(today.year(), month.number_from_month(), day)?;
    if this_year >= today {
        Some(this_year)
    } else {
        Date::from_ymd_opt(today.year() + 1, month.number_from_month(), day)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // 2024-03-13 is a Wednesday.
    fn today() -> Date {
        parse_iso("2024-03-13").unwrap()
    }

    fn parse(phrase: &str) -> Option<String> {
        parse_phrase(phrase, today(), WeekStart::Monday).map(format_iso)
    }

    #[test]
    fn simple_words() {
        assert_eq!(parse("today").unwrap(), "2024-03-13");
        assert_eq!(parse("Tomorrow").unwrap(), "2024-03-14");
        assert_eq!(parse("yesterday").unwrap(), "2024-03-12");
    }

    #[test]
    fn offsets() {
        assert_eq!(parse("in one week").unwrap(), "2024-03-20");
        assert_eq!(parse("in 3 days").unwrap(), "2024-03-16");
        assert_eq!(parse("in a month").unwrap(), "2024-04-13");
        assert_eq!(parse("2 weeks").unwrap(), "2024-03-27");
        assert_eq!(parse("3d").unwrap(), "2024-03-16");
        assert_eq!(parse("1 day ago").unwrap(), "2024-03-12");
    }

    #[test]
    fn weekdays() {
        assert_eq!(parse("friday").unwrap(), "2024-03-15");
        assert_eq!(parse("wednesday").unwrap(), "2024-03-13");
        assert_eq!(parse("next tuesday").unwrap(), "2024-03-19");
        assert_eq!(parse("next week").unwrap(), "2024-03-18");
    }

    #[test]
    fn ends_and_months() {
        assert_eq!(parse("end of february").unwrap(), "2025-02-28");
        assert_eq!(parse("end of march").unwrap(), "2024-03-31");
        assert_eq!(parse("end of month").unwrap(), "2024-03-31");
        assert_eq!(parse("end of week").unwrap(), "2024-03-17");
        assert_eq!(parse("next month").unwrap(), "2024-04-01");
        assert_eq!(parse("april 2").unwrap(), "2024-04-02");
        assert_eq!(parse("2 jan").unwrap(), "2025-01-02");
        assert_eq!(parse("banana"), None);
    }

    #[test]
    fn rewrites_lines() {
        let rewritten = rewrite_relative_dates(
            "Call bob due:tomorrow t:in one week +work note:today",
            today(),
            WeekStart::Monday,
        );
        assert_eq!(rewritten, "Call bob due:2024-03-14 t:2024-03-20 +work note:today");
        let untouched = "Pay due:2024-01-01 due:soonish";
        assert_eq!(rewrite_relative_dates(untouched, today(), WeekStart::Monday), untouched);
    }
}
