//! A whole todo.txt file as an ordered list of lines.
//!
//! todo.txt has no ids, so todos are addressed by line number. Every edit
//! takes the raw text the caller last saw and fails with
//! [`DocumentError::StaleLine`] if the file changed underneath.

use thiserror::Error;

use crate::todo::Todo;

#[derive(Debug, Clone, Error, PartialEq, Eq)]
pub enum DocumentError {
    #[error("line {line} changed since it was read; reload and try again")]
    StaleLine { line: usize },
    #[error("line {line} does not exist")]
    MissingLine { line: usize },
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Line {
    Todo(Todo),
    /// Blank or excluded lines, written back untouched.
    Verbatim(String),
}

/// A todo with its zero-based line number in the file.
#[derive(Debug, Clone, Copy)]
pub struct NumberedTodo<'a> {
    pub line: usize,
    pub todo: &'a Todo,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TodoDocument {
    lines: Vec<Line>,
    line_ending: &'static str,
    trailing_newline: bool,
    exclude_prefixes: Vec<String>,
}

impl TodoDocument {
    /// Parses file content. Lines starting with one of `exclude_prefixes`
    /// are kept verbatim and never shown.
    pub fn parse(content: &str, exclude_prefixes: &[String]) -> Self {
        let line_ending = if content.contains("\r\n") { "\r\n" } else { "\n" };
        let trailing_newline = content.ends_with('\n');
        let body = content.strip_suffix('\n').unwrap_or(content);
        let body = body.strip_suffix('\r').unwrap_or(body);
        let lines = if body.is_empty() && !trailing_newline {
            Vec::new()
        } else {
            body.split('\n')
                .map(|line| line.strip_suffix('\r').unwrap_or(line))
                .map(|line| classify_line(line, exclude_prefixes))
                .collect()
        };
        Self {
            lines,
            line_ending,
            trailing_newline: trailing_newline || content.is_empty(),
            exclude_prefixes: exclude_prefixes.to_vec(),
        }
    }

    pub fn empty() -> Self {
        Self::parse("", &[])
    }

    /// Serialises back to file content, keeping the original line endings.
    pub fn to_content(&self) -> String {
        let mut content = self
            .lines
            .iter()
            .map(|line| match line {
                Line::Todo(todo) => todo.raw(),
                Line::Verbatim(text) => text.as_str(),
            })
            .collect::<Vec<_>>()
            .join(self.line_ending);
        if self.trailing_newline && !self.lines.is_empty() {
            content.push_str(self.line_ending);
        }
        content
    }

