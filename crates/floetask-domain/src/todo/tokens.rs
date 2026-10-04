//! Splits a description into whitespace-separated tokens with their byte
//! spans, so edits can replace a single token in place.

use super::MULTILINE_SEPARATOR;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct Token<'a> {
    pub text: &'a str,
    pub start: usize,
    pub end: usize,
}

pub(super) fn is_separator(c: char) -> bool {
    c == ' ' || c == '\t' || c == MULTILINE_SEPARATOR
}

pub(super) fn tokens(text: &str) -> Vec<Token<'_>> {
    let mut result = Vec::new();
    let mut start = None;
    for (index, c) in text.char_indices() {
        match (is_separator(c), start) {
            (true, Some(s)) => {
                result.push(Token { text: &text[s..index], start: s, end: index });
                start = None;
            }
            (false, None) => start = Some(index),
            _ => {}
        }
    }
    if let Some(s) = start {
        result.push(Token { text: &text[s..], start: s, end: text.len() });
    }
    result
}

/// What a single description token means.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum TokenKind<'a> {
    Project(&'a str),
    Context(&'a str),
    Extension(&'a str, &'a str),
    Word,
}

pub(super) fn classify(token: &str) -> TokenKind<'_> {
    if let Some(name) = token.strip_prefix('+').filter(|n| !n.is_empty()) {
        return TokenKind::Project(name);
    }
    if let Some(name) = token.strip_prefix('@').filter(|n| !n.is_empty()) {
        return TokenKind::Context(name);
    }
    if let Some((key, value)) = token.split_once(':') {
        let valid = !key.is_empty()
            && !value.is_empty()
            && !value.contains(':')
            // URLs such as https://example.com are text, not extensions.
            && !value.starts_with("//");
        if valid {
            return TokenKind::Extension(key, value);
        }
    }
    TokenKind::Word
}

/// Removes the span `start..end` together with one adjacent separator, so
/// removing a token does not leave a double space behind.
pub(super) fn remove_span(text: &str, start: usize, end: usize) -> String {
    let before = &text[..start];
    let after = &text[end..];
    if let Some(stripped) = after.strip_prefix(' ') {
        format!("{before}{stripped}")
    } else if let Some(stripped) = before.strip_suffix(' ') {
        format!("{stripped}{after}")
    } else {
        format!("{before}{after}")
    }
}
