//! Todo priority, `(A)` to `(Z)`.

use std::fmt;

/// A todo.txt priority letter. `A` is the most important.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Priority(char);

impl Priority {
    pub fn new(letter: char) -> Option<Self> {
        letter.is_ascii_uppercase().then_some(Self(letter))
    }

    pub fn letter(self) -> char {
        self.0
    }

    /// Every priority from `A` to `Z`.
    pub fn all() -> impl Iterator<Item = Priority> {
        ('A'..='Z').map(Priority)
    }

    /// Parses the `(A)` form.
    pub fn parse_marker(token: &str) -> Option<Self> {
        let mut chars = token.chars();
        match (chars.next(), chars.next(), chars.next(), chars.next()) {
            (Some('('), Some(letter), Some(')'), None) => Self::new(letter),
            _ => None,
        }
    }
}

impl fmt::Display for Priority {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_marker() {
        assert_eq!(Priority::parse_marker("(B)"), Priority::new('B'));
        assert_eq!(Priority::parse_marker("(b)"), None);
        assert_eq!(Priority::parse_marker("(BB)"), None);
    }
}
