//! How the domain values inside stored application types are written.
//!
//! Settings, state, saved filters and colours derive serde themselves, but
//! the domain crate stays free of it. The few domain types they hold are
//! mapped here, each in a module used with `#[serde(with = "...")]`. Unknown
//! names in a hand-edited file fall back to a default or are dropped, so a
//! typo never makes the whole file unreadable.

use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// `monday`, `saturday` or `sunday`.
pub(crate) mod week_start {
    use floetask_domain::WeekStart;

    use super::*;

    pub fn serialize<S: Serializer>(value: &WeekStart, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(match value {
            WeekStart::Monday => "monday",
            WeekStart::Saturday => "saturday",
            WeekStart::Sunday => "sunday",
        })
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<WeekStart, D::Error> {
        Ok(match String::deserialize(deserializer)?.as_str() {
            "saturday" => WeekStart::Saturday,
            "sunday" => WeekStart::Sunday,
            _ => WeekStart::Monday,
        })
    }
}

/// `day`, `week` or `month`.
pub(crate) mod calendar_mode {
    use floetask_domain::calendar::CalendarMode;

    use super::*;

    pub fn serialize<S: Serializer>(value: &CalendarMode, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(match value {
            CalendarMode::Day => "day",
            CalendarMode::Week => "week",
            CalendarMode::Month => "month",
        })
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<CalendarMode, D::Error> {
        Ok(match String::deserialize(deserializer)?.as_str() {
            "day" => CalendarMode::Day,
            "week" => CalendarMode::Week,
            _ => CalendarMode::Month,
        })
    }
}

/// Attribute keys such as `"due"`; unknown keys are dropped.
pub(crate) mod attributes {
    use floetask_domain::listing::Attribute;

    use super::*;

    pub fn serialize<S: Serializer>(value: &[Attribute], serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_seq(value.iter().map(|attribute| attribute.key()))
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Vec<Attribute>, D::Error> {
        Ok(from_keys(&Vec::<String>::deserialize(deserializer)?))
    }

    pub(super) fn from_keys(keys: &[String]) -> Vec<Attribute> {
        keys.iter().filter_map(|key| Attribute::from_key(key)).collect()
    }
}

/// The sort list, by attribute key.
pub(crate) mod sorting {
    use floetask_domain::listing::{Attribute, SortCriterion, Sorting};

    use super::*;

    #[derive(Serialize, Deserialize)]
    struct SortingFile {
        criteria: Vec<CriterionFile>,
        file_order: bool,
        completed_last: bool,
    }

    #[derive(Serialize, Deserialize)]
    struct CriterionFile {
        attribute: String,
        descending: bool,
    }

    pub fn serialize<S: Serializer>(value: &Sorting, serializer: S) -> Result<S::Ok, S::Error> {
        SortingFile {
            criteria: value
                .criteria
                .iter()
                .map(|c| CriterionFile {
                    attribute: c.attribute.key().to_owned(),
                    descending: c.descending,
                })
                .collect(),
            file_order: value.file_order,
            completed_last: value.completed_last,
        }
        .serialize(serializer)
    }

    /// Unknown attributes are dropped and missing ones (added in a newer
    /// version, such as status) are inserted at their default position, so
    /// the sort list always covers every attribute exactly once.
    pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Sorting, D::Error> {
        let file = SortingFile::deserialize(deserializer)?;
        let mut criteria: Vec<SortCriterion> = Vec::new();
        for entry in file.criteria {
            if let Some(attribute) = Attribute::from_key(&entry.attribute)
                && !criteria.iter().any(|c| c.attribute == attribute)
            {
                criteria.push(SortCriterion {
                    attribute,
                    descending: entry.descending,
                });
            }
        }
        for (index, default) in Sorting::default().criteria.into_iter().enumerate() {
            if !criteria.iter().any(|c| c.attribute == default.attribute) {
                criteria.insert(index.min(criteria.len()), default);
            }
        }
        Ok(Sorting {
            criteria,
            file_order: file.file_order,
            completed_last: file.completed_last,
        })
    }
}

/// The drawer's view toggles and attribute filters.
pub(crate) mod view_options {
    use floetask_domain::listing::{Attribute, AttributeFilter, ViewOptions};

    use super::*;

    #[derive(Serialize, Deserialize)]
    #[serde(default)]
    struct ViewFile {
        show_completed: bool,
        show_hidden: bool,
        show_future_threshold: bool,
        show_future_due: bool,
        show_hidden_attributes: bool,
        show_hidden_statuses: bool,
        hidden_categories: Vec<String>,
        filters: Vec<FilterFile>,
    }

    impl Default for ViewFile {
        fn default() -> Self {
            Self::from(&ViewOptions::default())
        }
    }

    #[derive(Serialize, Deserialize)]
    struct FilterFile {
        attribute: String,
        value: String,
        exclude: bool,
    }

