//! `filters.toml`: saved search filters.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use floetask_application::ports::SavedFilterStore;
use floetask_application::saved_filters::default_filters;
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
        // No file yet: a new installation starts with the default filters.
        let Some(file) = read_toml::<FiltersFile>(&self.path)? else {
            return Ok(default_filters());
        };
        Ok(file.filter)
    }

    fn save(&self, filters: &[SavedFilter]) -> Result<(), AppError> {
        let file = FiltersFile {
            filter: filters.to_vec(),
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
    filter: Vec<SavedFilter>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_filters() {
        let dir = tempfile::tempdir().unwrap();
        let store = TomlSavedFilterStore::new(dir.path().join("filters.toml"));
        assert_eq!(store.load().unwrap(), default_filters());
        let filters = vec![SavedFilter {
            name: "Work".into(),
            query: "+work and due:".into(),
            suppress_notifications: true,
        }];
        store.save(&filters).unwrap();
        assert_eq!(store.load().unwrap(), filters);
    }

    #[test]
    fn suppress_notifications_is_optional() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("filters.toml");
        std::fs::write(&path, "[[filter]]\nname = \"Work\"\nquery = \"+work\"\n").unwrap();
        let filters = TomlSavedFilterStore::new(path).load().unwrap();
        assert!(!filters[0].suppress_notifications);
    }
}
