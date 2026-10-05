use super::build::group_by;
use super::{Attribute, DateContext, ListedTodo, Sorting, ViewOptions};
use crate::board::{BoardColumns, Column};
use crate::document::TodoDocument;
use crate::search::Query;
use crate::status::StatusSet;
use crate::todo::Todo;

/// One column of the board with its todos, sorted like the list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Lane {
    /// `None` for the trailing column of open todos whose status has no
    /// column of its own.
    pub column: Option<Column>,
    pub todos: Vec<ListedTodo>,
}

/// One board: a row of columns. When the list is grouped, each group gets
/// its own board, like swimlanes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoardGroup {
    /// The grouping attribute, `None` when the board is not grouped.
    pub attribute: Option<Attribute>,
    /// Header values, as in the list; empty for todos without the attribute.
    pub values: Vec<String>,
    pub lanes: Vec<Lane>,
}

/// Where a lane is: which group's board, which column.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LaneId {
    pub group: usize,
    pub lane: usize,
}

/// The status board of one file, one board per group.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Board {
    pub groups: Vec<BoardGroup>,
}

impl Board {
    pub fn todos(&self) -> impl Iterator<Item = &ListedTodo> {
        self.groups
            .iter()
            .flat_map(|group| group.lanes.iter())
            .flat_map(|lane| lane.todos.iter())
    }

    pub fn lane(&self, id: LaneId) -> Option<&Lane> {
        self.groups.get(id.group)?.lanes.get(id.lane)
    }

    /// The lane holding the todo on `line`.
    pub fn lane_of(&self, line: usize) -> Option<LaneId> {
        self.groups.iter().enumerate().find_map(|(group, board)| {
            board
                .lanes
                .iter()
                .position(|lane| lane.todos.iter().any(|entry| entry.line == line))
                .map(|lane| LaneId { group, lane })
        })
    }

    /// Whether the board is split into groups.
    pub fn is_grouped(&self) -> bool {
        self.groups.iter().any(|group| group.attribute.is_some())
    }
}

/// Lays a document out on its board. Search, attribute filters and view
/// toggles apply as in the list, with two differences: a status that has a
/// column is always shown (a `someday` column shows `someday` todos), and
/// completed todos appear in a `done` column even when the list hides them.
/// Open todos whose status has no column go to a trailing "other" lane,
/// unless their status is hidden by default.
///
/// When the list is grouped (the first sort criterion, unless the list is
/// in file order), every group gets its own board with the same columns.
/// Grouping by status is ignored, since the columns already are statuses.
pub fn build_board(
    document: &TodoDocument,
    options: &ViewOptions,
    sorting: &Sorting,
    query: &Query,
    dates: &DateContext,
    statuses: &StatusSet,
    columns: &BoardColumns,
) -> Board {
    let toggles = ViewOptions {
        show_completed: true,
        ..options.clone()
    };
    let shown = |todo: &Todo| {
        let status_shown = match todo.status() {
            Some(status) => columns.has_status(status) || options.passes_status(todo, statuses, query),
            None => true,
        };
        toggles.passes_toggles(todo, dates)
            && status_shown
            && options.passes_filters(todo, dates)
            && query.matches(todo, dates.today)
    };
    let mut todos: Vec<ListedTodo> = document
        .todos()
        .filter(|entry| shown(entry.todo))
        .map(|entry| ListedTodo {
            line: entry.line,
            todo: entry.todo.clone(),
        })
        .collect();
    todos.sort_by(|a, b| Sorting::compare(&sorting.criteria, &a.todo, &b.todo, statuses).then(a.line.cmp(&b.line)));

    let grouping = sorting
        .grouping_attribute()
        .filter(|attribute| *attribute != Attribute::Status);
    let groups: Vec<(Option<Attribute>, Vec<String>, Vec<ListedTodo>)> = match grouping {
        Some(attribute) => group_by(attribute, todos, dates)
            .into_iter()
            .map(|group| (group.attribute, group.values, group.todos))
            .collect(),
        None => vec![(None, Vec::new(), todos)],
    };

    let mut groups: Vec<BoardGroup> = groups
        .into_iter()
        .map(|(attribute, values, todos)| BoardGroup {
            attribute,
            values,
            lanes: into_lanes(todos, columns),
        })
        .collect();
    // Every board shows the same columns, so the "other" lane is kept or
    // dropped for all of them.
    let has_other = groups
        .iter()
        .any(|group| group.lanes.last().is_some_and(|lane| !lane.todos.is_empty()));
    if !has_other {
        for group in &mut groups {
            group.lanes.pop();
        }
    }
    Board { groups }
}

