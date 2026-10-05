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

/// How one file's board is laid out.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct BoardLayout {
    pub columns: BoardColumns,
    /// Split into one board per group when the list is grouped. Off keeps
    /// the sort order on a single board.
    pub grouped: bool,
}

/// The attribute the board would group by: the list's grouping attribute,
/// except status, since the columns already are statuses.
pub fn board_grouping(sorting: &Sorting) -> Option<Attribute> {
    sorting
        .grouping_attribute()
        .filter(|attribute| *attribute != Attribute::Status)
}

/// Lays a document out on its board. Search, attribute filters, view
/// toggles and sorting (including file order) apply exactly as in the list,
/// so the board always shows what the list shows. One addition: a status
/// that has a column is always shown (a `someday` column shows `someday`
/// todos). Open todos whose status has no column go to a trailing "other" lane,
/// unless their status is hidden by default.
///
/// When the list is grouped (the first sort criterion, unless the list is
/// in file order) and the layout asks for groups, every group gets its own
/// board with the same columns. Grouping by status is ignored, since the
/// columns already are statuses.
pub fn build_board(
    document: &TodoDocument,
    options: &ViewOptions,
    sorting: &Sorting,
    query: &Query,
    dates: &DateContext,
    statuses: &StatusSet,
    layout: &BoardLayout,
) -> Board {
    let columns = &layout.columns;
    let shown = |todo: &Todo| {
        let status_shown = match todo.status() {
            Some(status) => columns.has_status(status) || options.passes_status(todo, statuses, query),
            None => true,
        };
        options.passes_toggles(todo, dates)
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
    if sorting.file_order {
        // Documents list todos in file order already.
        if sorting.completed_last {
            todos.sort_by_key(|entry| entry.todo.is_complete());
        }
    } else {
        todos.sort_by(|a, b| Sorting::compare(&sorting.criteria, &a.todo, &b.todo, statuses).then(a.line.cmp(&b.line)));
    }

    let grouping = board_grouping(sorting).filter(|_| layout.grouped);
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
        board_sorted(columns, query, &Sorting::default(), true)
    }

    fn board_sorted(columns: &BoardColumns, query: &Query, sorting: &Sorting, grouped: bool) -> Board {
        board_with(columns, query, sorting, grouped, &ViewOptions::default())
    }

    fn board_with(
        columns: &BoardColumns,
        query: &Query,
        sorting: &Sorting,
        grouped: bool,
        options: &ViewOptions,
    ) -> Board {
        let dates = DateContext {
            today: parse_iso("2026-10-05").unwrap(),
            week_start: WeekStart::Monday,
            human_friendly: false,
        };
        build_board(
            &TodoDocument::parse(FILE, &[]),
            options,
            sorting,
            query,
            &dates,
            &StatusSet::default(),
            &BoardLayout {
                columns: columns.clone(),
                grouped,
            },
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
        let columns = BoardColumns::from_keys(["todo", "doing", "done"]);
        let board = board_sorted(&columns, &Query::Empty, &sorting, true);
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

        // Without groups the same sort order lands on one board.
        let flat = board_sorted(&columns, &Query::Empty, &sorting, false);
        assert!(!flat.is_grouped());
        let todo_lane: Vec<_> = flat.groups[0].lanes[0].todos.iter().map(|t| t.todo.body()).collect();
        assert_eq!(todo_lane, ["Urgent plain", "Plain task"]);
    }

    #[test]
    fn view_toggles_and_file_order_apply_like_in_the_list() {
        let hide_completed = ViewOptions {
            show_completed: false,
            ..ViewOptions::default()
        };
        let board = board_with(
            &BoardColumns::default(),
            &Query::Empty,
            &Sorting::default(),
            true,
            &hide_completed,
        );
        assert!(bodies(&board)[3].is_empty(), "Done column follows Show completed");

        let file_order = Sorting {
            file_order: true,
            ..Sorting::default()
        };
        let board = board_sorted(&BoardColumns::default(), &Query::Empty, &file_order, true);
        assert_eq!(bodies(&board)[0], ["Plain task", "Urgent plain"]);
    }
}
