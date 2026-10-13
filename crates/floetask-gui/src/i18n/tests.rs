use std::collections::{BTreeMap, BTreeSet};

use floetask_domain::Date;
use floetask_domain::date::parse_iso;
use fluent_syntax::ast::{Entry, Expression, InlineExpression, Pattern, PatternElement};

use super::*;

fn date(text: &str) -> Date {
    parse_iso(text).unwrap()
}

#[test]
fn formats_named_placeholders() {
    set_language(Language::English);
    let counts = trf(
        "counts",
        &[("visible", 1.into()), ("total", 2.into()), ("completed", 3.into())],
    );
    assert_eq!(counts, "1 of 2 todos, 3 completed");
    assert_eq!(tr("missing key"), "missing key");
}

#[test]
fn counts_pick_the_plural_form() {
    set_language(Language::English);
    assert_eq!(trf("renamed", &[("count", 1.into())]), "Updated 1 todo");
    assert_eq!(trf("renamed", &[("count", 4.into())]), "Updated 4 todos");
    assert_eq!(
        trf("notification_threshold", &[("days", 0.into())]),
        "Notify on the due date"
    );
    // French counts zero as singular.
    set_language(Language::French);
    assert_eq!(trf("archived", &[("count", 0.into())]), "0 tâche archivée");
    assert_eq!(trf("archived", &[("count", 2.into())]), "2 tâches archivées");
}

#[test]
fn switching_language_changes_every_string() {
    set_language(Language::German);
    assert_eq!(tr("settings_title"), "Einstellungen");
    assert_eq!(trf("calendar_more", &[("count", 2.into())]), "+2 weitere");
    set_language(Language::English);
    assert_eq!(tr("settings_title"), "Settings");
}

#[test]
fn the_setting_falls_back_from_locale_to_language_to_english() {
    assert_eq!(resolve("system", Some("de-CH")), Language::German);
    assert_eq!(resolve("system", Some("it_IT.UTF-8")), Language::Italian);
    assert_eq!(resolve("system", Some("pt-BR")), Language::English);
    assert_eq!(resolve("system", None), Language::English);
    assert_eq!(resolve("es", Some("de-DE")), Language::Spanish);
    // An unknown value means System.
    assert_eq!(resolve("klingon", Some("fr-CA")), Language::French);
}

#[test]
fn settings_offer_system_and_every_language() {
    let choices = language_choices();
    assert_eq!(&choices[..6], ["system", "en", "de", "fr", "es", "it"]);
}

#[test]
fn pseudo_locale_lengthens_and_brackets_english() {
    set_language(Language::Pseudo);
    assert_eq!(tr("save"), "[Save~~]");
    set_language(Language::English);
}

#[test]
fn dates_follow_the_language() {
    set_language(Language::English);
    assert_eq!(month_year(date("2026-10-06")), "October 2026");
    assert_eq!(long_date(date("2026-10-06")), "Tuesday, 6 October 2026");
    assert_eq!(date_range(date("2026-10-05"), date("2026-10-11")), "5 – 11 Oct 2026");
    assert_eq!(
        date_range(date("2026-09-28"), date("2026-10-04")),
        "28 Sep – 4 Oct 2026"
    );
    assert_eq!(
        date_range(date("2026-12-28"), date("2027-01-03")),
        "28 Dec 2026 – 3 Jan 2027"
    );
    assert_eq!(weekday_abbr(date("2026-10-06")), "Tue");

    set_language(Language::German);
    assert_eq!(month_year(date("2026-10-06")), "Oktober 2026");
    assert_eq!(long_date(date("2026-10-06")), "Dienstag, 6. Oktober 2026");
    assert_eq!(date_range(date("2026-10-05"), date("2026-10-11")), "5.–11. Okt. 2026");

    set_language(Language::French);
    assert_eq!(month_year(date("2026-10-06")), "Octobre 2026");
    assert_eq!(long_date(date("2026-10-06")), "Mardi 6 octobre 2026");
    set_language(Language::English);
}

