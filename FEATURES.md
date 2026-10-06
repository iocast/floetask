# floetask: feature specification

A desktop todo.txt manager written in Rust (edition 2024) with an [iced](https://iced.rs) GUI.

- Target: iced **0.14** (current stable), desktop only (Linux, Windows, macOS).
- Status: specification only. No code has been written.

Priority tags used throughout:

- **P0**: needed for a usable first release (core todo.txt editing).
- **P1**: needed for a complete feature set.
- **P2**: polish or platform extras; can follow later.
- **Out**: deliberately left out.

The specification is split into one file per topic in [features/](features/). Section numbers (§4.3) stay the same across files.

| § | Topic | Covers |
|---|---|---|
| 1 | [todo.txt data model and parsing (P0)](features/01-data-model.md) | todo.txt syntax, the parsed todo, round-trip rules, multi-line todos, `status:` |
| 2 | [Files (P0 / P1)](features/02-files.md) | opening and creating files, multiple files, done files, safe writes, file watching |
| 3 | [Todo list view (P0)](features/03-list-view.md) | rows, grouping, counts, Markdown, empty states, layout options, the status board |
| 4 | [Dates, deferral, recurrence (P0 / P1)](features/04-dates.md) | completion, creation dates, recurrence, thresholds, natural and human-friendly dates |
| 5 | [Add / edit dialog (P0)](features/05-editor.md) | the add/edit dialog: autocomplete, pickers, notes, bulk creation |
| 6 | [Sidebar drawer: attributes, filters, sorting (P1)](features/06-drawer.md) | the drawer: attributes, filter toggles, sorting |
| 7 | [Search (P1)](features/07-search.md) | search bar, filter expressions, saved filters |
| 8 | [Archiving (P1)](features/08-archiving.md) | moving completed todos to the done file |
| 9 | [Notifications, badges, tray (P1 / P2)](features/09-notifications.md) | due-date notifications, badges, tray |
| 10 | [Settings (P0 / P1)](features/10-settings.md) | the settings dialog and the config file |
| 11 | [Theming and colours (P1)](features/11-theming.md) | themes and the colour file |
| 12 | [Navigation, menus, keyboard (P0 / P1)](features/12-navigation.md) | navigation, menus, keyboard shortcuts, clipboard and undo |
| 13 | [Language support (P1)](features/13-i18n.md) | translated interface, language choice, translation files, dates and fonts per language |
| 14 | [Platform and distribution (P2)](features/14-platform.md) | platforms and packaging |
| 15 | [Out of scope](features/15-out-of-scope.md) | what floetask deliberately leaves out |
| 16 | [Suggested crate layout (for later, not yet created)](features/16-crate-layout.md) | the crate layout |
| 17 | [Suggested milestones](features/17-milestones.md) | milestones |
| 18 | [Calendar view (P1)](features/18-calendar.md) | day, week and month calendars of todos by due date |

## todo.txt extensions

floetask supports two extensions beyond the official todo.txt format. Each has its own requirements document:

- [`note:`](features/note-extension.md): links a todo to a Markdown notes file.
- [`status:`](features/status-extension.md): gives an open todo a workflow state.

New requirement documents go in [features/](features/) and get a line here.
