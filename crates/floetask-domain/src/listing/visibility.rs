use super::{Attribute, DateContext};
use crate::search::Query;
use crate::status::StatusSet;
use crate::todo::Todo;

/// A drawer filter on one attribute value.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AttributeFilter {
    pub attribute: Attribute,
    pub value: String,
    /// Exclude todos with this value instead of requiring it.
    pub exclude: bool,
}

/// View toggles and attribute filters (drawer "Filters" and "Attributes").
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ViewOptions {
    pub show_completed: bool,
    pub show_hidden: bool,
    pub show_future_threshold: bool,
    pub show_future_due: bool,
    /// Drawer counts include attributes of `h:1` todos.
    pub show_hidden_attributes: bool,
    /// Show todos whose status is hidden by default, such as `someday`.
    pub show_hidden_statuses: bool,
    /// Hide every todo that has any value for these attributes.
    pub hidden_categories: Vec<Attribute>,
    pub filters: Vec<AttributeFilter>,
}

impl Default for ViewOptions {
    fn default() -> Self {
        Self {
            show_completed: true,
            show_hidden: false,
            show_future_threshold: false,
            show_future_due: true,
            show_hidden_attributes: true,
            show_hidden_statuses: false,
            hidden_categories: Vec::new(),
            filters: Vec::new(),
        }
    }
}

impl ViewOptions {
    /// The toggles from the drawer's Filters tab: completion, `h:1`, and
    /// future threshold or due dates.
    pub fn passes_toggles(&self, todo: &Todo, dates: &DateContext) -> bool {
        (self.show_completed || !todo.is_complete())
            && (self.show_hidden || !todo.is_hidden())
            && (self.show_future_threshold || todo.threshold().is_none_or(|t| t <= dates.today))
            && (self.show_future_due || todo.due().is_none_or(|d| d <= dates.today))
    }

    /// Hides open todos whose status is hidden by default (`someday`) unless
    /// the user asks for them: with the toggle, a drawer filter on that
    /// status, or a search for `status:<value>`.
    pub fn passes_status(&self, todo: &Todo, statuses: &StatusSet, query: &Query) -> bool {
        let Some(status) = todo.status() else {
            return true;
        };
        !statuses.is_hidden(status)
            || self.show_hidden_statuses
            || self.filter_state(Attribute::Status, status) == Some(false)
            || query.mentions(&format!("status:{status}"))
    }

    /// Attribute filters and hidden categories. Include filters on one
    /// attribute need one match; exclude filters need no match.
    pub fn passes_filters(&self, todo: &Todo, dates: &DateContext) -> bool {
        if self.hidden_categories.iter().any(|attribute| attribute.has_value(todo)) {
            return false;
        }
        Attribute::ALL.into_iter().all(|attribute| {
            let filters: Vec<&AttributeFilter> = self.filters.iter().filter(|f| f.attribute == attribute).collect();
            if filters.is_empty() {
                return true;
            }
            let values = attribute.values(todo, dates);
            let has = |filter: &&AttributeFilter| values.contains(&filter.value);
            let includes: Vec<_> = filters.iter().filter(|f| !f.exclude).collect();
            let included = includes.is_empty() || includes.iter().any(|f| has(f));
            let excluded = filters.iter().filter(|f| f.exclude).any(has);
            included && !excluded
        })
    }

    pub fn filter_state(&self, attribute: Attribute, value: &str) -> Option<bool> {
        self.filters
            .iter()
            .find(|f| f.attribute == attribute && f.value == value)
            .map(|f| f.exclude)
    }

    /// Cycles a value's filter: none → include → none, or with `exclude`
    /// none → exclude → none.
    pub fn toggle_filter(&mut self, attribute: Attribute, value: &str, exclude: bool) {
        let existing = self
            .filters
            .iter()
            .position(|f| f.attribute == attribute && f.value == value);
        match existing {
            Some(index) if self.filters[index].exclude == exclude => {
                self.filters.remove(index);
            }
            Some(index) => self.filters[index].exclude = exclude,
            None => self.filters.push(AttributeFilter {
                attribute,
                value: value.to_owned(),
                exclude,
            }),
        }
    }

    pub fn toggle_hidden_category(&mut self, attribute: Attribute) {
        match self.hidden_categories.iter().position(|a| *a == attribute) {
            Some(index) => {
                self.hidden_categories.remove(index);
            }
            None => self.hidden_categories.push(attribute),
        }
    }

    pub fn has_active_filters(&self) -> bool {
        !self.filters.is_empty() || !self.hidden_categories.is_empty()
    }

    pub fn clear_filters(&mut self) {
        self.filters.clear();
        self.hidden_categories.clear();
    }
}
