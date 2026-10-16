//! Persisted "where I left off" state. Losing it costs nothing but layout.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use floetask_domain::calendar::CalendarMode;
use floetask_domain::listing::{Attribute, Sorting, ViewOptions};

/// A registered todo file and its optional done file for archiving.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TodoFileEntry {
    pub path: PathBuf,
    pub done_path: Option<PathBuf>,
}

impl TodoFileEntry {
    pub fn new(path: PathBuf) -> Self {
        Self { path, done_path: None }
    }

    pub fn file_name(&self) -> String {
        self.path
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_else(|| self.path.display().to_string())
    }
}

// Enums are stored by their lowercase name. The `other` variant comes last
// (serde requires it): an unknown name falls back to it.

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DrawerTab {
    Filters,
    Sorting,
    #[default]
    #[serde(other)]
    Attributes,
}

/// The order of the file drawer. Archive files never sort on their own:
/// they stay under the todo file that archives into them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FileSort {
    /// By file name, then by folder.
    Name,
    /// By folder path, then by file name.
    Folder,
    /// The order the user dragged the files into (the order of `files`).
    #[default]
    #[serde(other)]
    Custom,
}

/// How the active file is shown.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MainView {
    /// The status board with one column per status.
    Board,
    /// Todos on a day, week or month calendar by due date.
    Calendar,
    /// The grouped, sorted list.
    #[default]
    #[serde(other)]
    List,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct WindowState {
    pub width: f32,
    pub height: f32,
    pub x: Option<f32>,
    pub y: Option<f32>,
    pub maximized: bool,
}

impl Default for WindowState {
    fn default() -> Self {
        Self {
            width: 1000.0,
            height: 720.0,
            x: None,
            y: None,
            maximized: false,
        }
    }
}

/// Missing keys take their default, so older files stay valid.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct AppState {
    pub files: Vec<TodoFileEntry>,
    pub active_file: usize,
    /// The order of the file drawer.
    pub file_sort: FileSort,
    #[serde(with = "crate::persist::sorting")]
    pub sorting: Sorting,
    #[serde(with = "crate::persist::view_options")]
    pub view: ViewOptions,
    pub drawer_open: bool,
    pub drawer_tab: DrawerTab,
    /// The collapsible file list on the left.
    pub files_drawer_open: bool,
    #[serde(with = "crate::persist::attributes")]
    pub collapsed_sections: Vec<Attribute>,
    pub main_view: MainView,
    /// Day, week or month in the calendar view.
    #[serde(with = "crate::persist::calendar_mode")]
    pub calendar_mode: CalendarMode,
    /// The board splits into one board per group when the list is grouped.
    pub board_grouped: bool,
    /// Keys of the board groups the user collapsed (`BoardGroup::key`).
    pub collapsed_board_groups: Vec<String>,
    /// Keys of the list groups the user collapsed (`Group::key`).
    pub collapsed_list_groups: Vec<String>,
    pub window: WindowState,
    /// Where the note window was last, so it opens there again.
    pub note_window: WindowState,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            files: Vec::new(),
            active_file: 0,
            file_sort: FileSort::default(),
            sorting: Sorting::default(),
            view: ViewOptions::default(),
            drawer_open: false,
            drawer_tab: DrawerTab::default(),
            files_drawer_open: true,
            collapsed_sections: Vec::new(),
            main_view: MainView::default(),
            calendar_mode: CalendarMode::default(),
            board_grouped: true,
            collapsed_board_groups: Vec::new(),
            collapsed_list_groups: Vec::new(),
            window: WindowState::default(),
            note_window: WindowState {
                width: 760.0,
                height: 820.0,
                ..WindowState::default()
            },
        }
    }
}

impl AppState {
    pub fn active_entry(&self) -> Option<&TodoFileEntry> {
        self.files.get(self.active_file)
    }

    /// Registers a file (if new) and makes it active.
    pub fn open_file(&mut self, path: PathBuf) {
        match self.files.iter().position(|entry| entry.path == path) {
            Some(index) => self.active_file = index,
            None => {
                self.files.push(TodoFileEntry::new(path));
                self.active_file = self.files.len() - 1;
            }
        }
    }

    pub fn remove_file(&mut self, index: usize) {
        if index < self.files.len() {
            self.files.remove(index);
        }
        if self.active_file >= self.files.len() {
            self.active_file = self.files.len().saturating_sub(1);
        }
    }

    /// Whether the file at `index` is the archive of another registered file.
    /// Such a file is listed under that file, not on its own.
    pub fn is_archive_of_another(&self, index: usize) -> bool {
        let path = &self.files[index].path;
        self.files
            .iter()
            .enumerate()
            .any(|(other, entry)| other != index && entry.done_path.as_ref() == Some(path))
    }

    /// Indexes of the files the drawer lists on their own, in `file_sort`
    /// order. Archive files of another file are left out.
    pub fn drawer_order(&self) -> Vec<usize> {
        let mut order: Vec<usize> = (0..self.files.len())
            .filter(|&index| !self.is_archive_of_another(index))
            .collect();
        let name = |index: usize| self.files[index].file_name().to_lowercase();
        let folder = |index: usize| {
            self.files[index]
                .path
                .parent()
                .map(|parent| parent.to_string_lossy().to_lowercase())
                .unwrap_or_default()
        };
        match self.file_sort {
            FileSort::Custom => {}
            FileSort::Name => order.sort_by_cached_key(|&index| (name(index), folder(index))),
            FileSort::Folder => order.sort_by_cached_key(|&index| (folder(index), name(index))),
        }
        order
    }

