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
}
