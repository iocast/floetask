//! todo.txt parsing and task logic for floetask.
//!
//! This crate has no GUI dependency so all todo logic can be unit-tested
//! on its own. See FEATURES.md for the planned scope.

/// Crate version, shown in the app's about box.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_is_set() {
        assert!(!VERSION.is_empty());
    }
}
