//! `state.toml`: registered files, layout and view toggles.
//!
//! State is disposable: an unreadable file falls back to defaults instead of
//! failing the start-up.

use std::path::PathBuf;

use floetask_application::ports::StateStore;
use floetask_application::{AppError, AppState};

use super::{read_toml, write_toml};

pub struct TomlStateStore {
    path: PathBuf,
}

impl TomlStateStore {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }
}

impl StateStore for TomlStateStore {
    fn load(&self) -> AppState {
        read_toml(&self.path).ok().flatten().unwrap_or_default()
    }

    fn save(&self, state: &AppState) -> Result<(), AppError> {
        write_toml(&self.path, state)
    }
}

#[cfg(test)]
mod tests {
    use floetask_application::{DrawerTab, MainView};
    use floetask_domain::calendar::CalendarMode;
    use floetask_domain::listing::{Attribute, Sorting};

    use super::*;

    #[test]
    fn reads_the_format_older_versions_wrote() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("state.toml");
        std::fs::write(
            &path,
            r#"active_file = 1
drawer_open = true
drawer_tab = "sorting"
files_drawer_open = false
collapsed_sections = ["due", "nonsense"]
main_view = "board"
calendar_mode = "week"
board_grouped = false
collapsed_board_groups = ["priority:A"]
collapsed_list_groups = []

[[files]]
path = "/tmp/a.txt"

[[files]]
path = "/tmp/b.txt"
done_path = "/tmp/done.txt"

[sorting]
file_order = true
completed_last = false

[[sorting.criteria]]
attribute = "due"
descending = true

[view]
show_completed = false
hidden_categories = ["t"]

[[view.filters]]
attribute = "projects"
value = "work"
exclude = true

[window]
width = 1200.0
height = 800.0
x = 10.0
y = 20.0
maximized = true
"#,
        )
        .unwrap();
        let state = TomlStateStore::new(path).load();
        assert_eq!(state.active_file, 1);
        assert!(state.drawer_open && !state.files_drawer_open && !state.board_grouped);
        assert_eq!(state.drawer_tab, DrawerTab::Sorting);
        assert_eq!(state.collapsed_sections, [Attribute::Due]);
        assert_eq!(state.main_view, MainView::Board);
        assert_eq!(state.calendar_mode, CalendarMode::Week);
        assert_eq!(state.collapsed_board_groups, ["priority:A"]);
        assert_eq!(
            state.files[1].done_path.as_deref(),
            Some(std::path::Path::new("/tmp/done.txt"))
        );
        // Missing sort attributes come back at their default position.
        assert_eq!(state.sorting.criteria.len(), Sorting::default().criteria.len());
        assert!(
            state
                .sorting
                .criteria
                .iter()
                .any(|c| c.attribute == Attribute::Due && c.descending)
        );
        assert!(state.sorting.file_order);
        assert!(!state.view.show_completed);
        assert!(state.view.show_future_due, "missing view keys keep their default");
        assert_eq!(state.view.hidden_categories, [Attribute::Threshold]);
        assert_eq!(state.view.filters.len(), 1);
        assert_eq!(state.window.width, 1200.0);
        assert!(state.window.maximized);
        assert_eq!(state.note_window, AppState::default().note_window);
    }

    #[test]
    fn round_trips_state() {
        let dir = tempfile::tempdir().unwrap();
        let store = TomlStateStore::new(dir.path().join("state.toml"));
        let mut state = AppState::default();
        state.open_file("/tmp/todo.txt".into());
        state.files[0].done_path = Some("/tmp/done.txt".into());
        state.sorting.move_criterion(0, 1);
        state.sorting.toggle_direction(0);
        state.view.toggle_filter(Attribute::Projects, "work", true);
        state.drawer_tab = DrawerTab::Sorting;
        state.collapsed_sections.push(Attribute::Due);
        state.main_view = MainView::Board;
        state.calendar_mode = CalendarMode::Week;
        state.board_grouped = false;
        state.toggle_board_group("priority:A");
        state.toggle_list_group("projects:");
        state.note_window.x = Some(40.0);
        state.note_window.y = Some(60.0);
        state.note_window.width = 640.0;
        store.save(&state).unwrap();
        assert_eq!(store.load(), state);
    }

    #[test]
    fn corrupt_state_falls_back_to_defaults() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("state.toml");
        std::fs::write(&path, "this is = = not toml").unwrap();
        assert_eq!(TomlStateStore::new(path).load(), AppState::default());
    }
}
