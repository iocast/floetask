//! Turns search input into tokens.

use super::ast::{CmpOp, DateField, NameMatch};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum Token {
    LParen,
    RParen,
    Not,
    And,
    Or,
    Cmp(CmpOp),
    Project(NameMatch),
    Context(NameMatch),
    DateKey(DateField),
    PriorityKeyword,
    PriorityShorthand(char),
    Complete,
    Text(String),
    Regex(String),
    Word(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct LexError {
    pub message: String,
    pub at_end: bool,
}

const SPECIAL: &[char] = &['(', ')', '!', '&', '|', '=', '<', '>', '"', '\'', '/'];

pub(super) fn lex(input: &str) -> Result<Vec<Token>, LexError> {
    let chars: Vec<char> = input.chars().collect();
    let mut tokens = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        let next = chars.get(i + 1).copied();
        match c {
            c if c.is_whitespace() => i += 1,
            '(' => {
                if let (Some(letter), Some(')')) = (next, chars.get(i + 2))
                    && letter.is_ascii_uppercase()
                {
                    tokens.push(Token::PriorityShorthand(letter));
                    i += 3;
                } else {
                    tokens.push(Token::LParen);
                    i += 1;
                }
            }
            ')' => {
                tokens.push(Token::RParen);
                i += 1;
            }
            '!' if next == Some('=') => {
                tokens.push(Token::Cmp(CmpOp::Ne));
                i += 2;
            }
            '!' => {
                tokens.push(Token::Not);
                i += 1;
            }
            '&' | '|' => {
                if next != Some(c) {
                    return Err(LexError { message: format!("expected `{c}{c}`"), at_end: next.is_none() });
                }
                tokens.push(if c == '&' { Token::And } else { Token::Or });
                i += 2;
            }
            '=' => {
                tokens.push(Token::Cmp(CmpOp::Eq));
                i += if next == Some('=') { 2 } else { 1 };
            }
            '<' | '>' => {
                let or_equal = next == Some('=');
                tokens.push(Token::Cmp(match (c, or_equal) {
                    ('<', false) => CmpOp::Lt,
                    ('<', true) => CmpOp::Le,
                    ('>', false) => CmpOp::Gt,
                    _ => CmpOp::Ge,
                }));
                i += if or_equal { 2 } else { 1 };
            }
            '"' | '\'' => {
                let (text, end) = delimited(&chars, i, c)?;
                tokens.push(Token::Text(text));
                i = end;
            }
            '/' => {
                let (pattern, end) = delimited(&chars, i, '/')?;
                tokens.push(Token::Regex(pattern));
                i = end;
            }
            '+' | '@' => {
                let (name, end) = name_match(&chars, i + 1)?;
                tokens.push(if c == '+' { Token::Project(name) } else { Token::Context(name) });
                i = end;
            }
            _ => {
                let start = i;
                while i < chars.len() && !chars[i].is_whitespace() && !SPECIAL.contains(&chars[i]) {
                    i += 1;
                }
                let word: String = chars[start..i].iter().collect();
                push_word(&mut tokens, word);
            }
        }
    }
    Ok(tokens)
}

/// Reads `"..."`, `'...'` or `/.../` starting at `start`. A backslash escapes
/// the delimiter.
fn delimited(chars: &[char], start: usize, delimiter: char) -> Result<(String, usize), LexError> {
    let mut text = String::new();
    let mut i = start + 1;
    while i < chars.len() {
        match chars[i] {
            '\\' if chars.get(i + 1) == Some(&delimiter) => {
                if delimiter == '/' {
                    text.push('\\');
                }
                text.push(delimiter);
                i += 2;
            }
            c if c == delimiter => return Ok((text, i + 1)),
            c => {
                text.push(c);
                i += 1;
            }
        }
    }
    Err(LexError { message: format!("missing closing `{delimiter}`"), at_end: true })
}

fn name_match(chars: &[char], start: usize) -> Result<(NameMatch, usize), LexError> {
    match chars.get(start) {
        Some('"') => {
            let (name, end) = delimited(chars, start, '"')?;
            Ok((NameMatch::Exact(name), end))
        }
        Some(c) if !c.is_whitespace() && !SPECIAL.contains(c) => {
            let mut end = start;
            while end < chars.len() && !chars[end].is_whitespace() && !SPECIAL.contains(&chars[end]) {
                end += 1;
            }
            Ok((NameMatch::Contains(chars[start..end].iter().collect()), end))
        }
        _ => Ok((NameMatch::Any, start)),
    }
}

fn push_word(tokens: &mut Vec<Token>, word: String) {
    for (prefix, field) in [("due:", DateField::Due), ("t:", DateField::Threshold)] {
        if let Some(rest) = word.strip_prefix(prefix) {
            tokens.push(Token::DateKey(field));
            if !rest.is_empty() {
                tokens.push(Token::Word(rest.to_owned()));
            }
            return;
        }
    }
    tokens.push(match word.as_str() {
        "and" | "AND" => Token::And,
        "or" | "OR" => Token::Or,
        "not" | "NOT" => Token::Not,
        "pri" | "priority" => Token::PriorityKeyword,
        "complete" => Token::Complete,
        _ => Token::Word(word),
    });
}
