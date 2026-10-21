use super::{Attribute, DateContext, Sorting, ViewOptions};
use crate::date::format_iso;
use crate::document::TodoDocument;
use crate::human_date;
use crate::search::Query;
use crate::status::StatusSet;
use crate::todo::Todo;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListedTodo {
    /// Which of the listed documents the todo is in: its index in the
    /// overview's documents, always 0 when one file is listed.
    pub file: usize,
    /// Zero-based line number in the file.
    pub line: usize,
    pub todo: Todo,
}

impl ListedTodo {
    /// Tells todos apart across files: line numbers repeat in an overview.
    pub fn id(&self) -> (usize, usize) {
        (self.file, self.line)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Group {
    /// The document of an overview's group (see `build_overview`); `None`
    /// when one file is listed.
    pub file: Option<usize>,
    /// The grouping attribute, `None` in file-order mode and in an overview.
    pub attribute: Option<Attribute>,
    /// Header values; empty for todos without the attribute.
    pub values: Vec<String>,
    pub todos: Vec<ListedTodo>,
}

impl Group {
    /// A stable name for the group (see `group_key`). An overview's file
    /// groups are better named by their path, which only the caller knows.
    pub fn key(&self) -> String {
        group_key(self.attribute, &self.values)
    }
}

/// A stable name for a group, e.g. `priority:A`, or `projects:` for todos
/// without a project, so a collapsed group stays collapsed across refreshes
/// and restarts.
pub fn group_key(attribute: Option<Attribute>, values: &[String]) -> String {
    let attribute = attribute.map(Attribute::key).unwrap_or_default();
    format!("{attribute}:{}", values.join(","))
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
    statuses: &StatusSet,
) -> TodoListing {
    let mut visible: Vec<ListedTodo> = document
        .todos()
        .filter(|entry| {
            options.passes_toggles(entry.todo, dates)
                && options.passes_status(entry.todo, statuses, query)
                && options.passes_filters(entry.todo, dates)
                && query.matches(entry.todo, dates.today, dates.week_start)
        })
        .map(|entry| ListedTodo {
            file: 0,
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
                file: None,
                attribute: None,
                values: Vec::new(),
                todos: visible,
            }]
        }
        Some(attribute) => {
            visible.sort_by(|a, b| {
                Sorting::compare(&sorting.criteria, &a.todo, &b.todo, statuses).then(a.line.cmp(&b.line))
            });
            group_by(attribute, visible, dates)
        }
    };
    TodoListing { groups, counts }
}

/// Builds one list over several documents, one group per document in the
/// order given. Each group holds what `build_listing` shows for that
/// document, in the same order but without its group headers; documents
/// with nothing to show are left out. Counts add up over all documents.
pub fn build_overview(
    documents: &[&TodoDocument],
    options: &ViewOptions,
    sorting: &Sorting,
    query: &Query,
    dates: &DateContext,
    statuses: &StatusSet,
) -> TodoListing {
    let mut overview = TodoListing::default();
    for (file, document) in documents.iter().enumerate() {
        let listing = build_listing(document, options, sorting, query, dates, statuses);
        overview.counts.total += listing.counts.total;
        overview.counts.visible += listing.counts.visible;
        overview.counts.completed += listing.counts.completed;
        let todos: Vec<ListedTodo> = listing
            .groups
            .into_iter()
            .flat_map(|group| group.todos)
            .map(|entry| ListedTodo { file, ..entry })
            .collect();
        if !todos.is_empty() {
            overview.groups.push(Group {
                file: Some(file),
                attribute: None,
                values: Vec::new(),
                todos,
            });
        }
    }
    overview
}

