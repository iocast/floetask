use super::{Attribute, DateContext, Sorting, ViewOptions};
use crate::date::format_iso;
use crate::document::TodoDocument;
use crate::human_date;
use crate::search::Query;
use crate::todo::Todo;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListedTodo {
    /// Zero-based line number in the file.
    pub line: usize,
    pub todo: Todo,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Group {
    /// The grouping attribute, `None` in file-order mode.
    pub attribute: Option<Attribute>,
    /// Header values; empty for todos without the attribute.
    pub values: Vec<String>,
    pub todos: Vec<ListedTodo>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Counts {
    pub total: usize,
    pub visible: usize,
    pub completed: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TodoListing {
    pub groups: Vec<Group>,
    pub counts: Counts,
}

impl TodoListing {
    pub fn todos(&self) -> impl Iterator<Item = &ListedTodo> {
        self.groups.iter().flat_map(|group| group.todos.iter())
    }
}

/// Builds the visible, sorted and grouped list for a document.
pub fn build_listing(
    document: &TodoDocument,
    options: &ViewOptions,
    sorting: &Sorting,
    query: &Query,
    dates: &DateContext,
) -> TodoListing {
    let mut visible: Vec<ListedTodo> = document
        .todos()
        .filter(|entry| {
            options.passes_toggles(entry.todo, dates)
                && options.passes_filters(entry.todo, dates)
                && query.matches(entry.todo, dates.today)
        })
        .map(|entry| ListedTodo {
            line: entry.line,
            todo: entry.todo.clone(),
        })
        .collect();

    let counts = Counts {
        total: document.todo_count(),
        visible: visible.len(),
        completed: document.todos().filter(|e| e.todo.is_complete()).count(),
    };

    let groups = match sorting.grouping_attribute() {
        None => {
            if sorting.completed_last {
                visible.sort_by_key(|entry| entry.todo.is_complete());
            }
            vec![Group {
                attribute: None,
                values: Vec::new(),
                todos: visible,
            }]
        }
        Some(attribute) => {
            visible.sort_by(|a, b| Sorting::compare(&sorting.criteria, &a.todo, &b.todo).then(a.line.cmp(&b.line)));
            group_by(attribute, visible, dates)
        }
    };
    TodoListing { groups, counts }
}

fn group_by(attribute: Attribute, sorted: Vec<ListedTodo>, dates: &DateContext) -> Vec<Group> {
    let mut groups: Vec<Group> = Vec::new();
    for entry in sorted {
        let values = header_values(attribute, &entry.todo, dates);
        match groups.iter_mut().find(|group| group.values == values) {
            Some(group) => group.todos.push(entry),
            None => groups.push(Group {
                attribute: Some(attribute),
                values,
                todos: vec![entry],
            }),
        }
    }
    groups
}

fn header_values(attribute: Attribute, todo: &Todo, dates: &DateContext) -> Vec<String> {
    match attribute.date_of(todo) {
        Some(date) if dates.human_friendly => {
            let is_due = attribute == Attribute::Due;
            let label = human_date::display_bucket(date, dates.today, dates.week_start, is_due)
                .map(|bucket| bucket.key().to_owned())
                .unwrap_or_else(|| format_iso(date));
            vec![label]
        }
        _ => attribute.values(todo, dates),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::date::{WeekStart, parse_iso};
    use crate::listing::SortCriterion;

    fn dates() -> DateContext {
        DateContext {
            today: parse_iso("2024-03-13").unwrap(),
            week_start: WeekStart::Monday,
            human_friendly: false,
        }
    }

    fn lines(listing: &TodoListing) -> Vec<Vec<&str>> {
        listing
            .groups
            .iter()
            .map(|g| g.todos.iter().map(|t| t.todo.raw()).collect())
            .collect()
    }

    const FILE: &str = "\
(B) b task +work
(A) a task due:2024-03-20
plain task
x 2024-03-01 done +work
(A) a2 task due:2024-03-14
hidden h:1
later t:2024-04-01
";

    #[test]
    fn groups_by_first_criterion_with_missing_last() {
        let doc = TodoDocument::parse(FILE, &[]);
        let listing = build_listing(
            &doc,
            &ViewOptions::default(),
            &Sorting::default(),
            &Query::Empty,
            &dates(),
        );
        assert_eq!(
            lines(&listing),
            vec![
                vec!["(A) a2 task due:2024-03-14", "(A) a task due:2024-03-20"],
                vec!["(B) b task +work"],
                vec!["x 2024-03-01 done +work", "plain task"],
            ]
        );
        assert_eq!(listing.groups[0].values, vec!["A".to_owned()]);
        assert!(listing.groups[2].values.is_empty());
        assert_eq!(
            listing.counts,
            Counts {
                total: 7,
                visible: 5,
                completed: 1
            }
        );
    }

    #[test]
    fn descending_keeps_missing_last() {
        let doc = TodoDocument::parse(FILE, &[]);
        let mut sorting = Sorting::default();
        sorting.criteria[0] = SortCriterion {
            attribute: Attribute::Priority,
            descending: true,
        };
        let listing = build_listing(&doc, &ViewOptions::default(), &sorting, &Query::Empty, &dates());
        assert_eq!(listing.groups[0].values, vec!["B".to_owned()]);
        assert!(listing.groups.last().unwrap().values.is_empty());
    }

    #[test]
    fn file_order_with_completed_last() {
        let doc = TodoDocument::parse(FILE, &[]);
        let sorting = Sorting {
            file_order: true,
            completed_last: true,
            ..Sorting::default()
        };
        let listing = build_listing(&doc, &ViewOptions::default(), &sorting, &Query::Empty, &dates());
        assert_eq!(listing.groups.len(), 1);
        assert_eq!(
            listing.groups[0].todos.last().unwrap().todo.raw(),
            "x 2024-03-01 done +work"
        );
    }

    #[test]
    fn toggles_and_filters() {
        let doc = TodoDocument::parse(FILE, &[]);
        let mut options = ViewOptions {
            show_completed: false,
            show_hidden: true,
            show_future_threshold: true,
            ..ViewOptions::default()
        };
        options.toggle_filter(Attribute::Priority, "A", false);
        let listing = build_listing(&doc, &options, &Sorting::default(), &Query::Empty, &dates());
        assert_eq!(listing.counts.visible, 2);

        let mut options = ViewOptions::default();
        options.toggle_filter(Attribute::Projects, "work", true);
        let listing = build_listing(&doc, &options, &Sorting::default(), &Query::Empty, &dates());
        assert!(listing.todos().all(|t| !t.todo.projects().contains(&"work".to_owned())));

        let mut options = ViewOptions::default();
        options.toggle_hidden_category(Attribute::Due);
        let listing = build_listing(&doc, &options, &Sorting::default(), &Query::Empty, &dates());
        assert!(listing.todos().all(|t| t.todo.due().is_none()));
    }

    #[test]
    fn human_friendly_groups() {
        let doc = TodoDocument::parse(FILE, &[]);
        let sorting = Sorting {
            criteria: vec![SortCriterion {
                attribute: Attribute::Due,
                descending: false,
            }],
            ..Sorting::default()
        };
        let dates = DateContext {
            human_friendly: true,
            ..dates()
        };
        let listing = build_listing(&doc, &ViewOptions::default(), &sorting, &Query::Empty, &dates);
        assert_eq!(listing.groups[0].values, vec!["tomorrow".to_owned()]);
        assert_eq!(listing.groups[1].values, vec!["next week".to_owned()]);
    }
}
