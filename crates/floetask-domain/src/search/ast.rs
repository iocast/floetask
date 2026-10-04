use regex::Regex;

use crate::recurrence::RecurrenceUnit;

#[derive(Debug, Clone)]
pub enum Expr {
    And(Box<Expr>, Box<Expr>),
    Or(Box<Expr>, Box<Expr>),
    Not(Box<Expr>),
    Project(NameMatch),
    Context(NameMatch),
    Date(DateField, DateCondition),
    Priority(PriorityCondition),
    Complete,
    Text(String),
    Regex(Regex),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NameMatch {
    /// `+` alone: has any project.
    Any,
    /// `+bi`: a project containing `bi`, case-insensitive.
    Contains(String),
    /// `+"big"`: exactly this project.
    Exact(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DateField {
    Due,
    Threshold,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DateCondition {
    Exists,
    /// `due:2021-06` matches every due date in June 2021.
    Prefix(String),
    Compare(CmpOp, DateValue),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PriorityCondition {
    Exists,
    Compare(CmpOp, char),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CmpOp {
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
}

impl CmpOp {
    pub fn holds<T: Ord>(self, left: T, right: T) -> bool {
        match self {
            CmpOp::Eq => left == right,
            CmpOp::Ne => left != right,
            CmpOp::Lt => left < right,
            CmpOp::Le => left <= right,
            CmpOp::Gt => left > right,
            CmpOp::Ge => left >= right,
        }
    }
}

/// A date in an expression, resolved against "today" when evaluated.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DateValue {
    pub base: DateBase,
    pub offset: Option<(i64, RecurrenceUnit)>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DateBase {
    Absolute(crate::date::Date),
    Today,
    Tomorrow,
    Yesterday,
}
