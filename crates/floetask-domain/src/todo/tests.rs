use proptest::prelude::*;

use super::*;
use crate::date::parse_iso;

fn d(s: &str) -> Date {
    parse_iso(s).unwrap()
}

#[test]
fn parses_full_incomplete_line() {
    let todo = Todo::parse("(A) 2024-01-02 Call mom +family @phone due:2024-01-05 t:2024-01-03 rec:+1w pm:2 note:x");
    assert!(!todo.is_complete());
    assert_eq!(todo.priority(), Priority::new('A'));
    assert_eq!(todo.created(), Some(d("2024-01-02")));
    assert_eq!(todo.projects(), ["family"]);
    assert_eq!(todo.contexts(), ["phone"]);
    assert_eq!(todo.due(), Some(d("2024-01-05")));
    assert_eq!(todo.threshold(), Some(d("2024-01-03")));
    assert_eq!(todo.recurrence().unwrap().to_string(), "+w");
    assert_eq!(todo.pomodoros(), Some(2));
    assert_eq!(todo.extension("note"), Some("x"));
    assert_eq!(todo.body(), "Call mom note:x");
}

#[test]
fn parses_completed_line() {
    let todo = Todo::parse("x 2024-01-03 2024-01-01 Done thing pri:B");
    assert!(todo.is_complete());
    assert_eq!(todo.completed(), Some(d("2024-01-03")));
    assert_eq!(todo.created(), Some(d("2024-01-01")));
    assert_eq!(todo.stored_priority(), Priority::new('B'));
    assert_eq!(todo.priority(), None);
}

#[test]
fn x_without_space_is_not_completion() {
    let todo = Todo::parse("xylophone lessons");
    assert!(!todo.is_complete());
    assert_eq!(todo.body(), "xylophone lessons");
}

#[test]
fn priority_must_be_first() {
    let todo = Todo::parse("Call (A) mom");
    assert_eq!(todo.priority(), None);
}

#[test]
fn urls_are_not_extensions() {
    let todo = Todo::parse("Read https://example.com/a:b later");
    assert!(todo.extensions().is_empty());
    assert_eq!(todo.body(), "Read https://example.com/a:b later");
}

#[test]
fn hidden_flag() {
    assert!(Todo::parse("secret h:1").is_hidden());
    assert!(!Todo::parse("secret h:0").is_hidden());
}

#[test]
fn multiline_text_round_trips() {
    let todo = Todo::from_user_text("Shopping list\n- milk\n- bread +home");
    assert_eq!(todo.raw(), "Shopping list\u{10}- milk\u{10}- bread +home");
    assert_eq!(todo.editable_text(), "Shopping list\n- milk\n- bread +home");
    assert_eq!(todo.body(), "Shopping list\n- milk\n- bread");
    assert_eq!(todo.projects(), ["home"]);
}

#[test]
fn set_extension_replaces_in_place() {
    let todo = Todo::parse("Pay  rent due:2024-01-01   +home");
    let changed = todo.with_extension("due", Some("2024-02-01"));
    assert_eq!(changed.raw(), "Pay  rent due:2024-02-01   +home");
    let removed = todo.with_extension("due", None);
    assert_eq!(removed.raw(), "Pay  rent   +home");
    let added = Todo::parse("Pay rent").with_extension("t", Some("2024-01-01"));
    assert_eq!(added.raw(), "Pay rent t:2024-01-01");
}

#[test]
fn rename_and_remove_attributes() {
    let todo = Todo::parse("(B) Plan +trip @home +tripx");
    assert_eq!(
        todo.with_project_renamed("trip", "vacation").raw(),
        "(B) Plan +vacation @home +tripx"
    );
    assert_eq!(todo.without_context("home").raw(), "(B) Plan +trip +tripx");
    assert_eq!(todo.with_project_renamed("Trip", "x").raw(), todo.raw());
}

#[test]
fn completion_sets_dates_and_stores_priority() {
    let todo = Todo::parse("(A) Task +p");
    let done = todo.complete(d("2023-09-26")).completed;
    assert_eq!(done.raw(), "x 2023-09-26 2023-09-26 Task +p pri:A");
    let undone = done.uncomplete();
    assert_eq!(undone.raw(), "(A) 2023-09-26 Task +p");
}

#[test]
fn completion_keeps_existing_creation_date() {
    let done = Todo::parse("2023-01-01 Task").complete(d("2023-09-26")).completed;
    assert_eq!(done.raw(), "x 2023-09-26 2023-01-01 Task");
}

#[test]
fn recurrence_non_strict_keeps_threshold_gap() {
    // Worked example from FEATURES.md §4.3.
    let todo = Todo::parse("Water plants @home +quick due:2021-07-19 t:2021-07-09 rec:14d");
    let next = todo.complete(d("2021-07-13")).next.unwrap();
    assert_eq!(next.due(), Some(d("2021-07-27")));
    assert_eq!(next.threshold(), Some(d("2021-07-17")));
    assert_eq!(next.created(), Some(d("2021-07-13")));
    assert!(!next.is_complete());
    assert_eq!(next.contexts(), ["home"]);
}

