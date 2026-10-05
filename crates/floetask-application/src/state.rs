//! Persisted "where I left off" state. Losing it costs nothing but layout.

use std::path::PathBuf;

use floetask_domain::listing::{Attribute, Sorting, ViewOptions};

/// A registered todo file and its optional done file for archiving.
#[derive(Debug, Clone, PartialEq, Eq)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DrawerTab {
    #[default]
    Attributes,
    Filters,
    Sorting,
}

/// How the active file is shown.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MainView {
    /// The grouped, sorted list.
    #[default]
    List,
    /// The status board with one column per status.
    Board,
}

#[derive(Debug, Clone, Copy, PartialEq)]
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

#[derive(Debug, Clone, PartialEq)]
pub struct AppState {
    pub files: Vec<TodoFileEntry>,
    pub active_file: usize,
    pub sorting: Sorting,
    pub view: ViewOptions,
    pub drawer_open: bool,
    pub drawer_tab: DrawerTab,
    /// The collapsible file list on the left.
    pub files_drawer_open: bool,
    pub collapsed_sections: Vec<Attribute>,
    pub main_view: MainView,
    /// The board splits into one board per group when the list is grouped.
    pub board_grouped: bool,
    /// Keys of the board groups the user collapsed (`BoardGroup::key`).
    pub collapsed_board_groups: Vec<String>,
    pub window: WindowState,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            files: Vec::new(),
            active_file: 0,
            sorting: Sorting::default(),
            view: ViewOptions::default(),
            drawer_open: false,
            drawer_tab: DrawerTab::default(),
            files_drawer_open: true,
            collapsed_sections: Vec::new(),
            main_view: MainView::default(),
            board_grouped: true,
            collapsed_board_groups: Vec::new(),
            window: WindowState::default(),
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

    /// Collapses a board group, or expands it again.
    pub fn toggle_board_group(&mut self, key: &str) {
        match self.collapsed_board_groups.iter().position(|k| k == key) {
            Some(index) => {
                self.collapsed_board_groups.remove(index);
            }
            None => self.collapsed_board_groups.push(key.to_owned()),
        }
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
}
