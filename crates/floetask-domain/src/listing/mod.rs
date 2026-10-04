//! How a todo file becomes the list on screen: visibility toggles,
//! attribute filters, search, sorting and grouping, plus the attribute
//! counts shown in the drawer.

mod attribute;
mod build;
mod drawer;
mod sorting;
mod visibility;

use crate::date::{Date, WeekStart};

pub use attribute::Attribute;
pub use build::{Counts, Group, ListedTodo, TodoListing, build_listing};
pub use drawer::{AttributeSummary, AttributeValue, known_names, summarize_attributes};
pub use sorting::{SortCriterion, Sorting};
pub use visibility::{AttributeFilter, ViewOptions};

/// What "today" is and how dates are shown, needed to compare and label
/// dates consistently everywhere in a listing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DateContext {
    pub today: Date,
    pub week_start: WeekStart,
    /// Show and filter dates as buckets (`today`, `next week`) instead of ISO.
    pub human_friendly: bool,
}
