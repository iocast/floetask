# Changelog

All notable changes to this project are documented in this file.
Format: Keep a Changelog (https://keepachangelog.com), versioning: Semantic Versioning (https://semver.org).

## [Unreleased]

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

### Changed

### Fixed
