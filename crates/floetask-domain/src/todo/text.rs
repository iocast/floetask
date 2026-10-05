//! The todo text as a card shows it: like [`Todo::body`], but contexts stay
//! where they were typed, because they often belong to the sentence
//! ("SAP Cloud Logging @ktbe mit @DanielWalther besprechen").

use super::Todo;
use super::parse::visible_text;
use super::tokens::{TokenKind, classify};

/// A piece of the display text: plain words, or one context to highlight.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TextSegment {
    /// Words with their spaces and line breaks.
    Text(String),
    /// A context name, without the `@`.
    Context(String),
}

impl Todo {
    /// The description without projects and known extensions, contexts kept
    /// in place, line breaks restored.
    pub fn display_text(&self) -> String {
        visible_text(self.description(), true)
    }

    /// [`Todo::display_text`] split into plain text and contexts, in order.
    pub fn text_segments(&self) -> Vec<TextSegment> {
        let mut segments = Vec::new();
        let mut text = String::new();
        for (index, line) in self.display_text().split('\n').enumerate() {
            if index > 0 {
                text.push('\n');
            }
            for (position, word) in line.split(' ').enumerate() {
                if position > 0 {
                    text.push(' ');
                }
                match classify(word) {
                    TokenKind::Context(name) => {
                        if !text.is_empty() {
                            segments.push(TextSegment::Text(std::mem::take(&mut text)));
                        }
                        segments.push(TextSegment::Context(name.to_owned()));
                    }
                    _ => text.push_str(word),
                }
            }
        }
        if !text.is_empty() {
            segments.push(TextSegment::Text(text));
        }
        segments
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn contexts_stay_in_the_sentence() {
        let todo =
            Todo::parse("2026-08-17 SAP Cloud Logging @ktbe mit @DanielWalther besprechen +pitch due:2026-10-10");
        assert_eq!(todo.body(), "SAP Cloud Logging mit besprechen");
        assert_eq!(
            todo.display_text(),
            "SAP Cloud Logging @ktbe mit @DanielWalther besprechen"
        );
        assert_eq!(
            todo.text_segments(),
            vec![
                TextSegment::Text("SAP Cloud Logging ".into()),
                TextSegment::Context("ktbe".into()),
                TextSegment::Text(" mit ".into()),
                TextSegment::Context("DanielWalther".into()),
                TextSegment::Text(" besprechen".into()),
            ]
        );
    }

    #[test]
    fn segments_keep_line_breaks_and_leading_contexts() {
        let todo = Todo::from_user_text("@home water plants\nthen @shop");
        assert_eq!(
            todo.text_segments(),
            vec![
                TextSegment::Context("home".into()),
                TextSegment::Text(" water plants\nthen ".into()),
                TextSegment::Context("shop".into()),
            ]
        );
    }
}
