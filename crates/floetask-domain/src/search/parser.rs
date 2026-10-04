//! Recursive-descent parser. Precedence: `not` > `and` > `or`.

use regex::Regex;
use thiserror::Error;

use super::ast::{CmpOp, DateBase, DateCondition, DateValue, Expr, PriorityCondition};
use super::lexer::{Token, lex};
use crate::date::parse_iso;
use crate::recurrence::RecurrenceUnit;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("{message}")]
pub struct ParseError {
    pub message: String,
    /// The input ended while more was expected: the user is still typing.
    pub at_end: bool,
}

pub(super) fn parse(input: &str) -> Result<Expr, ParseError> {
    let tokens = lex(input).map_err(|e| ParseError {
        message: e.message,
        at_end: e.at_end,
    })?;
    let mut parser = Parser { tokens, position: 0 };
    let expr = parser.or()?;
    match parser.peek() {
        None => Ok(expr),
        Some(token) => Err(parser.error(format!("unexpected {token:?}"))),
    }
}

struct Parser {
    tokens: Vec<Token>,
    position: usize,
}

impl Parser {
    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.position)
    }

    fn next(&mut self) -> Option<Token> {
        let token = self.tokens.get(self.position).cloned();
        self.position += 1;
        token
    }

    fn error(&self, message: impl Into<String>) -> ParseError {
        ParseError {
            message: message.into(),
            at_end: self.position >= self.tokens.len(),
        }
    }

    fn or(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.and()?;
        while self.peek() == Some(&Token::Or) {
            self.next();
            let right = self.and()?;
            left = Expr::Or(Box::new(left), Box::new(right));
        }
        Ok(left)
    }

    fn and(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.unary()?;
        loop {
            match self.peek() {
                Some(Token::And) => {
                    self.next();
                }
                Some(token) if starts_term(token) => {}
                _ => return Ok(left),
            }
            let right = self.unary()?;
            left = Expr::And(Box::new(left), Box::new(right));
        }
    }

    fn unary(&mut self) -> Result<Expr, ParseError> {
        if self.peek() == Some(&Token::Not) {
            self.next();
            return Ok(Expr::Not(Box::new(self.unary()?)));
        }
        self.primary()
    }

    fn primary(&mut self) -> Result<Expr, ParseError> {
        let Some(token) = self.next() else {
            return Err(self.error("expected a search term"));
        };
        match token {
            Token::LParen => {
                let inner = self.or()?;
                match self.next() {
                    Some(Token::RParen) => Ok(inner),
                    _ => Err(self.error("missing `)`")),
                }
            }
            Token::Project(name) => Ok(Expr::Project(name)),
            Token::Context(name) => Ok(Expr::Context(name)),
            Token::DateKey(field) => Ok(Expr::Date(field, self.date_condition()?)),
            Token::PriorityKeyword => Ok(Expr::Priority(self.priority_condition()?)),
            Token::PriorityShorthand(letter) => Ok(Expr::Priority(PriorityCondition::Compare(CmpOp::Eq, letter))),
            Token::Complete => Ok(Expr::Complete),
            Token::Text(text) => Ok(Expr::Text(text.to_lowercase())),
            Token::Regex(pattern) => Regex::new(&pattern).map(Expr::Regex).map_err(|e| ParseError {
                message: e.to_string(),
                at_end: false,
            }),
            other => {
                self.position -= 1;
                Err(ParseError {
                    message: format!("unexpected {other:?}"),
                    at_end: false,
                })
            }
        }
    }

    fn date_condition(&mut self) -> Result<DateCondition, ParseError> {
        match self.peek().cloned() {
            Some(Token::Cmp(op)) => {
                self.next();
                Ok(DateCondition::Compare(op, self.date_value()?))
            }
            Some(Token::Word(word)) if is_partial_iso(&word) => {
                self.next();
                Ok(DateCondition::Prefix(word))
            }
            Some(Token::Word(_)) => Ok(DateCondition::Compare(CmpOp::Eq, self.date_value()?)),
            _ => Ok(DateCondition::Exists),
        }
    }

    fn priority_condition(&mut self) -> Result<PriorityCondition, ParseError> {
        let Some(Token::Cmp(op)) = self.peek().cloned() else {
            return Ok(PriorityCondition::Exists);
        };
        self.next();
        match self.next() {
            Some(Token::Word(word)) => {
                let mut chars = word.chars();
                match (chars.next(), chars.next()) {
                    (Some(letter), None) if letter.is_ascii_alphabetic() => {
                        Ok(PriorityCondition::Compare(op, letter.to_ascii_uppercase()))
                    }
                    _ => Err(self.error("expected a priority letter")),
                }
            }
            _ => Err(self.error("expected a priority letter")),
        }
    }

    /// `2021-06-01`, `today`, `today+3d`, `today - 2w`, `tomorrow +1b`.
    fn date_value(&mut self) -> Result<DateValue, ParseError> {
        let Some(Token::Word(word)) = self.next() else {
            return Err(self.error("expected a date"));
        };
        if let Some(date) = parse_iso(&word) {
            return Ok(DateValue {
                base: DateBase::Absolute(date),
                offset: None,
            });
        }
        let split = word.find(['+', '-']).unwrap_or(word.len());
        let base = match &word[..split] {
            "today" => DateBase::Today,
            "tomorrow" => DateBase::Tomorrow,
            "yesterday" => DateBase::Yesterday,
            _ => {
                return Err(ParseError {
                    message: format!("`{word}` is not a date"),
                    at_end: false,
                });
            }
        };
        let attached = &word[split..];
        let offset = if !attached.is_empty() {
            Some(parse_offset(attached).ok_or_else(|| self.error("invalid date offset"))?)
        } else {
            self.separate_offset()?
        };
        Ok(DateValue { base, offset })
    }

    /// An offset written after a space: `+ 3d`, `+3d`, `- 3d`, `-3d`.
    fn separate_offset(&mut self) -> Result<Option<(i64, RecurrenceUnit)>, ParseError> {
        let offset_text = match self.peek().cloned() {
            Some(Token::Project(super::ast::NameMatch::Any)) => match self.tokens.get(self.position + 1) {
                Some(Token::Word(amount)) => Some((2, format!("+{amount}"))),
                _ => None,
            },
            Some(Token::Project(super::ast::NameMatch::Contains(amount))) => Some((1, format!("+{amount}"))),
            Some(Token::Word(word)) if word == "-" => match self.tokens.get(self.position + 1) {
                Some(Token::Word(amount)) => Some((2, format!("-{amount}"))),
                _ => None,
            },
            Some(Token::Word(word)) if word.starts_with('-') => Some((1, word)),
            _ => None,
        };
        let Some((consumed, text)) = offset_text else {
            return Ok(None);
        };
        match parse_offset(&text) {
            Some(offset) => {
                self.position += consumed;
                Ok(Some(offset))
            }
            None => Ok(None),
        }
    }
}

fn starts_term(token: &Token) -> bool {
    !matches!(token, Token::RParen | Token::And | Token::Or | Token::Cmp(_))
}

/// `+3d`, `-2w`.
fn parse_offset(text: &str) -> Option<(i64, RecurrenceUnit)> {
    let (sign, rest) = match text.chars().next()? {
        '+' => (1, &text[1..]),
        '-' => (-1, &text[1..]),
        _ => return None,
    };
    let unit = RecurrenceUnit::from_symbol(rest.chars().last()?)?;
    let digits = &rest[..rest.len() - 1];
    let amount: i64 = if digits.is_empty() { 1 } else { digits.parse().ok()? };
    Some((sign * amount, unit))
}

/// `2021`, `2021-06` or a full ISO date.
fn is_partial_iso(word: &str) -> bool {
    let shape_ok = |w: &str| {
        w.char_indices()
            .all(|(i, c)| if i == 4 || i == 7 { c == '-' } else { c.is_ascii_digit() })
    };
    matches!(word.len(), 4 | 7 | 10) && shape_ok(word)
}
