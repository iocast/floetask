use super::{DateContext, ListedTodo, Sorting, ViewOptions};
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

/// The board for one file.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Board {
    pub lanes: Vec<Lane>,
}

impl Board {
    pub fn todos(&self) -> impl Iterator<Item = &ListedTodo> {
        self.lanes.iter().flat_map(|lane| lane.todos.iter())
    }

    /// The lane holding the todo on `line`.
    pub fn lane_of(&self, line: usize) -> Option<usize> {
        self.lanes
            .iter()
            .position(|lane| lane.todos.iter().any(|entry| entry.line == line))
    }
}

/// Lays a document out on its board. Search, attribute filters and view
/// toggles apply as in the list, with two differences: a status that has a
/// column is always shown (a `someday` column shows `someday` todos), and
/// completed todos appear in a `done` column even when the list hides them.
/// Open todos whose status has no column go to a trailing "other" lane,
/// unless their status is hidden by default.
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

    let mut lanes: Vec<Lane> = columns
        .columns()
        .iter()
        .map(|column| Lane {
            column: Some(column.clone()),
            todos: Vec::new(),
        })
        .collect();
    let mut other = Lane {
        column: None,
        todos: Vec::new(),
    };
    for entry in document.todos().filter(|entry| shown(entry.todo)) {
        let listed = ListedTodo {
            line: entry.line,
            todo: entry.todo.clone(),
        };
        match lanes
            .iter_mut()
            .find(|lane| lane.column.as_ref().is_some_and(|c| c.holds(entry.todo)))
        {
            Some(lane) => lane.todos.push(listed),
            None if !entry.todo.is_complete() => other.todos.push(listed),
            None => {}
        }
    }
    if !other.todos.is_empty() {
        lanes.push(other);
    }
    for lane in &mut lanes {
        lane.todos
            .sort_by(|a, b| Sorting::compare(&sorting.criteria, &a.todo, &b.todo, statuses).then(a.line.cmp(&b.line)));
    }
    Board { lanes }
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
            &Sorting::default(),
            query,
            &dates,
            &StatusSet::default(),
            columns,
        )
    }

    fn bodies(board: &Board) -> Vec<Vec<&str>> {
        board
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
        assert_eq!(board.lanes[4].column, None);
        assert_eq!(board.lane_of(1), Some(1));
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
}
