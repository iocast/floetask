# Changelog

All notable changes to this project are documented in this file.
Format: Keep a Changelog (https://keepachangelog.com), versioning: Semantic Versioning (https://semver.org).


## [Unreleased]

### Fixed
- The overview's file switches in the *Filters* tab show each file's folder below its name, as announced for 0.7.1; that release still showed the full path in brackets

## [0.7.1] - 2026-11-13

### Changed
- The overview's file switches in the *Filters* tab show each file's folder below its name in smaller text, as the file drawer does, instead of the full path in brackets

## [0.7.0] - 2026-11-12

### Added
- Share a todo file between people or devices through a folder that OneDrive, Dropbox, Syncthing or a network share syncs: *Share…* in a file's ⋮ menu chooses a folder (the shared copy is named like the file with `-share`) or an existing `.txt` file; *Join shared file…* in the file drawer links your own copy to one someone shared
- Everyone keeps their own local file: floetask merges changes from both sides line by line, including the notes the todos link to, after each save, when a file changes and every 30 seconds (*Settings → Sync*, `[sync] interval` in `config.toml`)
- An icon next to a shared file shows whether it is in sync, offline, missing its shared copy or has conflicts; lines changed differently on both sides are settled in a dialog with *Keep mine*, *Use theirs* or *Keep both*
- Conflict copies that OneDrive, Dropbox, Nextcloud or Syncthing create next to a shared copy are merged in and removed
- Shared files are listed in `sync.toml` in the state directory, with their base copies in `sync/`; `floetask --paths` shows where
- *Complete* in the edit dialog saves your edits and completes the todo in one step (a recurring todo adds its next occurrence); on a completed todo it reads *Reopen* and un-completes it
- *Delete* in the edit dialog deletes the todo after asking, as the row menu does (including whether its note goes too)
- *Work week* mode in the calendar: Monday to Friday as five day columns, next to *Day*, *Week* and *Month*
- Choose which files the overview shows: the drawer's *Filters* tab lists every file with a switch while the overview is open; at least one stays on, and the choice is kept in `state.toml` (`overview_hidden`)

### Changed
- Editing a todo no longer fails with "line changed" when lines above it were added or removed outside floetask, as long as no other todo has the same text
- A change saved while another app or a sync wrote the same file is applied on top of that change instead of overwriting it; archiving or moving a todo to another file then stops and leaves the todo in both files rather than losing anything
- Deleting a todo whose note no other todo uses asks whether to delete the note too: *Delete todo only* keeps the file as before, *Delete todo and note* removes it
- *Join shared file…* opens a step-by-step wizard instead of two file pickers in a row: choose the shared copy, then your own copy (suggested next to your open file, or a new or existing file), then check both and join; it says up front whether your copy will be created or merged
- The edit dialog shows the details in a side panel next to the text and the note, and can be resized by its edges or corner; its size is kept in `state.toml` (`edit_dialog`)
- The text field no longer shows `status:`, since the status picker does; *Delete* is a quiet text button and *×* clears the text
- In the overview, renaming or removing a project or context changes only the files the overview shows

## [0.7.0-rc.4] - 2026-11-12

### Added
- *Delete* in the edit dialog deletes the todo after asking, as the row menu does (including whether its note goes too)

## [0.7.0-rc.3] - 2026-11-11

### Changed
- Deleting a todo whose note no other todo uses asks whether to delete the note too: *Delete todo only* keeps the file as before, *Delete todo and note* removes it
- *Join shared file…* opens a step-by-step wizard instead of two file pickers in a row: choose the shared copy, then your own copy (suggested next to your open file, or a new or existing file), then check both and join; it says up front whether your copy will be created or merged

## [0.7.0-rc.2] - 2026-11-11

### Added
- *Complete* in the edit dialog saves your edits and completes the todo in one step (a recurring todo adds its next occurrence); on a completed todo it reads *Reopen* and un-completes it

## [0.7.0-rc.1] - 2026-11-07

### Added
- Share a todo file between people or devices through a folder that OneDrive, Dropbox, Syncthing or a network share syncs: *Share…* in a file's ⋮ menu chooses a folder (the shared copy is named like the file with `-share`) or an existing `.txt` file; *Join shared file…* in the file drawer links your own copy to one someone shared
- Everyone keeps their own local file: floetask merges changes from both sides line by line, including the notes the todos link to, after each save, when a file changes and every 30 seconds (*Settings → Sync*, `[sync] interval` in `config.toml`)
- An icon next to a shared file shows whether it is in sync, offline, missing its shared copy or has conflicts; lines changed differently on both sides are settled in a dialog with *Keep mine*, *Use theirs* or *Keep both*
- Conflict copies that OneDrive, Dropbox, Nextcloud or Syncthing create next to a shared copy are merged in and removed
- Shared files are listed in `sync.toml` in the state directory, with their base copies in `sync/`; `floetask --paths` shows where

### Changed
- Editing a todo no longer fails with "line changed" when lines above it were added or removed outside floetask, as long as no other todo has the same text
- A change saved while another app or a sync wrote the same file is applied on top of that change instead of overwriting it; archiving or moving a todo to another file then stops and leaves the todo in both files rather than losing anything

## [0.6.3] - 2026-11-07

### Removed
- The *Disable animations* setting, which had no effect because floetask has no animations; an existing `disable_animations` key in `config.toml` is ignored

## [0.6.2] - 2026-11-07

### Added
- The Windows MSI installer asks whether to create a desktop shortcut, like the setup `.exe` already does on its last page; a silent MSI install skips the shortcut with `DESKTOPSHORTCUT=""`
- Right-click in a note for a menu: with text selected it offers cut, copy, paste, bold, italic, strikethrough, code and *Link…*; inside a table it inserts or deletes rows and columns, deletes the table and sets the column alignment
- *Link…* opens a dialog in the middle of the note to set the text and address of a link, or remove it

### Removed
- The *Disable animations* setting, which had no effect because floetask has no animations; an existing `disable_animations` key in `config.toml` is ignored

### Fixed
- Updating from within the app no longer brings back a desktop shortcut you chose not to create or deleted
- Double-clicking a word in a note selects the whole word even if the mouse moves slightly; dragging after a double or triple click extends the selection by whole words or paragraphs

## [0.6.1] - 2026-11-07

### Added
- Week numbers in the calendar: the month view shows each week's ISO number beside it and the week view names its week in the header; turn them on with *Week numbers* in Settings → Dates (`week_numbers` in `config.toml`)
- Support floetask: Settings → About shows a card with links to sponsor it on GitHub Sponsors or Ko-fi

### Changed
- The Settings sidebar groups its sections under the captions General, Interface and App

### Fixed
- The list keeps its scroll position when the file drawer or the filter drawer is shown or hidden

## [0.6.0] - 2026-11-04

### Added
- Move a todo to another file: drag it from the list, board or calendar onto a file in the file drawer, or right-click it and pick *Move to file…*; a note in its notes folder moves along, a note linked by its path stays where it is
- A dragged todo, in the list, on the board or in the calendar, follows the pointer as a small card

## [0.5.2] - 2026-11-03

### Added
- Double-click a day in the calendar, or right-click it and pick *New todo*, to add a todo due that day
- Double-click a board column, or right-click it and pick *New todo*, to add a todo with that column's status and, on a board grouped by project, context or priority, the group's value
- Clicking or right-clicking a calendar day or a board column outlines it, so you can see which one a new todo goes to

### Removed
- The zoom slider in Settings → Appearance; the window always shows at 100 %, and `zoom_percent` in `config.toml` is ignored

## [0.5.1] - 2026-10-21

### Changed
- An available update shows as an *Update* button in the title bar, which opens Settings → About to install it, instead of a toast on start

## [0.5.0] - 2026-10-21

### Added
- Overview of all files: with two or more files, an *Overview* entry tops the file drawer and shows every file's todos at once, grouped by file in the list, one board per file (with its own columns) and one merged calendar; actions write to the todo's own file, the add dialog lets you pick the file for a new todo, and drawer counts and project/context renames span every file

### Fixed
- Dragging the title bar moves the window while a dialog is open; pressing it used to close the dialog instead

## [0.4.1] - 2026-10-19

### Added
- Right-click a card on the board or in the calendar to copy it, move it to the done file or delete it, as in the list; the menu opens at the cursor and the card is outlined while it is open

## [0.4.0] - 2026-10-17

### Added
- Note window: clicking a todo's note opens it in a separate floetask window with the Markdown editor instead of the default app; one window is reused for every note, saves as you type (and on `Ctrl+S` or closing), and can still open the note in the default app; it opens where you left it
- The add/edit dialog can open its note in the note window, taking along unsaved text
- Link a Markdown file from disk to a todo instead of creating a note: move it into the notes folder, or keep it where it is and link it by its path (`note:` accepts percent-encoded absolute paths, e.g. `note:C%3A/Users/me/My%20Notes/plan.md`); unlink a note again, the file is kept
- Release channel in Settings → About: *Stable* offers releases only, *Preview* also offers release candidates, to try the next version early
- Roll back in Settings → About: install one of the last ten earlier versions of your channel over the current one, e.g. to leave a preview; todo files and settings are kept
- The interface is translated into German, French, Spanish and Italian; pick the language in Settings → Appearance → Language, it applies without a restart, and *System* follows the OS language (e.g. `de-CH` uses German, unknown languages fall back to English); error messages are translated too, details reported by the OS stay as they are
- Calendar titles, month and weekday names follow the chosen language; on first start the week begins on the OS region's first day of the week
- Custom statuses have display names: type "In review" in settings or when adding a board column and floetask derives the key (`status:in-review`); in `config.toml`, `[statuses] order` and `[[boards]] columns` accept `{ key = "in-review", label = "In review" }` next to plain keys, which keep working
- Sort the file drawer by name or folder with the sort button next to *Files*, or drag files into your own order; an archive file always stays under (and moves with) its todo file, and `Ctrl+1`…`Ctrl+9` follow the order shown
- The notes editor's toolbar, link bar and status bar are translated too and switch with the interface language without a restart

### Changed
- The notes field in the add/edit dialog is now a WYSIWYG Markdown editor with a formatting toolbar and a Source mode, in floetask's colours; edited notes are saved in canonical Markdown
- Release candidates are published with an updater manifest, and the app lists releases through the GitHub API

## [0.4.0-rc.3] - 2026-10-13

### Added
- The interface is translated into German, French, Spanish and Italian; pick the language in Settings → Appearance → Language, it applies without a restart, and *System* follows the OS language (e.g. `de-CH` uses German, unknown languages fall back to English); error messages are translated too, details reported by the OS stay as they are
- Calendar titles, month and weekday names follow the chosen language; on first start the week begins on the OS region's first day of the week
- Custom statuses have display names: type "In review" in settings or when adding a board column and floetask derives the key (`status:in-review`); in `config.toml`, `[statuses] order` and `[[boards]] columns` accept `{ key = "in-review", label = "In review" }` next to plain keys, which keep working

## [0.4.0-rc.2] - 2026-10-12

### Added
- Release channel in Settings → About: *Stable* offers releases only, *Preview* also offers release candidates, to try the next version early
- Roll back in Settings → About: install one of the last ten earlier versions of your channel over the current one, e.g. to leave a preview; todo files and settings are kept

### Changed
- Release candidates are published with an updater manifest, and the app lists releases through the GitHub API

## [0.4.0-rc.1] - 2026-10-12

### Added
- Note window: clicking a todo's note opens it in a separate floetask window with the Markdown editor instead of the default app; one window is reused for every note, saves as you type (and on `Ctrl+S` or closing), and can still open the note in the default app; it opens where you left it
- The add/edit dialog can open its note in the note window, taking along unsaved text
- Link a Markdown file from disk to a todo instead of creating a note: move it into the notes folder, or keep it where it is and link it by its path (`note:` accepts percent-encoded absolute paths, e.g. `note:C%3A/Users/me/My%20Notes/plan.md`); unlink a note again, the file is kept

### Changed
- The notes field in the add/edit dialog is now a WYSIWYG Markdown editor with a formatting toolbar and a Source mode, in floetask's colours; edited notes are saved in canonical Markdown

## [0.3.1] - 2026-10-07

### Fixed
- Windows: notifications now show floetask with its name and icon instead of Windows PowerShell

## [0.3.0] - 2026-10-07

### Added
- Calendar view with day, week and month modes (`Ctrl+Alt+C`): todos by due date, filtered like the list; drag a todo to another day to change its due date
- Calendar header shows "N without a due date", which opens a panel of open undated todos; drag one onto a day or use "Set due date" to plan it
- Month cells show each todo on one line, with the full text in a tooltip on hover

### Changed
- Below 760px window width, side panels hide automatically in list, board and calendar (filter drawer first, then file drawer) and return when the window grows; opening a hidden panel shows it over the main view until you click beside it or press `Escape`
- The search field in the title bar shrinks when it cannot stay centred, so it never covers the view and window buttons

## [0.2.2] - 2026-10-06

### Added
- Saved search "overdue" (`due: < today+1d`) out of the box, as long as `filters.toml` does not exist yet; deleting it sticks

### Changed
- Autocomplete suggestions in the todo dialog show as pills in the project or context colour instead of a list; keyboard navigation is unchanged

### Fixed
- The todo dialog no longer crashes when the cursor sits right before a `+` or `@`

## [0.2.1] - 2026-10-06

### Added
- Desktop todo.txt manager for Linux, Windows and macOS with an iced GUI; edits change only the tokens they touch, so files round-trip exactly
- Several todo files at once in a file drawer (`Ctrl+Alt+H`), each with its own archive (done) file; files are written safely and reloaded when changed on disk
- Add/edit dialog with `+project` and `@context` autocomplete and pickers for priority, due and threshold dates, recurrence (`rec:`) and pomodoros
- Todo cards with priority accents, contexts shown in place in the text as pills that filter on click, and labelled due and threshold dates
- Filter drawer (`Ctrl+B`) with attributes, filter toggles and sorting; list groups collapse from their header
- Search bar with filter expressions such as `+work and due: < today+3d`, `(A) or pri >= C` and `/regex/`, plus saved searches in `filters.toml`
- Natural and human-friendly dates, recurrence and thresholds
- Archiving of completed todos (`Ctrl+Alt+A`) into the file's archive file
- `status:` extension for workflow statuses (doing, todo, waiting, someday or your own), with a status picker, grouping and sorting by status, and `someday` hidden unless asked for; completing a todo removes its status
- Status board (`Ctrl+Alt+B`) with drag and drop between status columns and a Done column; columns are set per file and kept as `[[boards]]` in `config.toml`; grouped lists get one collapsible board per group, with a switch to turn grouping off
- `note:` extension linking a todo to a Markdown note in `<file>-notes/` next to the todo file; notes are written in the todo dialog, created automatically, and moved along when archiving
- Due-date notifications through the OS, mutable per saved search
- Settings dialog with sections for todos, dates, statuses, appearance, notifications and files; custom statuses go in `[statuses]` in `config.toml`
- Borderless window with its own title bar, light and dark themes following the system (`Ctrl+Alt+D` toggles), and colour overrides in `colors.toml`
- Keyboard shortcuts for every main action (see the README)
- Command line: `floetask [TODO_FILE]`, `--config FILE` to use another config file, `--paths` to print where floetask keeps its files
- Files follow the XDG Base Directory layout on every OS: settings, colours and saved searches in `~/.config/floetask/`, app state in `~/.local/state/floetask/`
- Settings → About with the version, a check for new releases on start (`check_for_updates` in `config.toml`, on by default) or on request, and one-click install of signed updates
- Installers for every platform attached to each release: setup `.exe` and `.msi` on Windows, `.app` and `.dmg` on macOS, `.deb` and AppImage on Linux; on Windows the app opens without a console window

[unreleased]: https://github.com/iocast/floetask/compare/v0.7.1...HEAD
[0.7.1]: https://github.com/iocast/floetask/compare/v0.7.0...v0.7.1
[0.7.0]: https://github.com/iocast/floetask/compare/v0.6.3...v0.7.0
[0.7.0-rc.4]: https://github.com/iocast/floetask/compare/v0.7.0-rc.3...v0.7.0-rc.4
[0.7.0-rc.3]: https://github.com/iocast/floetask/compare/v0.7.0-rc.2...v0.7.0-rc.3
[0.7.0-rc.2]: https://github.com/iocast/floetask/compare/v0.7.0-rc.1...v0.7.0-rc.2
[0.7.0-rc.1]: https://github.com/iocast/floetask/compare/v0.6.3...v0.7.0-rc.1
[0.6.3]: https://github.com/iocast/floetask/compare/v0.6.2...v0.6.3
[0.6.2]: https://github.com/iocast/floetask/compare/v0.6.1...v0.6.2
[0.6.1]: https://github.com/iocast/floetask/compare/v0.6.0...v0.6.1
[0.6.0]: https://github.com/iocast/floetask/compare/v0.5.2...v0.6.0
[0.5.2]: https://github.com/iocast/floetask/compare/v0.5.1...v0.5.2
[0.5.1]: https://github.com/iocast/floetask/compare/v0.5.0...v0.5.1
[0.5.0]: https://github.com/iocast/floetask/compare/v0.4.1...v0.5.0
[0.4.1]: https://github.com/iocast/floetask/compare/v0.4.0...v0.4.1
[0.4.0]: https://github.com/iocast/floetask/compare/v0.3.1...v0.4.0
[0.4.0-rc.3]: https://github.com/iocast/floetask/compare/v0.4.0-rc.2...v0.4.0-rc.3
[0.4.0-rc.2]: https://github.com/iocast/floetask/compare/v0.4.0-rc.1...v0.4.0-rc.2
[0.4.0-rc.1]: https://github.com/iocast/floetask/compare/v0.3.1...v0.4.0-rc.1
[0.3.1]: https://github.com/iocast/floetask/compare/v0.3.0...v0.3.1
[0.3.0]: https://github.com/iocast/floetask/compare/v0.2.2...v0.3.0
[0.2.2]: https://github.com/iocast/floetask/compare/v0.2.1...v0.2.2
[0.2.1]: https://github.com/iocast/floetask/releases/tag/v0.2.1
[0.2.0]: https://github.com/iocast/floetask/releases/tag/v0.2.0