    /// Moves the file at `from` to where the file at `to` is in the drawer,
    /// and switches the drawer to the custom order, starting from the order
    /// shown. A registered archive file moves with its todo file.
    pub fn move_file(&mut self, from: usize, to: usize) {
        let mut order = self.drawer_order();
        let (Some(from_position), Some(to_position)) = (
            order.iter().position(|&index| index == from),
            order.iter().position(|&index| index == to),
        ) else {
            return;
        };
        let moved = order.remove(from_position);
        order.insert(to_position, moved);

        let mut placed = vec![false; self.files.len()];
        let mut indexes = Vec::with_capacity(self.files.len());
        for index in order {
            indexes.push(index);
            placed[index] = true;
            if let Some(done) = &self.files[index].done_path
                && let Some(archive) = self.files.iter().position(|entry| &entry.path == done)
                && !placed[archive]
            {
                indexes.push(archive);
                placed[archive] = true;
            }
        }
        // Archives of archives keep their old relative order at the end.
        indexes.extend((0..self.files.len()).filter(|&index| !placed[index]));

        let active = self.active_entry().map(|entry| entry.path.clone());
        let mut old: Vec<Option<TodoFileEntry>> = std::mem::take(&mut self.files).into_iter().map(Some).collect();
        self.files = indexes.into_iter().filter_map(|index| old[index].take()).collect();
        if let Some(active) = active {
            self.active_file = self.files.iter().position(|entry| entry.path == active).unwrap_or(0);
        }
        self.file_sort = FileSort::Custom;
    }

    /// Collapses a board group, or expands it again.
    pub fn toggle_board_group(&mut self, key: &str) {
        toggle_key(&mut self.collapsed_board_groups, key);
    }

    /// Collapses a list group, or expands it again.
    pub fn toggle_list_group(&mut self, key: &str) {
        toggle_key(&mut self.collapsed_list_groups, key);
    }

    pub fn toggle_section(&mut self, attribute: Attribute) {
        match self.collapsed_sections.iter().position(|a| *a == attribute) {
            Some(index) => {
                self.collapsed_sections.remove(index);
            }
            None => self.collapsed_sections.push(attribute),
        }
    }
}

fn toggle_key(keys: &mut Vec<String>, key: &str) {
    match keys.iter().position(|k| k == key) {
        Some(index) => {
            keys.remove(index);
        }
        None => keys.push(key.to_owned()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn open_and_remove_files() {
        let mut state = AppState::default();
        state.open_file("a.txt".into());
        state.open_file("b.txt".into());
        state.open_file("a.txt".into());
        assert_eq!(state.files.len(), 2);
        assert_eq!(state.active_file, 0);
        state.active_file = 1;
        state.remove_file(1);
        assert_eq!(state.active_file, 0);
        state.remove_file(0);
        assert!(state.active_entry().is_none());
    }

    fn names(state: &AppState) -> Vec<String> {
        state.files.iter().map(TodoFileEntry::file_name).collect()
    }

    fn drawer_names(state: &AppState) -> Vec<String> {
        state
            .drawer_order()
            .into_iter()
            .map(|index| state.files[index].file_name())
            .collect()
    }

    /// `b.txt` in `/z`, `a.txt` and its open archive `done.txt` in `/y`,
    /// `c.txt` in `/x`.
    fn three_files() -> AppState {
        let mut state = AppState::default();
        for path in ["/z/b.txt", "/y/a.txt", "/y/done.txt", "/x/c.txt"] {
            state.open_file(path.into());
        }
        state.files[1].done_path = Some("/y/done.txt".into());
        state
    }

    #[test]
    fn drawer_sorts_files_but_leaves_archives_out() {
        let mut state = three_files();
        assert_eq!(drawer_names(&state), ["b.txt", "a.txt", "c.txt"]);
        state.file_sort = FileSort::Name;
        assert_eq!(drawer_names(&state), ["a.txt", "b.txt", "c.txt"]);
        state.file_sort = FileSort::Folder;
        assert_eq!(drawer_names(&state), ["c.txt", "a.txt", "b.txt"]);
    }

    #[test]
    fn moving_a_file_takes_its_archive_along() {
        let mut state = three_files();
        state.active_file = 1;
        state.move_file(1, 3);
        assert_eq!(names(&state), ["b.txt", "c.txt", "a.txt", "done.txt"]);
        assert_eq!(state.files[state.active_file].file_name(), "a.txt");
        state.move_file(2, 0);
        assert_eq!(names(&state), ["a.txt", "done.txt", "b.txt", "c.txt"]);
    }

    #[test]
    fn moving_a_file_starts_from_the_sorted_order() {
        let mut state = three_files();
        state.file_sort = FileSort::Name;
        // Shown as a, b, c: dropping c on b puts it between a and b.
        state.move_file(3, 0);
        assert_eq!(state.file_sort, FileSort::Custom);
        assert_eq!(drawer_names(&state), ["a.txt", "c.txt", "b.txt"]);
    }
}