    pub fn todos(&self) -> impl Iterator<Item = NumberedTodo<'_>> {
        self.lines.iter().enumerate().filter_map(|(line, entry)| match entry {
            Line::Todo(todo) => Some(NumberedTodo { line, todo }),
            Line::Verbatim(_) => None,
        })
    }

    pub fn todo_count(&self) -> usize {
        self.todos().count()
    }

    pub fn todo_at(&self, line: usize) -> Option<&Todo> {
        match self.lines.get(line)? {
            Line::Todo(todo) => Some(todo),
            Line::Verbatim(_) => None,
        }
    }

    /// Appends todos at the end of the file. Trailing blank lines are reused
    /// so the file does not grow gaps.
    pub fn append(&mut self, todos: impl IntoIterator<Item = Todo>) {
        while matches!(self.lines.last(), Some(Line::Verbatim(text)) if text.trim().is_empty()) {
            self.lines.pop();
        }
        self.lines.extend(todos.into_iter().map(Line::Todo));
        self.trailing_newline = true;
    }

    /// Replaces the todo at `line` with zero or more todos.
    pub fn replace(
        &mut self,
        line: usize,
        expected_raw: &str,
        replacement: Vec<Todo>,
    ) -> Result<(), DocumentError> {
        self.check(line, expected_raw)?;
        self.lines
            .splice(line..=line, replacement.into_iter().map(Line::Todo));
        Ok(())
    }

    pub fn remove(&mut self, line: usize, expected_raw: &str) -> Result<Todo, DocumentError> {
        self.check(line, expected_raw)?;
        match self.lines.remove(line) {
            Line::Todo(todo) => Ok(todo),
            Line::Verbatim(_) => Err(DocumentError::MissingLine { line }),
        }
    }

    /// Removes every completed todo and returns them in file order.
    pub fn take_completed(&mut self) -> Vec<Todo> {
        let mut taken = Vec::new();
        self.lines.retain(|line| match line {
            Line::Todo(todo) if todo.is_complete() => {
                taken.push(todo.clone());
                false
            }
            _ => true,
        });
        taken
    }

    /// Applies `edit` to every todo, returning how many lines changed.
    pub fn map_todos(&mut self, edit: impl Fn(&Todo) -> Todo) -> usize {
        let mut changed = 0;
        for line in &mut self.lines {
            if let Line::Todo(todo) = line {
                let edited = edit(todo);
                if edited.raw() != todo.raw() {
                    *todo = edited;
                    changed += 1;
                }
            }
        }
        changed
    }

    pub fn exclude_prefixes(&self) -> &[String] {
        &self.exclude_prefixes
    }

    fn check(&self, line: usize, expected_raw: &str) -> Result<(), DocumentError> {
        match self.lines.get(line) {
            Some(Line::Todo(todo)) if todo.raw() == expected_raw => Ok(()),
            Some(Line::Todo(_)) => Err(DocumentError::StaleLine { line }),
            _ => Err(DocumentError::MissingLine { line }),
        }
    }
}

fn classify_line(line: &str, exclude_prefixes: &[String]) -> Line {
    let excluded = exclude_prefixes
        .iter()
        .any(|prefix| !prefix.is_empty() && line.starts_with(prefix.as_str()));
    if excluded || line.trim().is_empty() {
        Line::Verbatim(line.to_owned())
    } else {
        Line::Todo(Todo::parse(line))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn keeps_excluded_and_blank_lines() {
        let content = "## header\r\n(A) one\r\n\r\ntwo\r\n";
        let doc = TodoDocument::parse(content, &["##".to_owned()]);
        assert_eq!(doc.todo_count(), 2);
        assert_eq!(doc.to_content(), content);
    }

    #[test]
    fn replace_checks_expected_line() {
        let mut doc = TodoDocument::parse("one\ntwo\n", &[]);
        let err = doc.replace(1, "changed", vec![]).unwrap_err();
        assert_eq!(err, DocumentError::StaleLine { line: 1 });
        doc.replace(1, "two", vec![Todo::parse("2"), Todo::parse("3")]).unwrap();
        assert_eq!(doc.to_content(), "one\n2\n3\n");
    }

    #[test]
    fn append_reuses_trailing_blank_lines() {
        let mut doc = TodoDocument::parse("one\n\n\n", &[]);
        doc.append([Todo::parse("two")]);
        assert_eq!(doc.to_content(), "one\ntwo\n");
        let mut empty = TodoDocument::parse("", &[]);
        empty.append([Todo::parse("first")]);
        assert_eq!(empty.to_content(), "first\n");
    }

    #[test]
    fn take_completed_keeps_others() {
        let mut doc = TodoDocument::parse("x done\nopen\nx 2020-01-01 done2\n", &[]);
        let taken = doc.take_completed();
        assert_eq!(taken.len(), 2);
        assert_eq!(doc.to_content(), "open\n");
    }

    proptest! {
        #[test]
        fn content_round_trips(lines in proptest::collection::vec("[ -~]{0,40}", 0..12), crlf: bool, trailing: bool) {
            let ending = if crlf { "\r\n" } else { "\n" };
            let mut content = lines.join(ending);
            if trailing && !lines.is_empty() {
                content.push_str(ending);
            }
            let doc = TodoDocument::parse(&content, &["#".to_owned()]);
            prop_assert_eq!(doc.to_content(), content);
        }
    }
}
