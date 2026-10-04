//! The search language: plain text or filter expressions.
//!
//! ```text
//! +project @context due:  due: < today+3d  t:2021-06  pri >= B  (A)
//! complete  "literal text"  /regex/  not  and  or  ( ... )
//! ```
//!
//! Juxtaposed terms are combined with `and`. Regexes use the Rust `regex`
//! syntax, which differs slightly from JavaScript's (no look-around).

mod ast;
mod eval;
mod lexer;
mod parser;

use crate::date::Date;
use crate::todo::Todo;

pub use ast::{CmpOp, DateField, DateValue, Expr, NameMatch};
pub use parser::ParseError;

/// A search typed by the user.
#[derive(Debug, Clone)]
pub enum Query {
    /// Matches nothing out: no search active.
    Empty,
    /// Case-insensitive substring match on the raw line.
    Literal(String),
    Expression(Expr),
}

/// Result of interpreting search input.
#[derive(Debug, Clone)]
pub enum Interpretation {
    Ready(Query),
    /// The input looks like an expression that is not finished yet; the
    /// caller keeps showing the results of the last valid query.
    Incomplete,
}

impl Query {
    pub fn interpret(input: &str) -> Interpretation {
        let input = input.trim();
        if input.is_empty() {
            return Interpretation::Ready(Query::Empty);
        }
        match parser::parse(input) {
            Ok(expr) => Interpretation::Ready(Query::Expression(expr)),
            Err(error) if error.at_end => Interpretation::Incomplete,
            Err(_) => Interpretation::Ready(Query::Literal(input.to_lowercase())),
        }
    }

    /// Parses an expression only, for saved filters that must be valid.
    pub fn parse_expression(input: &str) -> Result<Expr, ParseError> {
        parser::parse(input.trim())
    }

    pub fn matches(&self, todo: &Todo, today: Date) -> bool {
        match self {
            Query::Empty => true,
            Query::Literal(needle) => todo.raw().to_lowercase().contains(needle.as_str()),
            Query::Expression(expr) => eval::evaluate(expr, todo, today),
        }
    }

    pub fn is_empty(&self) -> bool {
        matches!(self, Query::Empty)
    }
}

#[cfg(test)]
mod tests;
