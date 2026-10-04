//! `notified.toml`: keys of notifications already shown. Keys include the
//! date, so only today's keys are worth keeping; older ones are pruned.

use std::collections::BTreeSet;
use std::path::PathBuf;
use std::sync::Mutex;

use serde::{Deserialize, Serialize};

use floetask_application::AppError;
use floetask_application::ports::NotificationLog;

use super::{read_toml, write_toml};

pub struct TomlNotificationLog {
    path: PathBuf,
    cache: Mutex<Option<LogFile>>,
}

impl TomlNotificationLog {
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
            cache: Mutex::new(None),
        }
    }

    fn with_log<T>(&self, action: impl FnOnce(&mut LogFile) -> T) -> T {
        let mut cache = self.cache.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        let today = chrono::Local::now().date_naive().to_string();
        let log = cache.get_or_insert_with(|| read_toml(&self.path).ok().flatten().unwrap_or_default());
        if log.date != today {
            *log = LogFile {
                date: today,
                keys: BTreeSet::new(),
            };
        }
        action(log)
    }
}

impl NotificationLog for TomlNotificationLog {
    fn contains(&self, key: &str) -> bool {
        self.with_log(|log| log.keys.contains(key))
    }

    fn insert(&self, key: &str) -> Result<(), AppError> {
        let snapshot = self.with_log(|log| {
            log.keys.insert(key.to_owned());
            LogFile {
                date: log.date.clone(),
                keys: log.keys.clone(),
            }
        });
        write_toml(&self.path, &snapshot)
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(default)]
struct LogFile {
    date: String,
    keys: BTreeSet<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn remembers_keys_across_instances() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("notified.toml");
        let log = TomlNotificationLog::new(path.clone());
        assert!(!log.contains("abc"));
        log.insert("abc").unwrap();
        assert!(TomlNotificationLog::new(path).contains("abc"));
    }
}
