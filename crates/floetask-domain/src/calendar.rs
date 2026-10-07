//! The calendar view's rules: which days a day, week or month shows, how to
//! step between periods, and which todos land on which day (by `due:`).
//! See `features/18-calendar.md`.

use std::collections::{BTreeMap, BTreeSet};

use crate::date::{Date, WeekStart, add_days, add_months, end_of_month, first_of_month, start_of_week};
use crate::listing::ListedTodo;

/// How much time the calendar shows at once.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CalendarMode {
    Day,
    Week,
    #[default]
    Month,
}

impl CalendarMode {
    pub const ALL: [CalendarMode; 3] = [CalendarMode::Day, CalendarMode::Week, CalendarMode::Month];
}

/// The days on screen, first and last included.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CalendarSpan {
    pub first: Date,
    pub last: Date,
}

impl CalendarSpan {
    /// The days `mode` shows around `anchor`. A month covers whole weeks, so
    /// the grid starts and ends with days of the neighbouring months.
    pub fn new(mode: CalendarMode, anchor: Date, week_start: WeekStart) -> Self {
        match mode {
            CalendarMode::Day => Self {
                first: anchor,
                last: anchor,
            },
            CalendarMode::Week => {
                let first = start_of_week(anchor, week_start);
                Self {
                    first,
                    last: add_days(first, 6),
                }
            }
            CalendarMode::Month => {
                let first = start_of_week(first_of_month(anchor), week_start);
                let last_week = start_of_week(end_of_month(anchor), week_start);
                Self {
                    first,
                    last: add_days(last_week, 6),
                }
            }
        }
    }

    pub fn contains(&self, date: Date) -> bool {
        self.first <= date && date <= self.last
    }

    /// Every day of the span in order.
    pub fn days(&self) -> impl Iterator<Item = Date> + '_ {
        let count = (self.last - self.first).num_days() + 1;
        (0..count).map(|offset| add_days(self.first, offset))
    }
}

/// The anchor `steps` periods away: days, weeks or months.
pub fn step(mode: CalendarMode, anchor: Date, steps: i64) -> Date {
    match mode {
        CalendarMode::Day => add_days(anchor, steps),
        CalendarMode::Week => add_days(anchor, 7 * steps),
        CalendarMode::Month => add_months(first_of_month(anchor), steps),
    }
}

/// The todos of `span` by due day, keeping the order they come in. A todo
/// listed in several groups (grouped by project, say) appears once.
pub fn todos_by_day<'a>(
    todos: impl IntoIterator<Item = &'a ListedTodo>,
    span: &CalendarSpan,
) -> BTreeMap<Date, Vec<&'a ListedTodo>> {
    let mut days: BTreeMap<Date, Vec<&'a ListedTodo>> = BTreeMap::new();
    let mut seen = BTreeSet::new();
    for entry in todos {
        if let Some(due) = entry.todo.due()
            && span.contains(due)
            && seen.insert(entry.line)
        {
            days.entry(due).or_default().push(entry);
        }
    }
    days
}

/// The open todos without a due date, which no calendar day shows, each
/// once and in the order they come in. Completed ones need no planning.
pub fn undated<'a>(todos: impl IntoIterator<Item = &'a ListedTodo>) -> Vec<&'a ListedTodo> {
    let mut seen = BTreeSet::new();
    todos
        .into_iter()
        .filter(|entry| entry.todo.due().is_none() && !entry.todo.is_complete() && seen.insert(entry.line))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::date::parse_iso;
    use crate::todo::Todo;

    fn d(s: &str) -> Date {
        parse_iso(s).unwrap()
    }

    fn listed(line: usize, text: &str) -> ListedTodo {
        ListedTodo {
            line,
            todo: Todo::parse(text),
        }
    }

    #[test]
    fn spans_cover_a_day_a_week_and_whole_weeks_of_a_month() {
        // 2026-10-06 is a Tuesday; October 2026 starts on a Thursday.
        let day = CalendarSpan::new(CalendarMode::Day, d("2026-10-06"), WeekStart::Monday);
        assert_eq!((day.first, day.last), (d("2026-10-06"), d("2026-10-06")));
        let week = CalendarSpan::new(CalendarMode::Week, d("2026-10-06"), WeekStart::Monday);
        assert_eq!((week.first, week.last), (d("2026-10-05"), d("2026-10-11")));
        let sunday_week = CalendarSpan::new(CalendarMode::Week, d("2026-10-06"), WeekStart::Sunday);
        assert_eq!(sunday_week.first, d("2026-10-04"));
        let month = CalendarSpan::new(CalendarMode::Month, d("2026-10-06"), WeekStart::Monday);
        assert_eq!((month.first, month.last), (d("2026-09-28"), d("2026-11-01")));
        assert_eq!(month.days().count(), 35);
    }

    #[test]
    fn steps_by_period() {
        assert_eq!(step(CalendarMode::Day, d("2026-10-31"), 1), d("2026-11-01"));
        assert_eq!(step(CalendarMode::Week, d("2026-10-06"), -1), d("2026-09-29"));
        assert_eq!(step(CalendarMode::Month, d("2026-01-31"), 1), d("2026-02-01"));
    }

    #[test]
    fn places_todos_on_their_due_day_within_the_span() {
        let todos = [
            listed(0, "Pay rent due:2026-10-01"),
            listed(1, "Call mom due:2026-10-06"),
            listed(2, "Plan trip due:2026-12-24"),
            listed(3, "Someday maybe"),
            listed(5, "x 2026-10-01 Done already"),
            listed(4, "Dentist due:2026-10-06"),
            // The same todo again, as a grouped list repeats it.
            listed(1, "Call mom due:2026-10-06"),
            listed(3, "Someday maybe"),
        ];
        let span = CalendarSpan::new(CalendarMode::Month, d("2026-10-06"), WeekStart::Monday);
        let days = todos_by_day(&todos, &span);
        let lines = |date: &str| days[&d(date)].iter().map(|entry| entry.line).collect::<Vec<_>>();
        assert_eq!(lines("2026-10-06"), [1, 4]);
        assert_eq!(lines("2026-10-01"), [0]);
        assert_eq!(days.len(), 2);
        assert_eq!(undated(&todos).iter().map(|entry| entry.line).collect::<Vec<_>>(), [3]);
    }
}
