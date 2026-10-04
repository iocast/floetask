//! Recurrence rules, `rec:[+][N](d|b|w|m|y)`.

use std::cmp::Ordering;
use std::fmt;

use crate::date::{self, Date};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RecurrenceUnit {
    Day,
    BusinessDay,
    Week,
    Month,
    Year,
}

impl RecurrenceUnit {
    pub const ALL: [RecurrenceUnit; 5] = [
        RecurrenceUnit::Day,
        RecurrenceUnit::BusinessDay,
        RecurrenceUnit::Week,
        RecurrenceUnit::Month,
        RecurrenceUnit::Year,
    ];

    pub fn symbol(self) -> char {
        match self {
            RecurrenceUnit::Day => 'd',
            RecurrenceUnit::BusinessDay => 'b',
            RecurrenceUnit::Week => 'w',
            RecurrenceUnit::Month => 'm',
            RecurrenceUnit::Year => 'y',
        }
    }

    pub fn from_symbol(symbol: char) -> Option<Self> {
        Self::ALL.into_iter().find(|unit| unit.symbol() == symbol)
    }

    /// Moves `date` by `amount` of this unit; negative amounts go back.
    pub fn shift(self, date: Date, amount: i64) -> Date {
        match self {
            RecurrenceUnit::Day => date::add_days(date, amount),
            RecurrenceUnit::BusinessDay => date::add_business_days(date, amount),
            RecurrenceUnit::Week => date::add_days(date, amount * 7),
            RecurrenceUnit::Month => date::add_months(date, amount),
            RecurrenceUnit::Year => date::add_months(date, amount * 12),
        }
    }

    /// Approximate length in days, used only for ordering.
    fn approximate_days(self) -> f64 {
        match self {
            RecurrenceUnit::Day => 1.0,
            RecurrenceUnit::BusinessDay => 7.0 / 5.0,
            RecurrenceUnit::Week => 7.0,
            RecurrenceUnit::Month => 30.44,
            RecurrenceUnit::Year => 365.25,
        }
    }
}

/// A parsed `rec:` value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Recurrence {
    /// Strict recurrence (`rec:+1w`) counts from the previous due date
    /// instead of from the completion date.
    pub strict: bool,
    pub count: u32,
    pub unit: RecurrenceUnit,
}

impl Recurrence {
    pub fn parse(text: &str) -> Option<Self> {
        let (strict, rest) = match text.strip_prefix('+') {
            Some(rest) => (true, rest),
            None => (false, text),
        };
        let unit_symbol = rest.chars().last()?;
        let unit = RecurrenceUnit::from_symbol(unit_symbol)?;
        let digits = &rest[..rest.len() - unit_symbol.len_utf8()];
        let count = if digits.is_empty() {
            1
        } else {
            if !digits.bytes().all(|b| b.is_ascii_digit()) {
                return None;
            }
            digits.parse().ok()?
        };
        (count >= 1).then_some(Self { strict, count, unit })
    }

    /// Moves `date` forward by this interval.
    pub fn advance(&self, date: Date) -> Date {
        self.unit.shift(date, i64::from(self.count))
    }

    /// Semantic ordering for sorting: shorter intervals first, and for equal
    /// length relative recurrences before strict ones.
    pub fn semantic_cmp(&self, other: &Self) -> Ordering {
        let length = |r: &Self| f64::from(r.count) * r.unit.approximate_days();
        length(self)
            .total_cmp(&length(other))
            .then(self.strict.cmp(&other.strict))
    }
}

impl fmt::Display for Recurrence {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.strict {
            write!(f, "+")?;
        }
        if self.count != 1 {
            write!(f, "{}", self.count)?;
        }
        write!(f, "{}", self.unit.symbol())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_valid_forms() {
        let r = Recurrence::parse("+3m").unwrap();
        assert!(r.strict);
        assert_eq!(r.count, 3);
        assert_eq!(r.unit, RecurrenceUnit::Month);
        assert_eq!(Recurrence::parse("w").unwrap().count, 1);
        assert_eq!(Recurrence::parse("0d"), None);
        assert_eq!(Recurrence::parse("3x"), None);
        assert_eq!(Recurrence::parse("+-3d"), None);
    }

    #[test]
    fn display_round_trips() {
        for text in ["d", "+b", "2w", "+12m", "y"] {
            assert_eq!(Recurrence::parse(text).unwrap().to_string(), text);
        }
    }

    #[test]
    fn semantic_order() {
        let p = |s| Recurrence::parse(s).unwrap();
        assert_eq!(p("1d").semantic_cmp(&p("1w")), Ordering::Less);
        assert_eq!(p("1m").semantic_cmp(&p("1y")), Ordering::Less);
        assert_eq!(p("1w").semantic_cmp(&p("+1w")), Ordering::Less);
        assert_eq!(p("8d").semantic_cmp(&p("1w")), Ordering::Greater);
    }
}
