use std::cmp::Ordering;
use std::collections::BTreeSet;

use super::{Attribute, DateContext, ViewOptions};
use crate::date::parse_iso;
use crate::document::TodoDocument;
use crate::human_date::DateBucket;
use crate::search::Query;

/// One value in a drawer section, e.g. project `work` used by 3 todos.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttributeValue {
    pub value: String,
    /// Todos with this value in the currently filtered list.
    pub count: usize,
    pub overdue: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttributeSummary {
    pub attribute: Attribute,
    pub values: Vec<AttributeValue>,
}

impl AttributeSummary {
    pub fn has_overdue(&self) -> bool {
        self.values.iter().any(|v| v.overdue && v.count > 0)
    }
}

/// Drawer sections. Values come from every todo passing the view toggles,
/// counts only from todos also passing filters and search, so a value that
/// is filtered out can still be clicked.
pub fn summarize_attributes(
    document: &TodoDocument,
    options: &ViewOptions,
    query: &Query,
    dates: &DateContext,
) -> Vec<AttributeSummary> {
    let toggles = ViewOptions { show_hidden: options.show_hidden || options.show_hidden_attributes, ..options.clone() };
    let candidates: Vec<_> = document
        .todos()
        .filter(|entry| toggles.passes_toggles(entry.todo, dates))
        .collect();

    Attribute::ALL
        .into_iter()
        .map(|attribute| {
            let mut values: Vec<AttributeValue> = Vec::new();
            for entry in &candidates {
                let counted = options.passes_filters(entry.todo, dates)
                    && query.matches(entry.todo, dates.today);
                for value in attribute.values(entry.todo, dates) {
                    let overdue = attribute == Attribute::Due && is_overdue(&value, dates);
                    match values.iter_mut().find(|v| v.value == value) {
                        Some(existing) => existing.count += usize::from(counted),
                        None => values.push(AttributeValue { value, count: usize::from(counted), overdue }),
                    }
                }
            }
            values.sort_by(|a, b| compare_values(attribute, &a.value, &b.value));
            AttributeSummary { attribute, values }
        })
        .filter(|summary| !summary.values.is_empty())
        .collect()
}

/// Every project and context in the file, including `h:1` todos, for
/// autocomplete.
pub fn known_names(document: &TodoDocument) -> (BTreeSet<String>, BTreeSet<String>) {
    let mut projects = BTreeSet::new();
    let mut contexts = BTreeSet::new();
    for entry in document.todos() {
        projects.extend(entry.todo.projects().iter().cloned());
        contexts.extend(entry.todo.contexts().iter().cloned());
    }
    (projects, contexts)
}

fn is_overdue(value: &str, dates: &DateContext) -> bool {
    match parse_iso(value) {
        Some(date) => date < dates.today,
        None => value == DateBucket::Overdue.key(),
    }
}

fn compare_values(attribute: Attribute, a: &str, b: &str) -> Ordering {
    match attribute {
        Attribute::Pomodoro => a.parse::<u32>().ok().cmp(&b.parse::<u32>().ok()),
        _ => a.to_lowercase().cmp(&b.to_lowercase()).then(a.cmp(b)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::date::WeekStart;

    #[test]
    fn counts_follow_filters_but_values_stay() {
        let doc = TodoDocument::parse("a +x @home\nb +y @home\nc +x due:2020-01-01\nsecret +z h:1\n", &[]);
        let dates = DateContext {
            today: parse_iso("2024-03-13").unwrap(),
            week_start: WeekStart::Monday,
            human_friendly: false,
        };
        let mut options = ViewOptions::default();
        options.toggle_filter(Attribute::Projects, "x", false);
        let summary = summarize_attributes(&doc, &options, &Query::Empty, &dates);
        let projects = summary.iter().find(|s| s.attribute == Attribute::Projects).unwrap();
        let counts: Vec<_> = projects.values.iter().map(|v| (v.value.as_str(), v.count)).collect();
        assert_eq!(counts, vec![("x", 2), ("y", 0), ("z", 0)]);
        let due = summary.iter().find(|s| s.attribute == Attribute::Due).unwrap();
        assert!(due.has_overdue());

        let (projects, _) = known_names(&doc);
        assert!(projects.contains("z"));
    }
}
