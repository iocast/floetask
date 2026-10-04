use super::ast::{DateBase, DateCondition, DateField, DateValue, Expr, NameMatch, PriorityCondition};
use crate::date::{self, Date, format_iso};
use crate::todo::Todo;

pub(super) fn evaluate(expr: &Expr, todo: &Todo, today: Date) -> bool {
    match expr {
        Expr::And(left, right) => evaluate(left, todo, today) && evaluate(right, todo, today),
        Expr::Or(left, right) => evaluate(left, todo, today) || evaluate(right, todo, today),
        Expr::Not(inner) => !evaluate(inner, todo, today),
        Expr::Project(name) => matches_names(todo.projects(), name),
        Expr::Context(name) => matches_names(todo.contexts(), name),
        Expr::Date(field, condition) => {
            let value = match field {
                DateField::Due => todo.due(),
                DateField::Threshold => todo.threshold(),
            };
            matches_date(value, condition, today)
        }
        Expr::Priority(condition) => match (todo.priority(), condition) {
            (Some(_), PriorityCondition::Exists) => true,
            (Some(priority), PriorityCondition::Compare(op, letter)) => op.holds(priority.letter(), *letter),
            (None, _) => false,
        },
        Expr::Complete => todo.is_complete(),
        Expr::Text(needle) => todo.raw().to_lowercase().contains(needle.as_str()),
        Expr::Regex(regex) => regex.is_match(todo.raw()),
    }
}

fn matches_names(names: &[String], wanted: &NameMatch) -> bool {
    match wanted {
        NameMatch::Any => !names.is_empty(),
        NameMatch::Exact(exact) => names.iter().any(|name| name == exact),
        NameMatch::Contains(part) => {
            let part = part.to_lowercase();
            names.iter().any(|name| name.to_lowercase().contains(&part))
        }
    }
}

fn matches_date(value: Option<Date>, condition: &DateCondition, today: Date) -> bool {
    let Some(value) = value else {
        return false;
    };
    match condition {
        DateCondition::Exists => true,
        DateCondition::Prefix(prefix) => format_iso(value).starts_with(prefix.as_str()),
        DateCondition::Compare(op, target) => op.holds(value, resolve(target, today)),
    }
}

fn resolve(value: &DateValue, today: Date) -> Date {
    let base = match value.base {
        DateBase::Absolute(date) => date,
        DateBase::Today => today,
        DateBase::Tomorrow => date::add_days(today, 1),
        DateBase::Yesterday => date::add_days(today, -1),
    };
    match value.offset {
        None => base,
        Some((amount, unit)) => unit.shift(base, amount),
    }
}