pub(super) fn group_by(attribute: Attribute, sorted: Vec<ListedTodo>, dates: &DateContext) -> Vec<Group> {
    let mut groups: Vec<Group> = Vec::new();
    for entry in sorted {
        let values = header_values(attribute, &entry.todo, dates);
        match groups.iter_mut().find(|group| group.values == values) {
            Some(group) => group.todos.push(entry),
            None => groups.push(Group {
                file: None,
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
    use crate::search::Interpretation;

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

    /// The default sorting without the status criterion, so groups follow
    /// priority.
    fn by_priority() -> Sorting {
        let mut sorting = Sorting::default();
        sorting.criteria.retain(|c| c.attribute != Attribute::Status);
        sorting
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
            &by_priority(),
            &Query::Empty,
            &dates(),
            &StatusSet::default(),
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
        let listing = build_listing(
            &doc,
            &ViewOptions::default(),
            &sorting,
            &Query::Empty,
            &dates(),
            &StatusSet::default(),
        );
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
        let listing = build_listing(
            &doc,
            &ViewOptions::default(),
            &sorting,
            &Query::Empty,
            &dates(),
            &StatusSet::default(),
        );
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
        let listing = build_listing(
            &doc,
            &options,
            &Sorting::default(),
            &Query::Empty,
            &dates(),
            &StatusSet::default(),
        );
        assert_eq!(listing.counts.visible, 2);

        let mut options = ViewOptions::default();
        options.toggle_filter(Attribute::Projects, "work", true);
        let listing = build_listing(
            &doc,
            &options,
            &Sorting::default(),
            &Query::Empty,
            &dates(),
            &StatusSet::default(),
        );
        assert!(listing.todos().all(|t| !t.todo.projects().contains(&"work".to_owned())));

        let mut options = ViewOptions::default();
        options.toggle_hidden_category(Attribute::Due);
        let listing = build_listing(
            &doc,
            &options,
            &Sorting::default(),
            &Query::Empty,
            &dates(),
            &StatusSet::default(),
        );
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
        let listing = build_listing(
            &doc,
            &ViewOptions::default(),
            &sorting,
            &Query::Empty,
            &dates,
            &StatusSet::default(),
        );
        assert_eq!(listing.groups[0].values, vec!["tomorrow".to_owned()]);
        assert_eq!(listing.groups[1].values, vec!["next week".to_owned()]);
    }

    #[test]
    fn overview_groups_by_file_and_adds_up_counts() {
        let first = TodoDocument::parse(FILE, &[]);
        let second = TodoDocument::parse(
            "(C) other file +work
x 2024-03-02 done too
",
            &[],
        );
        let empty = TodoDocument::parse("", &[]);
        let listing = build_overview(
            &[&first, &empty, &second],
            &ViewOptions::default(),
            &by_priority(),
            &Query::Empty,
            &dates(),
            &StatusSet::default(),
        );
        assert_eq!(
            lines(&listing),
            vec![
                vec![
                    "(A) a2 task due:2024-03-14",
                    "(A) a task due:2024-03-20",
                    "(B) b task +work",
                    "x 2024-03-01 done +work",
                    "plain task",
                ],
                vec!["(C) other file +work", "x 2024-03-02 done too"],
            ]
        );
        let files: Vec<_> = listing.groups.iter().map(|group| group.file).collect();
        assert_eq!(files, vec![Some(0), Some(2)]);
        assert!(listing.groups[1].todos.iter().all(|entry| entry.file == 2));
        assert_eq!(listing.groups[1].todos[0].id(), (2, 0));
        assert_eq!(
            listing.counts,
            Counts {
                total: 9,
                visible: 7,
                completed: 2
            }
        );
    }

    #[test]
    fn default_sort_groups_by_status_and_hides_someday() {
        let file = "\
Learn Rust status:someday
Write report status:doing
Plain task
Legal review status:waiting
Review PR status:in-review
x 2024-03-01 done status:doing
";
        let doc = TodoDocument::parse(file, &[]);
        let statuses = StatusSet::default();
        let listing = build_listing(
            &doc,
            &ViewOptions::default(),
            &Sorting::default(),
            &Query::Empty,
            &dates(),
            &statuses,
        );
        let headers: Vec<_> = listing.groups.iter().map(|g| g.values.join(",")).collect();
        assert_eq!(headers, vec!["doing", "todo", "waiting", "in-review", ""]);

        let mut options = ViewOptions::default();
        options.toggle_filter(Attribute::Status, "someday", false);
        let listing = build_listing(&doc, &options, &Sorting::default(), &Query::Empty, &dates(), &statuses);
        assert_eq!(lines(&listing), vec![vec!["Learn Rust status:someday"]]);

        let Interpretation::Ready(query) = Query::interpret("status:someday") else {
            panic!("query should be ready");
        };
        let listing = build_listing(
            &doc,
            &ViewOptions::default(),
            &Sorting::default(),
            &query,
            &dates(),
            &statuses,
        );
        assert_eq!(listing.counts.visible, 1);

        let options = ViewOptions {
            show_hidden_statuses: true,
            ..ViewOptions::default()
        };
        let listing = build_listing(&doc, &options, &Sorting::default(), &Query::Empty, &dates(), &statuses);
        assert_eq!(listing.groups[3].values, vec!["someday".to_owned()]);
    }
}
