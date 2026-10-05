# floetask: feature specification

A desktop todo.txt manager written in Rust (edition 2024) with an [iced](https://iced.rs) GUI.

- Target: iced **0.14** (current stable), desktop only (Linux, Windows, macOS).
- Status: specification only. No code has been written.

Priority tags used below:

- **P0**: needed for a usable first release (core todo.txt editing).
- **P1**: needed for a complete feature set.
- **P2**: polish or platform extras; can follow later.
- **Out**: deliberately left out.

---

## 1. todo.txt data model and parsing (P0)

### 1.1 Supported syntax

| Element | Syntax | Notes |
|---|---|---|
| Completion | `x ` prefix | Followed by completion date then creation date. |
| Priority | `(A)`–`(Z)` at line start | A–C get distinct colours; D–Z share a neutral colour. |
| Creation date | `YYYY-MM-DD` after priority | Optional; appended automatically when the setting is on. |
| Completion date | `x YYYY-MM-DD` | Set automatically on completion. |
| Projects | `+name` | Any number, no spaces. |
| Contexts | `@name` | Any number, no spaces. |
| Due date | `due:YYYY-MM-DD` | |
| Threshold date | `t:YYYY-MM-DD` | Defers visibility (see §4). |
| Recurrence | `rec:[+][N](d\|b\|w\|m\|y)` | `+` = strict; `b` = business days. |
| Hidden | `h:1` | Todo hidden from list but its attributes still feed autocomplete and the drawer. |
| Pomodoro | `pm:N` | Parsed and displayed only; no timer. |
| Stored priority | `pri:X` | floetask moves the priority here on completion and restores it on un-completion. |
| Note | `note:name.md` | Links a Markdown notes file in `<filename>-notes/` next to the todo file (`todo.txt` → `todo-notes/`), written from the add/edit dialog; the file and tag are created automatically (P1). See [NOTE-EXTENSION.md](NOTE-EXTENSION.md). |
| Other `key:value` | any | Preserved verbatim on round-trip. |

### 1.2 Parsed todo object

Each line becomes a `Todo` with: `line_number`, `raw` string, `body`, `complete`, `priority: Option<char>`, `created`, `completed`, `due`, `threshold`, `rec`, `pm`, `hidden`, `projects: Vec<String>`, `contexts: Vec<String>`, `extensions: Vec<(String,String)>`, and `notify` (computed). Lines are identified by line number because todo.txt has no ids; every write must re-validate that the target line still matches what the UI showed.

### 1.3 Round-trip guarantees

- Unknown tokens and extension order are preserved exactly.
- Writing a todo back must not reformat parts the user did not change.
- Lines matching any `exclude_lines_with_prefix` entry (e.g. `##`) are skipped by the parser and preserved on write.
- Blank lines are ignored for display.

### 1.4 Multi-line todos (P1)

When multi-line input is used, line breaks inside a todo are stored as the DLE control character `\x10` on a single file line, and rendered back as line breaks in the UI. Files written this way by other todo.txt tools load unchanged.

**Rust notes:** the `todo_txt` crate (4.2.x) exists, but the extras floetask supports (`rec:`, `t:`, `h:`, `pm:`, `pri:`, DLE, exclusion prefixes, exact round-trip) suggest writing a small own parser in a `core` crate with property tests for round-trip. Use `chrono` (or `jiff`) for dates.

### 1.5 Workflow status: the `status:` extension (P1)

`status:<value>` gives an open todo a workflow state. The full requirements are in [STATUS-EXTENSION.md](STATUS-EXTENSION.md); in short:

- Values are one lowercase token of `[a-z0-9_-]`. No tag means `todo`; done is never a status. At most one tag; the first valid one counts.
- Built-in values: `todo`, `doing`, `waiting`, `someday`. Unknown values are kept unchanged and treated as open.
- Setting a status replaces the tag in place; setting `todo` removes it. Completing a todo removes the tag, so a reopened todo is `todo`; the next occurrence of a recurring todo starts as `todo`.
- The list hides `someday` by default; a Filters toggle, a drawer filter on that status, or a search for `status:someday` shows it. `waiting` is styled distinctly.
- Default sort and grouping: `doing`, `todo`, `waiting`, `someday`, then custom statuses.
- `config.toml` `[statuses]` sets the order (custom statuses included) and which statuses are hidden by default; Settings → Statuses edits it (§10).
- The add/edit dialog has a status picker offering the global statuses plus any status that only the active file's board has a column for; cards show a status chip for every status except `todo`; the drawer has a Status section.

---

## 2. Files (P0 / P1)

| Feature | Pri | Behaviour |
|---|---|---|
| Open existing todo.txt | P0 | File picker (`rfd`), `Ctrl+O`. |
| Create new todo.txt | P0 | Save dialog, creates an empty file. |
| Drag and drop a file onto the window | P1 | Opens it. iced exposes `window::Event::FileDropped`. |
| Multiple files | P1 | Several registered files, one active at a time. |
| File tabs | P1 | Tab bar to switch files; toggle tabs visibility (View menu). |
| Switch file by shortcut | P1 | `Ctrl+1`…`Ctrl+9`. |
| Tab context menu | P1 | Change done-file location, reveal todo file, reveal done file (open in OS file manager), remove from list (with confirmation). |
| Done file per todo file | P1 | Each todo file can link a `done.txt` used for archiving. |
| File watcher | P0 | External changes reload the list automatically (see §2.1). |
| Safe writes | P1 | Write to `.tmp`, verify, create `.bak`, atomically replace, delete both on success. Toggle in settings, default on. |
| Remember files and active file | P0 | Persisted in config. |

### 2.1 File watching

- Watch all registered todo files and the saved-filters file.
- Debounce events until the file has been stable for about 100 ms so sync tools (Syncthing, Dropbox) do not cause flicker or false deletions.
- Watcher options are configurable (e.g. debounce ms, polling on/off, poll interval).
- Ignore events caused by the app's own writes.

**iced notes:** use `notify` (8.x) + `notify-debouncer-full`, wrapped as an iced `Subscription` (`Subscription::run` with a channel stream) that emits `Message::FileChanged(path)`. All file I/O runs in `Task::perform` so the UI never blocks.

---

## 3. Todo list view (P0)

### 3.1 Rows

Each row shows:

- Completion checkbox (`Space` toggles when focused).
- Priority badge with colour bar (A–C coloured, others grey). The badge is hidden when the list is already grouped by priority.
- Body text, rendered with Markdown (§3.4).
- Attribute chips for projects, contexts, due, t, rec, pm. Clicking a chip toggles it as a filter. Overdue/today due chips are red with a dot.
- Created and completed date indicators (tooltip with the date).
- Inline date picker on the due/t chip to change the date directly.
- Completed todos are visually muted/struck through.

Row interactions:

- Click or `Enter` opens the edit dialog.
- Right-click context menu: **Copy** (raw line to clipboard), **Archive** (move this single todo to the done file, completed or not), **Delete** (with confirmation showing the raw line).
- `Delete` or `Ctrl+Backspace` deletes the selected row (with confirmation).
- `Up`/`Down` moves row focus; `Left`/`Right` moves between buttons inside a row.

### 3.2 Grouping

- The list is grouped by the **first** active sort attribute (e.g. by priority, then sorted within groups by the rest).
- Each group has a header with its value (attribute values in headers are right-clickable for rename/remove, §6).
- Groups whose todos are all hidden are not shown.
- "File order" mode (`fileSorting`) disables grouping and shows todos in file order; in that mode an optional "sort completed last" applies.

**Collapsing groups:** clicking a group header (chevron, name and count) collapses the group to its header; clicking again expands it. Keyboard selection skips the rows of collapsed groups. Collapsed groups are remembered in `state.toml` by attribute and value (e.g. `status:todo`), separately from the board's.

### 3.3 Header counts

Show visible / total counts and completed count.

### 3.4 Markdown in descriptions (P1)

- Bold, italic, strikethrough, inline code, lists, blockquotes, headings, tables, links.
- Bare URLs are linkified.
- Links support `http(s)://`, `file://` and custom schemes (`joplin://`, `cbthunderlink://`, …), opened via the OS handler (`open` crate) through an explicit open icon.

**iced notes:** iced 0.14 ships a `markdown` widget (`iced::widget::markdown`, pulldown-cmark based) that covers most of this; table support needs verifying. Link clicks arrive as messages, which keeps link opening explicit. Rendering hundreds of markdown rows can be heavy: parse once per todo into `markdown::Item`s, cache in state, and use `lazy` or a virtualised list for large files.

### 3.5 Empty states (P0)

Splash screens for: no file open (open/create buttons), file has no todos (add button), and no todos visible because of filters (reset filters button).

### 3.6 Layout options (P1)

- Compact mode (denser rows), toggle in settings and View menu.
- Zoom 50–150 % in 10 % steps. iced: apply via the application `scale_factor`.
- Disable animations toggle (relevant only if floetask adds animations).

### 3.7 Status board (P1)

A kanban view of the active file, built on the `status:` extension (1.5).

- **Switching:** a title-bar button and `Ctrl+Alt+B` switch between the list and the board. The choice is remembered in `state.toml`.
- **Columns:** each column is a status (`todo`, `doing`, a custom value such as `in-review`) or `done`, which holds completed todos (done is never a status). Default columns: To do, Doing, Waiting, Done.
- **Per file:** every todo file has its own columns, edited in a Columns dialog on the board (add a known status, type a new one, reorder, remove, reset). They are stored in `config.toml` as `[[boards]]` entries with the file path and the column keys; a file with the default columns has no entry. A board always keeps at least one column.
- **Column statuses stay per file:** a column for a status that is not in the global list (e.g. `in-review`) does not add it to `[statuses]`. The todo dialog's status picker offers it while that file is active. To use a status everywhere, add it in Settings → Statuses.
- **One board per group:** when the list is grouped (the first Sorting criterion, unless file order is on), every group gets its own board with the same columns, stacked under the group's header like swimlanes (e.g. one board per priority or per project). Grouping by status is ignored on the board, since the columns already are statuses, so the default sort shows a single board. Columns in grouped boards grow with their cards and the page scrolls both ways. Dragging a card to another group's board only changes its status: dropped on the same column of another group, nothing changes. If any group has todos in the "Other statuses" column, every group shows that column.
- **Collapsing groups:** clicking a group's header (chevron, name and count) collapses its board to just the header, and clicking again expands it. Collapsed groups are remembered in `state.toml` by attribute and value (e.g. `priority:A`), so they stay collapsed after a refresh or restart.
- **Grouping switch:** left of the Columns button, a "Group by <attribute>" switch turns the per-group boards off and on. Off shows a single board while keeping the sort order inside each column. The choice is remembered in `state.toml`; the switch is disabled while the list is grouped by status or in file order.
- **Width:** the board sits between the file drawer and the filter drawer and never goes under either. Columns share the available width equally, so opening a drawer narrows them. Only when a column would get narrower than 200 px do columns keep that width and the board scroll sideways. The list or board area is clipped to its slot, so nothing ever draws under a drawer.
- **Cards:** the same card as the list (priority bar, checkbox, text, chips) without the status chip, sorted inside a column by the Sorting tab criteria.
- **Drag and drop:** press a card, move to another column (it is outlined) and release to move the card there. Dropping on a status column sets `status:` in place (`todo` removes the tag) and reopens a completed todo; dropping on Done completes it, adding the next occurrence of a recurring todo. Pressing and releasing in the same column opens the todo for editing. `Escape` cancels a drag.
- **What is shown:** the board follows every change in the drawer and the search exactly like the list: search, attribute filters, view toggles (Show completed also empties the Done column) and sorting, including file order. The one addition: a status with a column is always shown (a Someday column shows `someday` todos). Open todos whose status has no column appear in a trailing "Other statuses" column, unless their status is hidden by default; completed todos without a Done column are not shown.

**iced notes:** iced 0.14 has no drag and drop. The board uses `mouse_area`: pressing a card starts the drag, each column reports entering and leaving, and a release anywhere on the board drops on the column under the cursor.

---

## 4. Dates, deferral, recurrence (P0 / P1)

### 4.1 Completion

Marking complete:

1. Prepend `x ` and today's completion date.
2. If no creation date exists, set creation date to today (so the line becomes `x 2023-09-26 2023-09-26 …`).
3. Move priority into `pri:X`.
4. If `rec:` exists, create the next occurrence (§4.3).

Un-completing reverses 1 and 3 (restores priority from `pri:`).

### 4.2 Creation date

Setting `appendCreationDate`: new todos get today's date as creation date.

### 4.3 Recurrence

- Units: `d` daily, `b` business days (Mon–Fri), `w` weekly, `m` monthly, `y` yearly; optional count `N ≥ 1` (`rec:3m`).
- **Non-strict** (default): next due = completion date + interval.
- **Strict** (`rec:+…`): next due = previous due + interval; if there was no due date, falls back to completion date + interval.
- A todo with `rec:` but no due date still gets a due date on the new copy.
- With both `due:` and `t:`:
  - strict: add the interval to both;
  - non-strict: add the interval to `due` from completion date, then set `t` so the original `due − t` gap is preserved.
- The new copy gets today's creation date and keeps all other attributes.

Worked examples like this one are unit tests, e.g. `Water plants … due:2021-07-19 t:2021-07-09 rec:14d` completed 2021-07-13 → `due:2021-07-27 t:2021-07-17`.

### 4.4 Threshold (deferred) todos

`t:` in the future hides the todo unless the drawer toggle "show todos with future threshold date" is on. A matching toggle exists for future due dates.

### 4.5 Natural-language dates (P1)

When `convertRelativeToAbsoluteDates` is on, `due:tomorrow`, `t:in one week`, `due:next tuesday`, `due:end of february` are rewritten to ISO dates on save. Week start (Monday / Saturday / Sunday) is a setting.

**Rust notes:** candidates are `chrono-english`, `two_timer`, or a small own grammar covering the common phrases. The spec target is the common English phrases; a broader grammar is a stretch goal.

### 4.6 Human-friendly date display (P1)

Optional setting that displays dates as `overdue` / `elapsed`, `last week`, `today`, `tomorrow`, `next week`, `this month`, `next month`, else ISO. In the drawer, a date contributes to all buckets it falls in (e.g. due today shows under `today`, `this week`, `this month`).

---

## 5. Add / edit dialog (P0)

- Opened by `Ctrl+N`, the add button, or opening a row.
- Multi-line text field holding the raw todo.txt line. `Ctrl+Enter` saves, `Escape` closes.
- **Autocomplete**: typing `+` or `@` shows suggestions from all known projects/contexts (including from hidden `h:1` todos). `Up`/`Down` to move, `Enter` to insert.
- **Pickers** that edit the text in place:
  - Priority picker (none, A–Z).
  - Due date picker.
  - Threshold date picker.
  - Recurrence picker (unit + count + strict toggle).
  - Pomodoro picker (number).
- **Notes** (P1): a Markdown field for the todo's notes. Saving writes them to the `note:` file, creating the file and the tag from the task text when the todo has none (see [NOTE-EXTENSION.md](NOTE-EXTENSION.md)).
- **Bulk creation** (setting): each line in the field becomes a separate todo; the Add button shows the count, e.g. `Add (3)`.
- Empty input shows a non-blocking warning (toast).
- Creating a todo from the search box: `Ctrl+Enter` in the search field saves its text as a new todo.

**iced notes:** use `text_editor` for the multi-line field. iced has no built-in autocomplete or date picker: autocomplete is a custom overlay anchored to the cursor position (track the token at the cursor from `text_editor::Content`), and the date picker can come from `iced_aw` (check 0.14 compatibility) or be a small custom calendar widget. Dialogs are modal overlays built with `stack` + `opaque` + `mouse_area`.

---

## 6. Sidebar drawer: attributes, filters, sorting (P1)

Toggled with `Ctrl+B`; `Escape` closes. Three tabs:

### 6.1 Attributes tab

- One collapsible section per attribute: priority, status, projects, contexts, due, t, rec, pm, created, completed (open/closed state persisted).
- Each value shows a count of todos having it; counts reflect the filtered set; overdue sections show a red indicator.
- Click a value to **include** filter; second action (modifier-click) to **exclude** filter. Multiple values combine (include = must match one of; exclude = must match none).
- Hide a whole category (e.g. hide every todo that has any context).
- Right-click a value: **Rename** or **Remove** across all todos in the active file, with confirmation. Case-sensitive. Not available for dates.
- Option to include attributes from hidden todos in the drawer.

### 6.2 Filters tab (view toggles)

- Show completed todos (`Ctrl+H`).
- Show hidden (`h:1`) todos.
- Show attributes from hidden todos in drawer.
- Show todos with future threshold dates.
- Show todos with future due dates.
- Show todos whose status is hidden by default (`someday`).

### 6.3 Sorting tab

- Ordered list of sort attributes (default order: status, priority, projects, contexts, due, t, completed, created, rec, pm). Reorder by drag and drop.
- Per attribute: invert direction.
- Recurrence sorts by semantic length (1d < 1w < 1m < 1y; relative before strict; missing last).
- Missing values always sort last regardless of direction.
- "File order" toggle and, under it, "sort completed last".

**iced notes:** iced 0.14 has no built-in drag-and-drop list reordering; implement with `mouse_area` + manual drag state, or provide up/down buttons as a P1 fallback and drag as P2. Collapsible sections are simple custom components.

---

## 7. Search (P1)

### 7.1 Search bar

- `Ctrl+F` opens/focuses. `Escape` clears, a second `Escape` hides.
- Plain text does a case-insensitive substring match on the raw line.

### 7.2 Advanced filter expressions

If the input parses as an expression it is evaluated; otherwise it falls back to literal match. While typing an invalid expression, keep showing results of the last valid one.

Grammar:

- Logical: `or`/`OR`/`||`, `and`/`AND`/`&&`, `not`/`NOT`/`!`, parentheses; precedence not > and > or.
- `+` any project; `+bi` project contains `bi`; `+"big"` exact project.
- `@` / `@ho` / `@"home"` likewise for contexts.
- `due:` has a due date; `due: <op> DATE`; `due:2021-06` prefix match. Same for `t:`.
- DATE = ISO date or `today` / `tomorrow` / `yesterday`, optionally `± N(d|b|w|m|y)`.
- `priority` / `pri` has priority; `pri <op> A`; `(B)` shorthand for `pri == B`.
- `complete` keyword.
- `"text"` or `'text'` literal; `/regex/` regex match.
- Operators: `==`, `=`, `!=`, `<`, `<=`, `>`, `>=`.

**Rust notes:** implement with `pest` or `chumsky` (or hand-written recursive descent); `regex` crate for `/…/` (syntax differs slightly from JavaScript regex; document the difference).

### 7.3 Saved search filters

- Save the current query as a named filter (`Ctrl+Shift+F` / arrow-down opens the list).
- Pick, delete (with confirmation) saved filters; `Up`/`Down` + `Enter` in the list.
- Per filter: **suppress notifications** for todos matching it (bell icon).
- Stored in `filters.json`, watched for external changes.

---

## 8. Archiving (P1)

- Archive all completed todos of the active file to its done file (`Ctrl+Alt+A`, navigation button). Appends to done.txt and removes from todo.txt.
- Archive a single todo from the row context menu, whether completed or not.
- If no done file is linked, prompt to choose or create one.
- Optional view of the archive.

Both writes must be done so that a crash cannot lose todos: append to done first (safe write), then rewrite todo.

---

## 9. Notifications, badges, tray (P1 / P2)

| Feature | Pri | Behaviour |
|---|---|---|
| Due-date notifications | P1 | OS notification for incomplete todos due today through the threshold (0–10 days, default 2). Not for overdue or completed todos. Toggle in settings. |
| De-duplication | P1 | Store a hash of (today's date + todo text) in `notifiedTodoObjects.json`; skip if seen. |
| Suppression via saved filter | P1 | See §7.3. |
| Overdue badges in UI | P1 | Red dot on overdue due chips; drawer counts and sections turn red. |
| Dock badge count (macOS) | P2 | Number of todos due within the threshold. |
| System tray | P2 | Toggle; tray menu shows "show floetask", file list to switch, quit. Options: invert tray icon colour, start minimised to tray. Closing the window keeps the app in the tray when enabled. |

**iced notes:** `notify-rust` for notifications; a periodic `iced::time::every` subscription re-checks due dates (e.g. on file change and every few minutes / at midnight). Tray via `tray-icon` (0.26), which needs its own event loop integration: on Linux it requires a GTK loop, so run it on a dedicated thread and forward events into iced through a subscription channel. Window hide/show via `iced::window` tasks. Dock badge has no cross-platform crate; mark macOS-only.

---

## 10. Settings (P0 / P1)

Settings dialog (`Ctrl+,`). A sidebar on the left lists the sections (Todos, Dates, Statuses, Appearance, Notifications, Files); the right side shows the chosen section. Every setting shows its name with a short description underneath in grey, and its control on the right.

| Section | Settings |
|---|---|
| Todos | append creation date, bulk todo creation |
| Dates | convert relative dates, human-friendly dates, week start |
| Statuses | the global status list (below) |
| Appearance | theme, language, zoom, compact mode, disable animations |
| Notifications | notifications, notification threshold |
| Files | safe writes; shows where the settings file is |

**Status management** (Statuses section): the global statuses of the `status:` extension (1.5) in sort order, each with its name, key and a description (built-in meaning, or "Your own status"). Each status can be moved up or down and marked hidden (left out of the list by default; not for `todo`). Custom statuses can be added (same rules as status values) and removed; the four built-in ones cannot be removed. Changes are saved to `[statuses]` in `config.toml` right away and re-sort and re-filter the list.

All settings:

| Setting | Type | Default |
|---|---|---|
| Append creation date | toggle | off |
| Convert relative to absolute dates | toggle | on |
| Human-friendly dates | toggle | off |
| Tray | toggle | off |
| Invert tray colour (needs tray) | toggle | off |
| Start minimised (needs tray) | toggle | off |
| Safe writes | toggle | on |
| Menu bar visibility (Win/Linux) | toggle | on |
| Bulk todo creation | toggle | off |
| Disable animations | toggle | off |
| Compact mode | toggle | off |
| Notifications | toggle | on |
| Notification threshold | slider 0–10 days | 2 |
| Zoom | slider 50–150 % | 100 |
| Colour theme | system / light / dark | system |
| Week start | Mon / Sat / Sun | Mon |
| Language | select | system |

Config-file-only settings: `exclude_lines_with_prefix` (list), watcher options (§2.1).

Persisted UI state (not in the dialog): registered files and active file, sorting, drawer/navigation/search open state, accordion states, file tabs visibility, window size/position/maximised, view toggles from §6.2.

**Rust notes:** config as TOML or JSON via `serde` in the platform config dir (`directories` crate), e.g. `~/.config/floetask/`. Files: `config`, `colors`, `filters`, `notified`. Versioned with a schema number and migrations.

---

## 11. Theming and colours (P1)

- Light and dark themes; "system" follows the OS (`Ctrl+Alt+D` toggles).
- `colors.json`-equivalent user file overriding semantic colours, theme colours, navigation colours and priority colours (A, B, C, others). Reloaded on restart (P1) or live (P2).

**iced notes:** iced 0.14 supports custom `Theme::custom(palette)` and per-widget style closures; build a floetask palette struct loaded from the colour file. Follow-system needs OS dark-mode detection (`dark-light` crate) polled or subscribed.

---

## 12. Navigation, menus, keyboard (P0 / P1)

### 12.1 Navigation sidebar

Vertical icon bar: add todo, toggle drawer, archive (when a done file is set and completed todos exist), open file, settings. Toggle with `Ctrl+Alt+H`.

### 12.2 Application menu (P2)

An application menu (File, Edit, View, Todos, Window, Help). iced has no native menu bar; options are an in-app menu bar (`iced_aw` menu or custom) or `muda` for native menus. All menu actions must also be reachable via shortcuts or the UI so the menu stays optional.

### 12.3 Keyboard shortcuts

| Action | Shortcut (Cmd on macOS where applicable) |
|---|---|
| New todo | Ctrl+N |
| Find | Ctrl+F |
| Show saved search filters | Ctrl+Shift+F |
| Toggle completed | Ctrl+H |
| Reset search and filters | Ctrl+0 |
| Archive completed | Ctrl+Alt+A |
| Open file | Ctrl+O |
| Switch to file 1–9 | Ctrl+1…9 |
| Settings | Ctrl+, |
| Toggle drawer | Ctrl+B |
| Toggle navigation | Ctrl+Alt+H |
| Toggle theme | Ctrl+Alt+D |
| Close window / Quit | Ctrl+W / Ctrl+Q |
| Save in dialog / save search as todo | Ctrl+Enter |
| Close dialog / drawer / clear search | Escape |
| Open selected todo | Enter |
| Delete selected todo | Delete or Ctrl+Backspace |
| Move focus in list | Up / Down, Left / Right |
| Toggle complete | Space |

**iced notes:** global shortcuts via `keyboard::listen` / `keyboard::on_key_press` subscription mapping to messages; row focus and selection must be app state since iced's focus model only covers text inputs. Use `widget::operation` (focus, scroll_to) to keep the selected row visible.

### 12.4 Clipboard and undo

- Copy todo to clipboard (`iced::clipboard`).
- Undo/redo inside text fields. P2: app-level undo of the last file change.

---

## 13. Internationalisation (P2)

Candidate languages: English, German, Italian, Spanish, French, Simplified Chinese, Brazilian Portuguese, Portuguese, Japanese, Turkish, Hungarian, Czech, Polish, Russian, Korean, Hindi. floetask should be built i18n-ready from the start (all strings through a lookup) with English first. Candidates: `rust-i18n` (simple, JSON/YAML) or `fluent`. CJK and Devanagari need a bundled font fallback in iced.

---

## 14. Platform and distribution (P2)

- Linux, Windows, macOS builds (`cargo-dist` or `cargo-packager`).
- Packaging targets: AppImage, Flatpak, Snap, AUR, Homebrew cask, Windows/Mac stores. Sandboxed builds need portal-based file access (Flatpak document portal, macOS security-scoped bookmarks); `rfd` with the `xdg-portal` backend covers the Flatpak picker.
- Single-instance behaviour and window state restore.

---

## 15. Out of scope

- **Telemetry and analytics**.
- **Sponsoring / donation links**.
- **Pomodoro timer**: floetask only parses and displays `pm:`.

---

## 16. Suggested crate layout (for later, not yet created)

```
floetask/
  Cargo.toml            # workspace, edition = "2024"
  crates/
    floetask-core/        # todo.txt parser, recurrence, dates, filter language, sort/group, archive, safe write
    floetask-app/         # iced application: state, messages, views, subscriptions
```

Keeping all todo logic in `floetask-core` with no iced dependency lets it be fully unit-tested, with worked examples as acceptance tests.

## 17. Suggested milestones

1. **M1 core**: parser with round-trip tests, completion, recurrence, safe write, file watcher.
2. **M2 usable app**: open/create file, list view with grouping and sorting, add/edit dialog with pickers, settings persistence, light/dark theme, main shortcuts.
3. **M3 full feature set**: drawer (attributes, filters, sorting, rename/remove), advanced search and saved filters, archiving, multiple files and tabs, notifications, markdown, natural dates, human-friendly dates, bulk/multi-line.
4. **M4 polish**: tray, native menu, i18n, colour file, packaging.
