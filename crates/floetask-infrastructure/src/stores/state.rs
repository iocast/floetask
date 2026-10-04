//! `state.toml`: registered files, layout and view toggles.
//!
//! State is disposable: an unreadable file falls back to defaults instead of
//! failing the start-up.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use floetask_application::ports::StateStore;
use floetask_application::{AppError, AppState, DrawerTab, TodoFileEntry, WindowState};
use floetask_domain::listing::{Attribute, AttributeFilter, SortCriterion, Sorting, ViewOptions};

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
        read_toml::<StateFile>(&self.path)
            .ok()
            .flatten()
            .map(AppState::from)
            .unwrap_or_default()
    }

    fn save(&self, state: &AppState) -> Result<(), AppError> {
        write_toml(&self.path, &StateFile::from(state))
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(default)]
struct StateFile {
    active_file: usize,
    drawer_open: bool,
    drawer_tab: String,
    navigation_visible: Option<bool>,
    search_visible: bool,
    tabs_visible: Option<bool>,
    collapsed_sections: Vec<String>,
    files: Vec<FileEntry>,
    sorting: Option<SortingFile>,
    view: Option<ViewFile>,
    window: Option<WindowFile>,
}

#[derive(Debug, Serialize, Deserialize)]
struct FileEntry {
    path: PathBuf,
    done_path: Option<PathBuf>,
}

#[derive(Debug, Serialize, Deserialize)]
struct SortingFile {
    criteria: Vec<CriterionFile>,
    file_order: bool,
    completed_last: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct CriterionFile {
    attribute: String,
    descending: bool,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(default)]
struct ViewFile {
    show_completed: bool,
    show_hidden: bool,
    show_future_threshold: bool,
    show_future_due: bool,
    show_hidden_attributes: bool,
    hidden_categories: Vec<String>,
    filters: Vec<FilterFile>,
}

impl Default for ViewFile {
    fn default() -> Self {
        Self::from(&ViewOptions::default())
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct FilterFile {
    attribute: String,
    value: String,
    exclude: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct WindowFile {
    width: f32,
    height: f32,
    x: Option<f32>,
    y: Option<f32>,
    maximized: bool,
}

impl From<&AppState> for StateFile {
    fn from(state: &AppState) -> Self {
        Self {
            active_file: state.active_file,
            drawer_open: state.drawer_open,
            drawer_tab: match state.drawer_tab {
                DrawerTab::Attributes => "attributes",
                DrawerTab::Filters => "filters",
                DrawerTab::Sorting => "sorting",
            }
            .to_owned(),
            navigation_visible: Some(state.navigation_visible),
            search_visible: state.search_visible,
            tabs_visible: Some(state.tabs_visible),
            collapsed_sections: state.collapsed_sections.iter().map(|a| a.key().to_owned()).collect(),
            files: state
                .files
                .iter()
                .map(|f| FileEntry { path: f.path.clone(), done_path: f.done_path.clone() })
                .collect(),
            sorting: Some(SortingFile {
                criteria: state
                    .sorting
                    .criteria
                    .iter()
                    .map(|c| CriterionFile { attribute: c.attribute.key().to_owned(), descending: c.descending })
                    .collect(),
                file_order: state.sorting.file_order,
                completed_last: state.sorting.completed_last,
            }),
            view: Some(ViewFile::from(&state.view)),
            window: Some(WindowFile {
                width: state.window.width,
                height: state.window.height,
                x: state.window.x,
                y: state.window.y,
                maximized: state.window.maximized,
            }),
        }
    }
}

impl From<&ViewOptions> for ViewFile {
    fn from(view: &ViewOptions) -> Self {
        Self {
            show_completed: view.show_completed,
            show_hidden: view.show_hidden,
            show_future_threshold: view.show_future_threshold,
            show_future_due: view.show_future_due,
            show_hidden_attributes: view.show_hidden_attributes,
            hidden_categories: view.hidden_categories.iter().map(|a| a.key().to_owned()).collect(),
            filters: view
                .filters
                .iter()
                .map(|f| FilterFile { attribute: f.attribute.key().to_owned(), value: f.value.clone(), exclude: f.exclude })
                .collect(),
        }
    }
}

impl From<StateFile> for AppState {
    fn from(file: StateFile) -> Self {
        let defaults = AppState::default();
        let attributes = |keys: &[String]| keys.iter().filter_map(|k| Attribute::from_key(k)).collect();
        Self {
            files: file
                .files
                .into_iter()
                .map(|f| TodoFileEntry { path: f.path, done_path: f.done_path })
                .collect(),
            active_file: file.active_file,
            sorting: file.sorting.map(sorting_from_file).unwrap_or(defaults.sorting),
            view: file
                .view
                .map(|view| ViewOptions {
                    show_completed: view.show_completed,
                    show_hidden: view.show_hidden,
                    show_future_threshold: view.show_future_threshold,
                    show_future_due: view.show_future_due,
                    show_hidden_attributes: view.show_hidden_attributes,
                    hidden_categories: attributes(&view.hidden_categories),
                    filters: view
                        .filters
                        .into_iter()
                        .filter_map(|f| {
                            Some(AttributeFilter {
                                attribute: Attribute::from_key(&f.attribute)?,
                                value: f.value,
                                exclude: f.exclude,
                            })
                        })
                        .collect(),
                })
                .unwrap_or(defaults.view),
            drawer_open: file.drawer_open,
            drawer_tab: match file.drawer_tab.as_str() {
                "filters" => DrawerTab::Filters,
                "sorting" => DrawerTab::Sorting,
                _ => DrawerTab::Attributes,
            },
            navigation_visible: file.navigation_visible.unwrap_or(defaults.navigation_visible),
            search_visible: file.search_visible,
            tabs_visible: file.tabs_visible.unwrap_or(defaults.tabs_visible),
            collapsed_sections: attributes(&file.collapsed_sections),
            window: file
                .window
                .map(|w| WindowState { width: w.width, height: w.height, x: w.x, y: w.y, maximized: w.maximized })
                .unwrap_or(defaults.window),
        }
    }
}

/// Unknown attributes are dropped and missing ones appended, so the sort
/// list always covers every attribute exactly once.
fn sorting_from_file(file: SortingFile) -> Sorting {
    let mut criteria: Vec<SortCriterion> = Vec::new();
    for entry in file.criteria {
        if let Some(attribute) = Attribute::from_key(&entry.attribute)
            && !criteria.iter().any(|c| c.attribute == attribute)
        {
            criteria.push(SortCriterion { attribute, descending: entry.descending });
        }
    }
    for default in Sorting::default().criteria {
        if !criteria.iter().any(|c| c.attribute == default.attribute) {
            criteria.push(default);
        }
    }
    Sorting { criteria, file_order: file.file_order, completed_last: file.completed_last }
}

#[cfg(test)]
mod tests {
    use super::*;

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