#[test]
fn recurrence_strict_counts_from_due() {
    let todo = Todo::parse("(B) Rent due:2021-07-01 t:2021-06-25 rec:+1m");
    let next = todo.complete(d("2021-07-05")).next.unwrap();
    assert_eq!(next.due(), Some(d("2021-08-01")));
    assert_eq!(next.threshold(), Some(d("2021-07-25")));
    assert_eq!(next.priority(), Priority::new('B'));
}

#[test]
fn recurrence_without_due_gets_a_due_date() {
    let todo = Todo::parse("Stretch rec:+2d");
    let next = todo.complete(d("2021-07-05")).next.unwrap();
    assert_eq!(next.raw(), "2021-07-05 Stretch rec:+2d due:2021-07-07");
}

#[test]
fn recurrence_business_days() {
    // 2024-03-08 is a Friday.
    let todo = Todo::parse("Standup rec:1b");
    let next = todo.complete(d("2024-03-08")).next.unwrap();
    assert_eq!(next.due(), Some(d("2024-03-11")));
}

#[test]
fn status_defaults_to_todo_and_is_ignored_when_done() {
    assert_eq!(Todo::parse("Plain task").status(), Some("todo"));
    let doing = Todo::parse("(A) Write report +work status:doing due:2026-10-10");
    assert_eq!(doing.status(), Some("doing"));
    assert_eq!(doing.body(), "Write report");
    assert_eq!(Todo::parse("x 2026-10-01 Old status:doing").status(), None);
    // Custom values are kept; invalid ones are not a status but stay as text.
    assert_eq!(Todo::parse("Review status:in-review").status(), Some("in-review"));
    assert_eq!(Todo::parse("Odd status:Doing").status(), Some("todo"));
    assert_eq!(
        Todo::parse("Twice status:waiting status:doing").status(),
        Some("waiting")
    );
}

#[test]
fn set_status_replaces_in_place_and_todo_removes_it() {
    let todo = Todo::parse("Call bob status:waiting who:bob");
    assert_eq!(todo.with_status(Some("doing")).raw(), "Call bob status:doing who:bob");
    assert_eq!(todo.with_status(Some("todo")).raw(), "Call bob who:bob");
    assert_eq!(todo.with_status(None).raw(), "Call bob who:bob");
    assert_eq!(
        Todo::parse("Call bob").with_status(Some("doing")).raw(),
        "Call bob status:doing"
    );
    assert_eq!(
        Todo::parse("Odd status:Doing x").with_status(Some("doing")).raw(),
        "Odd status:doing x"
    );
}

#[test]
fn completing_removes_status_and_reopening_gives_todo() {
    let todo = Todo::parse("(A) Ship it status:doing +work");
    let done = todo.complete(d("2026-10-05")).completed;
    assert_eq!(done.raw(), "x 2026-10-05 2026-10-05 Ship it +work pri:A");
    assert_eq!(done.uncomplete().status(), Some("todo"));

    let recurring = Todo::parse("Water plants status:doing rec:1w");
    let next = recurring.complete(d("2026-10-05")).next.unwrap();
    assert_eq!(next.status_tag(), None);
}

proptest! {
    #[test]
    fn parse_preserves_raw(line in "[ -~\u{10}]{0,80}") {
        let todo = Todo::parse(&line);
        prop_assert_eq!(todo.raw(), line.as_str());
    }

    #[test]
    fn noop_edits_preserve_raw(line in "[ -~]{0,80}") {
        let todo = Todo::parse(&line);
        let same_priority = todo.with_priority(todo.priority());
        prop_assert_eq!(same_priority.raw(), line.as_str());
        let not_renamed = todo.with_project_renamed("zz-none", "x");
        prop_assert_eq!(not_renamed.raw(), line.as_str());
    }

    #[test]
    fn status_edits_touch_only_the_status_token(text in "[a-z]{1,8}( [a-z]{1,8}){0,4}", status in "[a-z][a-z0-9_-]{0,8}") {
        let line = format!("{text} +p status:{status} @c");
        let todo = Todo::parse(&line);
        prop_assert_eq!(todo.status_tag(), Some(status.as_str()));
        let changed = todo.with_status(Some("doing")).with_status(Some(status.as_str()));
        let expected = if status == "todo" { format!("{text} +p @c") } else { line.clone() };
        prop_assert_eq!(changed.raw(), expected.as_str());
    }

    #[test]
    fn complete_then_uncomplete_restores_priority(letter in proptest::char::range('A', 'Z'), text in "[a-z ]{1,30}") {
        let line = format!("({letter}) 2020-01-01 {}", text.trim());
        let todo = Todo::parse(&line);
        let back = todo.complete(parse_iso("2020-02-02").unwrap()).completed.uncomplete();
        prop_assert_eq!(back.priority(), todo.priority());
        prop_assert_eq!(back.created(), todo.created());
    }
}