    impl From<&ViewOptions> for ViewFile {
        fn from(view: &ViewOptions) -> Self {
            Self {
                show_completed: view.show_completed,
                show_hidden: view.show_hidden,
                show_future_threshold: view.show_future_threshold,
                show_future_due: view.show_future_due,
                show_hidden_attributes: view.show_hidden_attributes,
                show_hidden_statuses: view.show_hidden_statuses,
                hidden_categories: view.hidden_categories.iter().map(|a| a.key().to_owned()).collect(),
                filters: view
                    .filters
                    .iter()
                    .map(|f| FilterFile {
                        attribute: f.attribute.key().to_owned(),
                        value: f.value.clone(),
                        exclude: f.exclude,
                    })
                    .collect(),
            }
        }
    }

    pub fn serialize<S: Serializer>(value: &ViewOptions, serializer: S) -> Result<S::Ok, S::Error> {
        ViewFile::from(value).serialize(serializer)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<ViewOptions, D::Error> {
        let view = ViewFile::deserialize(deserializer)?;
        Ok(ViewOptions {
            show_completed: view.show_completed,
            show_hidden: view.show_hidden,
            show_future_threshold: view.show_future_threshold,
            show_future_due: view.show_future_due,
            show_hidden_attributes: view.show_hidden_attributes,
            show_hidden_statuses: view.show_hidden_statuses,
            hidden_categories: super::attributes::from_keys(&view.hidden_categories),
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
    }
}

/// One status or column key: a plain key such as `"doing"`, or a table with
/// a display name, `{ key = "in-review", label = "In review" }`. Plain keys
/// keep older files valid and leave built-in statuses translated.
#[derive(Serialize, Deserialize)]
#[serde(untagged)]
enum StatusEntry {
    Key(String),
    Labelled { key: String, label: String },
}

impl StatusEntry {
    fn new(key: String, label: Option<&str>) -> Self {
        match label {
            Some(label) => Self::Labelled {
                key,
                label: label.to_owned(),
            },
            None => Self::Key(key),
        }
    }

    fn key(&self) -> &str {
        match self {
            Self::Key(key) | Self::Labelled { key, .. } => key,
        }
    }
}

/// `[statuses]`: the user's workflow statuses for the `status:` extension.
pub(crate) mod statuses {
    use floetask_domain::StatusSet;

    use super::*;

    #[derive(Serialize, Deserialize)]
    #[serde(default)]
    struct StatusesFile {
        /// Display and sort order; custom statuses go here too.
        order: Vec<StatusEntry>,
        /// Statuses left out of the default list, like `someday`.
        hidden: Vec<String>,
    }

    impl Default for StatusesFile {
        fn default() -> Self {
            Self::from(&StatusSet::default())
        }
    }

    impl From<&StatusSet> for StatusesFile {
        fn from(statuses: &StatusSet) -> Self {
            Self {
                order: statuses
                    .names()
                    .iter()
                    .map(|key| StatusEntry::new(key.clone(), statuses.label(key)))
                    .collect(),
                hidden: statuses.hidden().to_vec(),
            }
        }
    }

    pub fn serialize<S: Serializer>(value: &StatusSet, serializer: S) -> Result<S::Ok, S::Error> {
        StatusesFile::from(value).serialize(serializer)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<StatusSet, D::Error> {
        let file = StatusesFile::deserialize(deserializer)?;
        let keys = file.order.iter().map(|entry| entry.key().to_owned()).collect();
        let mut set = StatusSet::new(keys, file.hidden);
        for entry in &file.order {
            if let StatusEntry::Labelled { key, label } = entry {
                set.set_label(key, label);
            }
        }
        Ok(set)
    }
}

/// `[[boards]]`: board columns of one todo file each.
pub(crate) mod boards {
    use std::collections::BTreeMap;
    use std::path::{Path, PathBuf};

    use floetask_domain::board::BoardColumns;

    use super::*;

    #[derive(Serialize, Deserialize)]
    struct BoardFile {
        file: PathBuf,
        /// Column keys, with a display name for statuses only this board knows.
        columns: Vec<StatusEntry>,
    }

    impl BoardFile {
        fn new(file: &Path, columns: &BoardColumns) -> Self {
            Self {
                file: file.to_path_buf(),
                columns: columns
                    .keys()
                    .into_iter()
                    .map(|key| StatusEntry::new(key.clone(), columns.label(&key)))
                    .collect(),
            }
        }

        fn columns(&self) -> BoardColumns {
            let mut columns = BoardColumns::from_keys(self.columns.iter().map(StatusEntry::key));
            for entry in &self.columns {
                if let StatusEntry::Labelled { key, label } = entry {
                    columns.set_label(key, label);
                }
            }
            columns
        }
    }

    pub fn serialize<S: Serializer>(value: &BTreeMap<PathBuf, BoardColumns>, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_seq(value.iter().map(|(file, columns)| BoardFile::new(file, columns)))
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<BTreeMap<PathBuf, BoardColumns>, D::Error> {
        Ok(Vec::<BoardFile>::deserialize(deserializer)?
            .into_iter()
            .map(|board| {
                let columns = board.columns();
                (board.file, columns)
            })
            .collect())
    }
}
