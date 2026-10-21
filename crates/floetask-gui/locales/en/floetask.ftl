# floetask: English, the source language.
#
# Every other language copies these ids and translates the values. A new
# string is added here first. Placeholders are named ({ $count }), plurals
# use selectors; see CONTRIBUTING.md.

## Navigation and files

add_todo = Add todo
toggle_drawer = Filters and sorting
archive = Archive completed todos
open_file = Open todo file
create_file = Create todo file
settings = Settings
search = Search
no_file_title = No todo file open
no_file_body = Open an existing todo.txt or create a new one. You can also drop a file on the window.
no_todos_title = This file has no todos yet
no_todos_body = Add your first todo with Ctrl+N.
no_visible_title = No todos match the current filters
reset_filters = Reset filters
file_dialog_title = todo.txt files
reveal_todo_file = Show in file manager
remove_file = Remove from list
remove_file_confirm = Remove { $file } from the list? The file itself is not deleted.
file_load_failed = Could not open { $file }
done_file_needed = Choose or create a done file for archiving first.
archived =
    { $count ->
        [one] Archived 1 todo
       *[other] Archived { $count } todos
    }
archived_one = Todo archived
new = New
toggle_files = Show or hide files
files = Files
file_actions = File actions
set_archive_file = Set archive file
open_archive_file = Open archive file
close_file = Close
sort_files = Sort files
file_sort_custom = Custom order
file_sort_custom_hint = Drag files to arrange them
file_sort_name = Name
file_sort_folder = Folder
overview = Overview
overview_hint = All files, grouped by file
toggle_theme = Switch light / dark

## Counts

counts =
    { $visible } of { $total ->
        [one] { $total } todo
       *[other] { $total } todos
    }, { $completed } completed
groups = groups

## List

created_on = Created { $date }
completed_on = Completed { $date }
no_value = No { $attribute }
copy = Copy
copied = Copied to clipboard
delete = Delete
delete_confirm = Delete this todo?
edit = Edit
open_link = Open link
archive_one = Move to done file
due_label = due
threshold_label = threshold
details = Details

## Editor

new_todo = New todo
edit_todo = Edit todo
editor_file = In { $file }
editor_add_to = Add to
editor_placeholder = (A) Call mom +family @phone due:tomorrow
editor_hint = Ctrl+Enter to save, Esc to close
add = Add
add_count = Add ({ $count })
save = Save
cancel = Cancel
close = Close
confirm = Confirm
empty_todo = Type something first
priority = Priority
due = Due
t = Threshold
rec = Repeat
pm = Pomodoros
status = Status
status_todo = To do
status_doing = Doing
status_waiting = Waiting
status_someday = Someday
status_done = Done
set = Set
clear = Clear
today = Today
strict = Strict
note = Note
notes = Notes
previous_month = Previous month
next_month = Next month

## Notes

notes_placeholder = Notes in Markdown, saved to a file next to your todo file
note_saved_to = Saved to { $path }
open_note = Open note
link_note_file = Link a Markdown file instead of creating one
link_note_dialog_title = Link a Markdown file
markdown_files = Markdown and text
note_outside_folder = { $file } is not in { $folder }, where this list keeps its notes.
note_move_into = Move it to { $folder }
note_keep_in_place = Keep it where it is
unlink_note = Unlink the note; its file is kept
open_note_window = Open in a separate window
note_in_window = This note is open in its own window.
show_note_window = Show window
note_saved = Saved
note_unsaved = Unsaved changes
open_in_default_app = Open in the default app
note_missing = { $note } does not exist yet; open it to create it
note_invalid = { $note } is not a valid note name
note_orphaned = No todo links to { $note } any more; the file was kept

## Status board

show_board = Show as board (Ctrl+Alt+B)
show_list = Show as list (Ctrl+Alt+B)
board_columns = Columns
board_group = Group
board_group_by = Group by { $attribute }
board_group_hint = One board per group. Off keeps the sort order on a single board.
board_other = Other statuses
board_columns_title = Board columns for { $file }
board_columns_hint = Each column is a status; Done holds completed todos. Drag a card to another column to change its status.
status_placeholder = Name, e.g. In review
remove_column = Remove column
reset_columns = Reset

## Calendar view

