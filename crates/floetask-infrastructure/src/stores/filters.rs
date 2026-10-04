//! `filters.toml`: saved search filters.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use floetask_application::ports::SavedFilterStore;
use floetask_application::{AppError, SavedFilter};

use super::{read_toml, write_toml};

pub struct TomlSavedFilterStore {
    path: PathBuf,
}

impl TomlSavedFilterStore {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }
}

impl SavedFilterStore for TomlSavedFilterStore {
    fn load(&self) -> Result<Vec<SavedFilter>, AppError> {
        let file: Option<FiltersFile> = read_toml(&self.path)?;
        Ok(file
            .unwrap_or_default()
            .filter
            .into_iter()
            .map(|f| SavedFilter { name: f.name, query: f.query, suppress_notifications: f.suppress_notifications })
            .collect())
    }

    fn save(&self, filters: &[SavedFilter]) -> Result<(), AppError> {
        let file = FiltersFile {
            filter: filters
                .iter()
                .map(|f| FilterEntry {
                    name: f.name.clone(),
                    query: f.query.clone(),
                    suppress_notifications: f.suppress_notifications,
                })
                .collect(),
        };
        write_toml(&self.path, &file)
    }

    fn location(&self) -> PathBuf {
        self.path.clone()
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(default)]
struct FiltersFile {
    /// `[[filter]]` tables.
    filter: Vec<FilterEntry>,
}

#[derive(Debug, Serialize, Deserialize)]
struct FilterEntry {
    name: String,
    query: String,
    #[serde(default)]
    suppress_notifications: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_filters() {
        let dir = tempfile::tempdir().unwrap();
        let store = TomlSavedFilterStore::new(dir.path().join("filters.toml"));
        assert!(store.load().unwrap().is_empty());
        let filters = vec![SavedFilter { name: "Work".into(), query: "+work and due:".into(), suppress_notifications: true }];
        store.save(&filters).unwrap();
        assert_eq!(store.load().unwrap(), filters);
    }
}
