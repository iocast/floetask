use std::cmp::Ordering;

use super::Attribute;
use crate::todo::Todo;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SortCriterion {
    pub attribute: Attribute,
    pub descending: bool,
}

/// Sorting tab settings. The first criterion also groups the list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Sorting {
    pub criteria: Vec<SortCriterion>,
    /// Show todos in file order, without grouping.
    pub file_order: bool,
    /// In file order, move completed todos to the end.
    pub completed_last: bool,
}

impl Default for Sorting {
    fn default() -> Self {
        let order = [
            Attribute::Priority,
            Attribute::Projects,
            Attribute::Contexts,
            Attribute::Due,
            Attribute::Threshold,
            Attribute::Completed,
            Attribute::Created,
            Attribute::Recurrence,
            Attribute::Pomodoro,
        ];
        Self {
            criteria: order
                .into_iter()
                .map(|attribute| SortCriterion {
                    attribute,
                    descending: false,
                })
                .collect(),
            file_order: false,
            completed_last: false,
        }
    }
}

impl Sorting {
    pub fn grouping_attribute(&self) -> Option<Attribute> {
        (!self.file_order)
            .then(|| self.criteria.first().map(|c| c.attribute))
            .flatten()
    }

    /// Moves the criterion at `index` one step up (`-1`) or down (`1`).
    pub fn move_criterion(&mut self, index: usize, step: isize) {
        let target = index as isize + step;
        if index < self.criteria.len() && target >= 0 && (target as usize) < self.criteria.len() {
            self.criteria.swap(index, target as usize);
        }
    }

    pub fn toggle_direction(&mut self, index: usize) {
        if let Some(criterion) = self.criteria.get_mut(index) {
            criterion.descending = !criterion.descending;
        }
    }

    /// Compares by the given criteria. Missing values always sort last,
    /// whatever the direction.
    pub fn compare(criteria: &[SortCriterion], a: &Todo, b: &Todo) -> Ordering {
        for criterion in criteria {
            let attribute = criterion.attribute;
            let ordering = match (attribute.has_value(a), attribute.has_value(b)) {
                (true, false) => Ordering::Less,
                (false, true) => Ordering::Greater,
                (false, false) => Ordering::Equal,
                (true, true) => {
                    let ascending = attribute.compare(a, b).unwrap_or(Ordering::Equal);
                    if criterion.descending {
                        ascending.reverse()
                    } else {
                        ascending
                    }
                }
            };
            if ordering != Ordering::Equal {
                return ordering;
            }
        }
        Ordering::Equal
    }
}