show_calendar = Calendar (Ctrl+Alt+C)
calendar_day = Day
calendar_week = Week
calendar_month = Month
calendar_today = Today
calendar_previous = Previous
calendar_next = Next
calendar_more = +{ $count } more
calendar_undated = { $count } without a due date
calendar_undated_title = Without due date
calendar_undated_hint = Drag a todo onto a day, or set its date here.
calendar_set_due = Set due date
calendar_empty_day = Nothing due this day.

## Dates
# Calendar titles. $month and $weekday are the names below.

date_month_year = { $month } { $year }
date_long = { $weekday }, { $day } { $month } { $year }
date_range_in_month = { $first_day } – { $last_day } { $month } { $year }
date_range_in_year = { $first_day } { $first_month } – { $last_day } { $last_month } { $year }
date_range = { $first_day } { $first_month } { $first_year } – { $last_day } { $last_month } { $last_year }

month_1 = January
month_2 = February
month_3 = March
month_4 = April
month_5 = May
month_6 = June
month_7 = July
month_8 = August
month_9 = September
month_10 = October
month_11 = November
month_12 = December

month_short_1 = Jan
month_short_2 = Feb
month_short_3 = Mar
month_short_4 = Apr
month_short_5 = May
month_short_6 = Jun
month_short_7 = Jul
month_short_8 = Aug
month_short_9 = Sep
month_short_10 = Oct
month_short_11 = Nov
month_short_12 = Dec

weekday_mon = Monday
weekday_tue = Tuesday
weekday_wed = Wednesday
weekday_thu = Thursday
weekday_fri = Friday
weekday_sat = Saturday
weekday_sun = Sunday

# Next to the day number in the week view.
weekday_abbr_mon = Mon
weekday_abbr_tue = Tue
weekday_abbr_wed = Wed
weekday_abbr_thu = Thu
weekday_abbr_fri = Fri
weekday_abbr_sat = Sat
weekday_abbr_sun = Sun

# Column headers of month calendars.
mon = Mo
tue = Tu
wed = We
thu = Th
fri = Fr
sat = Sa
sun = Su

## Human-friendly dates

bucket_overdue = overdue
bucket_elapsed = elapsed
bucket_last_week = last week
bucket_today = today
bucket_tomorrow = tomorrow
bucket_this_week = this week
bucket_next_week = next week
bucket_this_month = this month
bucket_next_month = next month

## Recurrence units

unit_d = days
unit_b = business days
unit_w = weeks
unit_m = months
unit_y = years

## Attributes and the drawer

projects = Projects
contexts = Contexts
created = Created
completed = Completed
attributes = Attributes
filters = Filters
sorting = Sorting
hide_category = Hide todos with any value
show_category = Show todos with this attribute again
rename = Rename
rename_title = Rename { $value } everywhere in this file
rename_title_all = Rename { $value } everywhere in every file
remove_value = Remove
remove_value_confirm = Remove { $value } from every todo in this file?
remove_value_confirm_all = Remove { $value } from every todo in every file?
renamed =
    { $count ->
        [one] Updated 1 todo
       *[other] Updated { $count } todos
    }
include_hint = Click to include, Alt+click to exclude. Right-click a project or context to rename or remove it.
show_completed = Show completed todos
show_hidden = Show hidden todos (h:1)
show_hidden_attributes = Show attributes of hidden todos
show_hidden_statuses = Show { $statuses } todos
show_future_threshold = Show todos with a future threshold date
show_future_due = Show todos with a future due date
file_order = File order (no grouping)
completed_last = Sort completed todos last
move_up = Move up
move_down = Move down
invert = Invert

## Search

search_placeholder = Search, or filter: +project @context due: < today
saved_filters = Saved filters
save_filter = Save current search
filter_name = Name
no_saved_filters = No saved filters yet
delete_filter_confirm = Delete the saved filter { $name }?
suppress_notifications = Mute notifications for matching todos

## Settings