/// Distributes sorted todos into the configured columns plus a trailing
/// "other" lane, keeping their order.
fn into_lanes(todos: Vec<ListedTodo>, columns: &BoardColumns) -> Vec<Lane> {
    let mut lanes: Vec<Lane> = columns
        .columns()
        .iter()
        .map(|column| Lane {
            column: Some(column.clone()),
            todos: Vec::new(),
        })
        .chain(std::iter::once(Lane {
            column: None,
            todos: Vec::new(),
        }))
        .collect();
    let other = lanes.len() - 1;
    for entry in todos {
        let index = lanes
            .iter()
            .position(|lane| lane.column.as_ref().is_some_and(|c| c.holds(&entry.todo)));
        match index {
            Some(index) => lanes[index].todos.push(entry),
            None if !entry.todo.is_complete() => lanes[other].todos.push(entry),
            None => {}
        }
    }
    lanes
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::date::{WeekStart, parse_iso};

    const FILE: &str = "\
Plain task
(A) Write report status:doing
Legal review status:waiting
Learn Rust status:someday
Review PR status:in-review
x 2026-10-01 Sent invoice
(B) Urgent plain
";

    fn board(columns: &BoardColumns, query: &Query) -> Board {
        board_sorted(columns, query, &Sorting::default())
    }

    fn board_sorted(columns: &BoardColumns, query: &Query, sorting: &Sorting) -> Board {
        let dates = DateContext {
            today: parse_iso("2026-10-05").unwrap(),
            week_start: WeekStart::Monday,
            human_friendly: false,
        };
        let options = ViewOptions {
            show_completed: false,
            ..ViewOptions::default()
        };
        build_board(
            &TodoDocument::parse(FILE, &[]),
            &options,
            sorting,
            query,
            &dates,
            &StatusSet::default(),
            columns,
        )
    }

    fn bodies(board: &Board) -> Vec<Vec<&str>> {
        assert_eq!(board.groups.len(), 1);
        board.groups[0]
            .lanes
            .iter()
            .map(|lane| lane.todos.iter().map(|t| t.todo.body()).collect())
            .collect()
    }

    #[test]
    fn default_columns_with_other_lane() {
        let board = board(&BoardColumns::default(), &Query::Empty);
        assert_eq!(
            bodies(&board),
            vec![
                vec!["Urgent plain", "Plain task"],
                vec!["Write report"],
                vec!["Legal review"],
                vec!["Sent invoice"],
                vec!["Review PR"],
            ]
        );
        assert!(!board.is_grouped());
        assert_eq!(board.groups[0].lanes[4].column, None);
        assert_eq!(board.lane_of(1), Some(LaneId { group: 0, lane: 1 }));
    }

    #[test]
    fn a_someday_column_shows_someday_and_search_applies() {
        let columns = BoardColumns::from_keys(["someday", "todo"]);
        let board = board(&columns, &Query::Literal("r".into()));
        assert_eq!(
            bodies(&board),
            vec![
                vec!["Learn Rust"],
                vec!["Urgent plain"],
                vec!["Write report", "Legal review", "Review PR"]
            ]
        );
    }

    #[test]
    fn grouping_gives_each_group_its_own_board() {
        let mut sorting = Sorting::default();
        sorting.criteria.retain(|c| c.attribute != Attribute::Status);
        let board = board_sorted(
            &BoardColumns::from_keys(["todo", "doing", "done"]),
            &Query::Empty,
            &sorting,
        );
        assert!(board.is_grouped());
        let groups: Vec<_> = board
            .groups
            .iter()
            .map(|group| {
                let lanes: Vec<Vec<&str>> = group
                    .lanes
                    .iter()
                    .map(|lane| lane.todos.iter().map(|t| t.todo.body()).collect())
                    .collect();
                (group.values.join(","), lanes)
            })
            .collect();
        assert_eq!(
            groups,
            vec![
                ("A".to_owned(), vec![vec![], vec!["Write report"], vec![], vec![]]),
                ("B".to_owned(), vec![vec!["Urgent plain"], vec![], vec![], vec![]]),
                (
                    String::new(),
                    vec![
                        vec!["Plain task"],
                        vec![],
                        vec!["Sent invoice"],
                        vec!["Legal review", "Review PR"]
                    ]
                ),
            ]
        );
        assert_eq!(board.lane_of(2), Some(LaneId { group: 2, lane: 3 }));
    }
}
