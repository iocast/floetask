# 1. todo.txt data model and parsing (P0)

## 1.1 Supported syntax

| Element | Syntax | Notes |
|---|---|---|
| Completion | `x ` prefix | Followed by completion date then creation date. |
| Priority | `(A)`–`(Z)` at line start | A–C get distinct colours; D–Z share a neutral colour. |
| Creation date | `YYYY-MM-DD` after priority | Optional; appended automatically when the setting is on. |
| Completion date | `x YYYY-MM-DD` | Set automatically on completion. |
| Projects | `+name` | Any number, no spaces. |
| Contexts | `@name` | Any number, no spaces. |
| Due date | `due:YYYY-MM-DD` | |
| Threshold date | `t:YYYY-MM-DD` | Defers visibility (see [§4](04-dates.md#4-dates-deferral-recurrence-p0--p1)). |
| Recurrence | `rec:[+][N](d\|b\|w\|m\|y)` | `+` = strict; `b` = business days. |
| Hidden | `h:1` | Todo hidden from list but its attributes still feed autocomplete and the drawer. |
| Pomodoro | `pm:N` | Parsed and displayed only; no timer. |
| Stored priority | `pri:X` | floetask moves the priority here on completion and restores it on un-completion. |
| Note | `note:name.md` | Links a Markdown notes file in `<filename>-notes/` next to the todo file (`todo.txt` → `todo-notes/`), written from the add/edit dialog; the file and tag are created automatically (P1). See [note-extension.md](note-extension.md). |
| Other `key:value` | any | Preserved verbatim on round-trip. |

## 1.2 Parsed todo object

Each line becomes a `Todo` with: `line_number`, `raw` string, `body`, `complete`, `priority: Option<char>`, `created`, `completed`, `due`, `threshold`, `rec`, `pm`, `hidden`, `projects: Vec<String>`, `contexts: Vec<String>`, `extensions: Vec<(String,String)>`, and `notify` (computed). Lines are identified by line number because todo.txt has no ids; every write must re-validate that the target line still matches what the UI showed.

## 1.3 Round-trip guarantees

- Unknown tokens and extension order are preserved exactly.
- Writing a todo back must not reformat parts the user did not change.
- Lines matching any `exclude_lines_with_prefix` entry (e.g. `##`) are skipped by the parser and preserved on write.
- Blank lines are ignored for display.

## 1.4 Multi-line todos (P1)

When multi-line input is used, line breaks inside a todo are stored as the DLE control character `\x10` on a single file line, and rendered back as line breaks in the UI. Files written this way by other todo.txt tools load unchanged.

**Rust notes:** the `todo_txt` crate (4.2.x) exists, but the extras floetask supports (`rec:`, `t:`, `h:`, `pm:`, `pri:`, DLE, exclusion prefixes, exact round-trip) suggest writing a small own parser in a `core` crate with property tests for round-trip. Use `chrono` (or `jiff`) for dates.

## 1.5 Workflow status: the `status:` extension (P1)

`status:<value>` gives an open todo a workflow state. The full requirements are in [status-extension.md](status-extension.md); in short:

- Values are one lowercase token of `[a-z0-9_-]`. No tag means `todo`; done is never a status. At most one tag; the first valid one counts.
- Built-in values: `todo`, `doing`, `waiting`, `someday`. Unknown values are kept unchanged and treated as open.
- Setting a status replaces the tag in place; setting `todo` removes it. Completing a todo removes the tag, so a reopened todo is `todo`; the next occurrence of a recurring todo starts as `todo`.
- The list hides `someday` by default; a Filters toggle, a drawer filter on that status, or a search for `status:someday` shows it. `waiting` is styled distinctly.
- Default sort and grouping: `doing`, `todo`, `waiting`, `someday`, then custom statuses.
- `config.toml` `[statuses]` sets the order (custom statuses included) and which statuses are hidden by default; Settings → Statuses edits it ([§10](10-settings.md#10-settings-p0--p1)).
- The add/edit dialog has a status picker offering the global statuses plus any status that only the active file's board has a column for; cards show a status chip for every status except `todo`; the drawer has a Status section.

---

[Back to the overview](../FEATURES.md)