settings_title = Settings
section_todos = Todos
section_dates = Dates
section_statuses = Statuses
section_appearance = Appearance
section_notifications = Notifications
section_files = Files
section_about = About
append_creation_date = Add creation date to new todos
append_creation_date_desc = New todos start with today's date, as todo.txt allows.
convert_relative_dates = Convert relative dates (due:tomorrow) to dates
convert_relative_dates_desc = Words like tomorrow or next friday in due: and t: are saved as real dates.
human_friendly_dates = Show dates as today, next week, ...
human_friendly_dates_desc = Dates read as today, tomorrow or next week instead of 2026-10-05.
safe_writes = Safe writes (temporary file and backup)
safe_writes_desc = Writes to a temporary file first and keeps a backup, so a crash cannot corrupt your todo file.
bulk_creation = Create one todo per line
bulk_creation_desc = In the add dialog, every line becomes its own todo.
disable_animations = Disable animations
disable_animations_desc = Turns off motion; floetask has no animations yet.
compact = Compact mode
compact_desc = Tighter rows and cards, so more todos fit on screen.
month_overflow = Full month cells
month_overflow_desc = When a calendar month cell has more todos than fit: show +N more, or scroll.
month_overflow_more = Show +N more
month_overflow_scroll = Scroll
week_start = Week starts on
week_start_desc = First day of the week in calendars and for this week and next week.
monday = Monday
saturday = Saturday
sunday = Sunday
theme = Theme
theme_desc = Light, dark, or follow your system.
theme_system = System
theme_light = Light
theme_dark = Dark
language = Language
language_desc = Language of the interface. System follows your operating system.
language_system = System
zoom = Zoom { $percent }%
zoom_desc = Makes everything in the window larger or smaller.
notifications = Due-date notifications
notifications_desc = A system notification when a todo becomes due.
notification_threshold =
    { $days ->
        [0] Notify on the due date
        [one] Notify 1 day ahead
       *[other] Notify { $days } days ahead
    }
notification_threshold_desc = How many days before the due date you are notified.
config_file = Settings file
config_file_desc = Where these settings are saved. Board columns and statuses live here too.
config_location = Settings file: { $path }
settings_save_failed = Could not save settings
statuses_desc = Statuses for the status: tag, in the order the list sorts them. Hidden statuses stay out of the list until you ask for them. A board column can add a status for one file only.
status_todo_desc = Not started. The same as having no status: tag.
status_doing_desc = Actively being worked on.
status_waiting_desc = Blocked on someone or something else.
status_someday_desc = Parked, not committed to.
status_custom_desc = Your own status.
status_hidden = Hidden
remove_status = Remove status
add_status = Add status

## About and updates

about_tagline = A todo.txt manager for the desktop.
about_version = Version { $version }
check_for_updates = Check for updates on start
check_for_updates_desc = Look for a new release when floetask starts. Nothing is sent but the version check.
check_now = Check now
update_checking = Checking for updates…
update_up_to_date = floetask is up to date.
update_available = floetask { $version } is available.
update_available_toast = floetask { $version } is available. Install it from Settings → About.
update_install = Install and restart
update_installing = Downloading and installing floetask { $version }… floetask restarts when it is done.
update_preview_version = { $version } (preview)
update_preview_on_stable = You are running a preview. The stable channel updates it with the next release; to leave it now, roll back below.
update_channel = Release channel
update_channel_desc = Preview also offers release candidates: the next version a few days early, to try new features and report problems before everyone gets them. They are tested less. Switch back to Stable at any time.
update_channel_stable = Stable
update_channel_preview = Preview
rollback = Roll back
rollback_desc = Install an earlier version over this one, to leave a preview or to step back from a release with a problem. Your todo files and settings are kept.
rollback_pick = Earlier version
rollback_install = Install
rollback_check_first = Check for updates to list earlier versions.
rollback_none = No earlier version on this channel.
update_failed = Update failed: { $error }
updates_not_configured = This build cannot update itself. Releases from the project's release page can.

## Notifications

due_today = Due today
due_tomorrow = Due tomorrow
due_in_days = Due in { $days } days

## Errors
# $message is the operating system's own text and stays as it is.

error_io = Could not access { $path }: { $message }
error_stale_line = Line { $line } changed since it was read; reload and try again.
error_missing_line = Line { $line } does not exist.
error_note_empty = The note name is empty.
error_note_character = “{ $character }” is not allowed in a note name; use lowercase a-z, 0-9, “.”, “_” and “-”.
error_note_outside = A note name must stay inside the notes folder.
error_no_done_file = { $file } has no done file to archive into.
error_empty_todo = The todo is empty.
error_config = Invalid configuration: { $details }
