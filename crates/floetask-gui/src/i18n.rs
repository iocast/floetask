//! User-facing strings. Every label goes through [`tr`], so adding a
//! language means adding a table here; English is the only one so far.

/// Languages with a translation table.
pub const LANGUAGES: [&str; 2] = ["system", "en"];

/// Looks up a string; unknown keys are returned as they are.
pub fn tr(key: &str) -> &str {
    english(key).unwrap_or(key)
}

/// Replaces `{}` placeholders in order.
pub fn trf(key: &str, args: &[&dyn std::fmt::Display]) -> String {
    let mut result = String::new();
    let mut args = args.iter();
    let mut pieces = tr(key).split("{}").peekable();
    while let Some(piece) = pieces.next() {
        result.push_str(piece);
        if pieces.peek().is_some()
            && let Some(arg) = args.next()
        {
            result.push_str(&arg.to_string());
        }
    }
    result
}

fn english(key: &str) -> Option<&'static str> {
    Some(match key {
        // Navigation and files
        "add_todo" => "Add todo",
        "toggle_drawer" => "Filters and sorting",
        "archive" => "Archive completed todos",
        "open_file" => "Open todo file",
        "create_file" => "Create todo file",
        "settings" => "Settings",
        "search" => "Search",
        "no_file_title" => "No todo file open",
        "no_file_body" => "Open an existing todo.txt or create a new one. You can also drop a file on the window.",
        "no_todos_title" => "This file has no todos yet",
        "no_visible_title" => "No todos match the current filters",
        "reset_filters" => "Reset filters",
        "file_dialog_title" => "todo.txt files",
        "reveal_todo_file" => "Show in file manager",
        "remove_file" => "Remove from list",
        "remove_file_confirm" => "Remove {} from the list? The file itself is not deleted.",
        "file_load_failed" => "Could not open {}",
        "done_file_needed" => "Choose or create a done file for archiving first.",
        "archived" => "Archived {} todos",
        "archived_one" => "Todo archived",
        // Counts
        "counts" => "{} of {} todos, {} completed",
        // List
        "created_on" => "Created {}",
        "completed_on" => "Completed {}",
        "no_value" => "No {}",
        "copy" => "Copy",
        "copied" => "Copied to clipboard",
        "delete" => "Delete",
        "delete_confirm" => "Delete this todo?",
        "edit" => "Edit",
        "open_link" => "Open link",
        // Editor
        "new_todo" => "New todo",
        "edit_todo" => "Edit todo",
        "editor_placeholder" => "(A) Call mom +family @phone due:tomorrow",
        "add" => "Add",
        "add_count" => "Add ({})",
        "save" => "Save",
        "cancel" => "Cancel",
        "close" => "Close",
        "confirm" => "Confirm",
        "empty_todo" => "Type something first",
        "priority" => "Priority",
        "due" => "Due",
        "t" => "Threshold",
        "rec" => "Repeat",
        "pm" => "Pomodoros",
        "set" => "Set",
        "clear" => "Clear",
        "today" => "Today",
        "strict" => "Strict",
        // Attributes
        "projects" => "Projects",
        "contexts" => "Contexts",
        "created" => "Created",
        "completed" => "Completed",
        "attributes" => "Attributes",
        "filters" => "Filters",
        "sorting" => "Sorting",
        "hide_category" => "Hide todos with any value",
        "show_category" => "Show todos with this attribute again",
        "rename" => "Rename",
        "rename_title" => "Rename {} everywhere in this file",
        "remove_value" => "Remove",
        "remove_value_confirm" => "Remove {} from every todo in this file?",
        "renamed" => "Updated {} todos",
        "include_hint" => {
            "Click to include, Alt+click to exclude. Right-click a project or context to rename or remove it."
        }
        // View toggles
        "show_completed" => "Show completed todos",
        "show_hidden" => "Show hidden todos (h:1)",
        "show_hidden_attributes" => "Show attributes of hidden todos",
        "show_future_threshold" => "Show todos with a future threshold date",
        "show_future_due" => "Show todos with a future due date",
        "file_order" => "File order (no grouping)",
        "completed_last" => "Sort completed todos last",
        "move_up" => "Move up",
        "move_down" => "Move down",
        "invert" => "Invert",
        // Search
        "search_placeholder" => "Search, or filter: +project @context due: < today",
        "saved_filters" => "Saved filters",
        "save_filter" => "Save current search",
        "filter_name" => "Name",
        "no_saved_filters" => "No saved filters yet",
        "delete_filter_confirm" => "Delete the saved filter {}?",
        "suppress_notifications" => "Mute notifications for matching todos",
        // Settings
        "settings_title" => "Settings",
        "append_creation_date" => "Add creation date to new todos",
        "convert_relative_dates" => "Convert relative dates (due:tomorrow) to dates",
        "human_friendly_dates" => "Show dates as today, next week, ...",
        "safe_writes" => "Safe writes (temporary file and backup)",
        "bulk_creation" => "Create one todo per line",
        "disable_animations" => "Disable animations",
        "compact" => "Compact list",
        "new" => "New",
        "toggle_files" => "Show or hide files",
        "files" => "Files",
        "file_actions" => "File actions",
        "set_archive_file" => "Set archive file",
        "open_archive_file" => "Open archive file",
        "close_file" => "Close",
        "toggle_theme" => "Switch light / dark",
        "archive_one" => "Move to done file",
        "t_short" => "starts",
        "groups" => "groups",
        "details" => "Details",
        "editor_hint" => "Ctrl+Enter to save, Esc to close",
        "no_todos_body" => "Add your first todo with Ctrl+N.",
        "previous_month" => "Previous month",
        "next_month" => "Next month",
        "section_todos" => "Todos",
        "section_notifications" => "Notifications",
        "section_appearance" => "Appearance",
        "notifications" => "Due-date notifications",
        "notification_threshold" => "Notify {} days ahead",
        "zoom" => "Zoom {}%",
        "theme" => "Theme",
        "theme_system" => "System",
        "theme_light" => "Light",
        "theme_dark" => "Dark",
        "week_start" => "Week starts on",
        "monday" => "Monday",
        "saturday" => "Saturday",
        "sunday" => "Sunday",
        "language" => "Language",
        "language_system" => "System",
        "language_en" => "English",
        "config_location" => "Settings file: {}",
        "settings_save_failed" => "Could not save settings",
        // Notifications
        "due_today" => "Due today",
        "due_tomorrow" => "Due tomorrow",
        "due_in_days" => "Due in {} days",
        // Human-friendly date buckets
        "overdue" => "overdue",
        "elapsed" => "elapsed",
        "last week" => "last week",
        "tomorrow" => "tomorrow",
        "this week" => "this week",
        "next week" => "next week",
        "this month" => "this month",
        "next month" => "next month",
        // Recurrence units
        "unit_d" => "days",
        "unit_b" => "business days",
        "unit_w" => "weeks",
        "unit_m" => "months",
        "unit_y" => "years",
        // Weekdays (calendar header)
        "mon" => "Mo",
        "tue" => "Tu",
        "wed" => "We",
        "thu" => "Th",
        "fri" => "Fr",
        "sat" => "Sa",
        "sun" => "Su",
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_placeholders() {
        assert_eq!(trf("counts", &[&1, &2, &3]), "1 of 2 todos, 3 completed");
        assert_eq!(tr("missing key"), "missing key");
    }
}
