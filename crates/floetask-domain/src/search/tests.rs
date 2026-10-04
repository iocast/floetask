use super::*;
use crate::date::parse_iso;

fn today() -> Date {
    parse_iso("2024-03-13").unwrap()
}

fn query(input: &str) -> Query {
    match Query::interpret(input) {
        Interpretation::Ready(query) => query,
        Interpretation::Incomplete => panic!("`{input}` is incomplete"),
    }
}

fn hits(input: &str, line: &str) -> bool {
    query(input).matches(&Todo::parse(line), today())
}

#[test]
fn plain_text_is_case_insensitive_substring() {
    assert!(matches!(query("Milk bread"), Query::Literal(_)));
    assert!(hits("milk", "Buy MILK"));
    assert!(!hits("eggs", "Buy MILK"));
}

#[test]
fn projects_and_contexts() {
    assert!(hits("+", "a +big"));
    assert!(!hits("+", "a @home"));
    assert!(hits("+bi", "a +big"));
    assert!(hits("+\"big\"", "a +big"));
    assert!(!hits("+\"bi\"", "a +big"));
    assert!(hits("@ho", "a @home"));
}

#[test]
fn logic_and_precedence() {
    let line = "a +work @home";
    assert!(hits("+work and @home", line));
    assert!(hits("+work @home", line));
    assert!(hits("+play or @home", line));
    assert!(!hits("not +work", line));
    assert!(hits("!+play && (+work || @office)", line));
    assert!(hits("+play or +work and @home", line));
    assert!(!hits("(+play or +work) and @office", line));
}

#[test]
fn dates() {
    let line = "a due:2024-03-14 t:2024-03-01";
    assert!(hits("due:", line));
    assert!(!hits("due:", "no date"));
    assert!(hits("due:2024-03", line));
    assert!(hits("due:2024", line));
    assert!(!hits("due:2024-04", line));
    assert!(hits("due: == tomorrow", line));
    assert!(hits("due:tomorrow", line));
    assert!(hits("due: < today+3d", line));
    assert!(hits("due: < today + 3d", line));
    assert!(!hits("due: < today", line));
    assert!(hits("due: > today - 1w", line));
    assert!(hits("t: <= 2024-03-01", line));
}

#[test]
fn priority_and_complete() {
    assert!(hits("(B)", "(B) task"));
    assert!(!hits("(B)", "(C) task"));
    assert!(hits("pri", "(C) task"));
    assert!(hits("priority >= B", "(C) task"));
    assert!(!hits("pri < B", "(C) task"));
    assert!(hits("complete", "x 2024-01-01 done"));
    assert!(hits("not complete", "open"));
}

#[test]
fn text_and_regex() {
    assert!(hits("\"call bob\"", "Call Bob today"));
    assert!(hits("/^Call \\w+/", "Call Bob today"));
    assert!(!hits("/^bob/", "Call Bob today"));
}

#[test]
fn unfinished_expressions_are_incomplete() {
    for input in ["+work and", "due: <", "(+work", "\"open", "pri >"] {
        assert!(matches!(Query::interpret(input), Interpretation::Incomplete), "{input}");
    }
}