#[test]
fn errors_are_translated_but_os_details_are_kept() {
    set_language(Language::German);
    assert_eq!(error_text(&AppError::EmptyTodo), "Die Aufgabe ist leer.");
    let io = AppError::io("todo.txt", "Access denied");
    assert_eq!(error_text(&io), "Kein Zugriff auf todo.txt: Access denied");
    set_language(Language::English);
}

// Quality checks (§13.8): every shipped language has exactly the English
// strings, with the same placeholders, and parses without errors.

/// Message id → the variables it uses, for one locale file.
fn messages(language: Language) -> BTreeMap<String, BTreeSet<String>> {
    let resource = fluent_syntax::parser::parse(language.source())
        .unwrap_or_else(|(_, errors)| panic!("{} does not parse: {errors:?}", language.code()));
    let mut messages = BTreeMap::new();
    for entry in &resource.body {
        match entry {
            Entry::Message(message) => {
                let mut variables = BTreeSet::new();
                if let Some(value) = &message.value {
                    pattern_variables(value, &mut variables);
                }
                let id = message.id.name.to_owned();
                assert!(
                    messages.insert(id.clone(), variables).is_none(),
                    "{}: `{id}` is defined twice",
                    language.code()
                );
            }
            Entry::Junk { content } => panic!("{}: cannot parse {content}", language.code()),
            _ => {}
        }
    }
    messages
}

fn pattern_variables(pattern: &Pattern<&str>, variables: &mut BTreeSet<String>) {
    for element in &pattern.elements {
        if let PatternElement::Placeable { expression } = element {
            expression_variables(expression, variables);
        }
    }
}

fn expression_variables(expression: &Expression<&str>, variables: &mut BTreeSet<String>) {
    match expression {
        Expression::Inline(inline) => inline_variables(inline, variables),
        Expression::Select { selector, variants } => {
            inline_variables(selector, variables);
            for variant in variants {
                pattern_variables(&variant.value, variables);
            }
        }
    }
}

fn inline_variables(inline: &InlineExpression<&str>, variables: &mut BTreeSet<String>) {
    match inline {
        InlineExpression::VariableReference { id } => {
            variables.insert(id.name.to_owned());
        }
        InlineExpression::Placeable { expression } => expression_variables(expression, variables),
        _ => {}
    }
}

#[test]
fn every_language_has_exactly_the_english_strings() {
    let english = messages(Language::English);
    for language in Language::SHIPPED {
        let translated = messages(language);
        let missing: Vec<_> = english.keys().filter(|id| !translated.contains_key(*id)).collect();
        let extra: Vec<_> = translated.keys().filter(|id| !english.contains_key(*id)).collect();
        assert!(missing.is_empty(), "{} misses {missing:?}", language.code());
        assert!(extra.is_empty(), "{} has ids English lacks: {extra:?}", language.code());
    }
}

#[test]
fn every_language_uses_the_english_placeholders() {
    let english = messages(Language::English);
    for language in Language::SHIPPED {
        for (id, variables) in messages(language) {
            let expected = &english[&id];
            let unknown: Vec<_> = variables.difference(expected).collect();
            assert!(unknown.is_empty(), "{}: `{id}` uses {unknown:?}", language.code());
        }
    }
}

#[test]
fn views_pass_no_literal_text_to_widgets() {
    let views = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/view");
    let mut literals = Vec::new();
    for entry in std::fs::read_dir(views).unwrap() {
        let path = entry.unwrap().path();
        let source = std::fs::read_to_string(&path).unwrap();
        for call in ["text(\"", "button(\""] {
            for (start, _) in source.match_indices(call) {
                let rest = &source[start + call.len()..];
                let literal = &rest[..rest.find('"').unwrap_or(rest.len())];
                // The product name is the same in every language.
                if literal.chars().any(char::is_alphabetic) && literal != "floetask" {
                    literals.push(format!("{}: {call}{literal}\"", path.display()));
                }
            }
        }
    }
    assert!(literals.is_empty(), "use tr() instead of {literals:#?}");
}
