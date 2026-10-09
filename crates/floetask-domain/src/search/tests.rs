use super::*;
use crate::date::{WeekStart, parse_iso};

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
    query(input).matches(&Todo::parse(line), today(), WeekStart::Monday)
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
fn natural_date_phrases() {
    // Today is Wednesday 2024-03-13; the week ends on Sunday 2024-03-17.
    let friday = "a due:2024-03-15";
    let sunday = "a due:2024-03-17";
    let next_monday = "a due:2024-03-18";
    assert!(hits("due: end of week", friday));
    assert!(hits("due:end of the week", sunday));
    assert!(!hits("due: end of week", next_monday));
    assert!(hits("due: End Of Month", next_monday));
    assert!(hits("due: friday", friday));
    assert!(!hits("due: friday", sunday));
    assert!(hits("due: <= friday", friday));
    assert!(hits("due: > end of week", next_monday));
    assert!(hits("due: == next week", next_monday));
    assert!(hits("t: <= end of month", "a t:2024-03-31"));
    assert!(hits("due: end of week and +work", "a +work due:2024-03-14"));
    assert!(!hits("due: end of week and +work", "a +home due:2024-03-14"));
}

#[test]
fn deadline_follows_the_week_start() {
    let saturday = Todo::parse("a due:2024-03-16");
    let query = query("due: end of week");
    assert!(query.matches(&saturday, today(), WeekStart::Monday));
    assert!(!query.matches(&saturday, today(), WeekStart::Saturday));
}

#[test]
fn unknown_words_after_a_date_key_are_text() {
    assert!(matches!(query("due: soonish"), Query::Literal(_)));
    assert!(matches!(query("due: end of +work"), Query::Literal(_)));
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
    for input in [
        "+work and",
        "due: <",
        "(+work",
        "\"open",
        "pri >",
        "due: end",
        "due: end of",
        "due: <= next",
    ] {
        assert!(matches!(Query::interpret(input), Interpretation::Incomplete), "{input}");
    }
}
