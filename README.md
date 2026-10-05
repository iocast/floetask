<p align="center"><img src="assets/logo.svg" width="128" alt="floetask logo"></p>

# floetask

A todo.txt manager for the desktop, written in Rust with an [iced](https://iced.rs) GUI. The full specification is in [FEATURES.md](FEATURES.md); contributor and agent guidance is in [AGENTS.md](AGENTS.md).

## Run

```sh
cargo run --release -p floetask-app -- [path/to/todo.txt]
```

```text
floetask [OPTIONS] [TODO_FILE]

  -c, --config <FILE>  Config file to use instead of ~/.config/floetask/config.toml
      --paths          Print where floetask keeps its files and exit
```

A file passed on the command line is registered and opened. Without one, floetask reopens the files from last time.

## Files floetask uses

floetask follows the [XDG Base Directory Specification](https://specifications.freedesktop.org/basedir-spec/latest/) on every platform (Linux, macOS and Windows), relative to your home directory: what you configure goes in `~/.config`, where you left off goes in `~/.local/state`. The `XDG_CONFIG_HOME` and `XDG_STATE_HOME` variables override the defaults. `colors.toml` and `filters.toml` sit next to the config file, also when `--config` points elsewhere.

| Path | What | Lose it and... |
|---|---|---|
| `~/.config/floetask/config.toml` | Settings (written with defaults on first start; `--config` overrides) | settings reset |
| `~/.config/floetask/colors.toml` | Optional colour overrides | default colours |
| `~/.config/floetask/filters.toml` | Saved search filters (moved here from `~/.local/share/floetask/` on first start) | your saved searches are gone |
| `~/.local/state/floetask/state.toml` | Open files, layout, sorting, view toggles, window size | floetask forgets where you left off |
| `~/.local/state/floetask/notified.toml` | Notifications already shown today | you may see a notification twice |

Safe writes create `todo.txt.tmp` and `todo.txt.bak` next to your file for a moment and remove them on success.

### config.toml

```toml
append_creation_date = false
convert_relative_dates = true   # due:tomorrow -> due:2026-10-05 on save
human_friendly_dates = false    # show "today", "next week", ...
safe_writes = true
bulk_creation = false           # one todo per line in the add dialog
compact = false
notifications = true
notification_threshold_days = 2 # 0-10
zoom_percent = 100              # 50-150
theme = "system"                # system, light, dark
week_start = "monday"           # monday, saturday, sunday
language = "system"
exclude_lines_with_prefix = ["##"]

[statuses]                      # the status: extension
order = ["doing", "todo", "waiting", "someday"]  # sort order; add your own, e.g. "in-review"
hidden = ["someday"]            # left out of the list unless asked for

[watcher]
debounce_ms = 100
polling = false
poll_interval_ms = 1000

[[boards]]                      # board columns of one todo file; the Columns dialog writes these
file = "C:/Users/me/todo.txt"
columns = ["todo", "doing", "in-review", "done"]  # statuses, plus "done" for completed todos
```

### colors.toml

Any of `background`, `text`, `primary`, `project`, `success` (contexts), `warning`, `danger`, `navigation` (cards and panels), `priority_a`, `priority_b`, `priority_c`, `priority_other`, per mode:

```toml
[light]
priority_a = "#e11d48"

[dark]
background = "#111827"
```

Colours are read at start-up.

## Using it

floetask draws its own title bar, like Firefox and Zed: the file-drawer toggle and logo on the left, the search field in the middle, the filter drawer and settings on the right, then the window buttons. Drag the empty space to move the window, double-click it to maximise, and drag the window edges to resize.

The **file drawer** on the left (`Ctrl+Alt+H` to show or hide) has the **New todo** button, your files (each with its archive file listed right under it), and Open / Create at the bottom. Each file's ⋮ menu opens a floating list that sets its archive (done) file, opens the archive file in floetask, archives completed todos, shows the file in your file manager, or closes it.

The theme follows the system until you pick light or dark in Settings (or press `Ctrl+Alt+D`).

- **Add** with `Ctrl+N`. Type plain todo.txt; `+` and `@` autocomplete known projects and contexts (`Up`/`Down`, `Enter` or `Tab`). Pickers set priority, due and threshold dates, recurrence and pomodoros. `Ctrl+Enter` saves.
- **Status** with `status:doing`, `status:waiting`, `status:someday` or your own value ([status-extension.md](features/status-extension.md)); the dialog has a status picker. Manage the global statuses (order, hidden, your own) in Settings → Statuses. The list groups by status (doing, to do, waiting), hides `someday` until you turn it on in the Filters tab, filter on it or search for `status:someday`, and completing a todo removes its status.
- **Board** with the title-bar button or `Ctrl+Alt+B`: one column per status, plus Done. Drag a card to another column to change its status, or to Done to complete it. **Columns** on the board sets the columns for that file. When the list is grouped (for example by priority, after moving Priority to the top of the Sorting tab), each group gets its own board; the "Group by" switch next to Columns turns that off while keeping the sort order.
- **Edit** by clicking a todo or pressing `Enter` on the selected one. Hover a todo for Edit, Copy, Archive and Delete.
- **Complete** with the checkbox or `Space`. Completing a `rec:` todo adds its next occurrence.
- **Filter** with the chips on a todo or in the drawer (`Ctrl+B`): click to include, Alt+click to exclude, right-click a project or context to rename or remove it across the file.
- **Search** in the title bar (`Ctrl+F` focuses it). Plain text matches anywhere; expressions such as `+work and due: < today+3d`, `(A) or pri >= C`, `not complete`, `/regex/` are evaluated. `Ctrl+Enter` in the search field turns the text into a new todo. Save searches with the star and pick them from the arrow (`Ctrl+Shift+F`); the bell mutes notifications for matching todos.
- **Archive** completed todos with `Ctrl+Alt+A` or the file's ⋮ menu once an archive file is set (⋮ → Set archive file, or you are asked on first archive).

| Shortcut | Action |
|---|---|
| `Ctrl+N` | New todo |
| `Ctrl+F` / `Ctrl+Shift+F` | Focus search / saved filters |
| `Ctrl+H` | Show or hide completed todos |
| `Ctrl+0` | Reset search and filters |
| `Ctrl+Alt+A` | Archive completed todos |
| `Ctrl+O` | Open a file |
| `Ctrl+1` … `Ctrl+9` | Switch file |
| `Ctrl+,` | Settings |
| `Ctrl+B` | Filter drawer |
| `Ctrl+Alt+H` | File drawer |
| `Ctrl+Alt+B` | Switch between list and board |
| `Ctrl+Alt+D` | Toggle light/dark |
| `Ctrl+W` / `Ctrl+Q` | Quit |
| `Up` / `Down`, `Enter`, `Space`, `Delete` | Select, open, complete, delete |
| `Escape` | Close dialog, menu, filter drawer, then clear the search |

## Status against FEATURES.md

Implemented: the todo.txt model with exact round-trip and multi-line todos (DLE), completion with `pri:`, recurrence (strict, business days, threshold gap), safe writes, a debounced file watcher with polling option, multiple files in a file drawer with per-file menus, drag and drop, done files and archiving, grouped and sorted list with counts, Markdown and explicit link opening, `note:` links to notes files (see [note-extension.md](features/note-extension.md)), empty states, compact mode and zoom, the add/edit dialog with autocomplete and pickers, bulk creation, the drawer (attributes, filters, sorting, rename/remove, hide category), the search language and saved filters, due-date notifications with de-duplication and suppression, the settings dialog, system/light/dark themes and the colour file, natural-language and human-friendly dates, keyboard shortcuts, and i18n-ready strings.

Not done yet:

- System tray, dock badge, start minimised (P2).
- The status board has no keyboard navigation and no reordering inside a column; cards follow the sort order.
- Native or in-app menu bar (P2); every action is reachable by shortcut or UI.
- Translations other than English; the language setting only offers English.
- Packaging (P2).
- App-level undo (P2), an archive viewer, single-instance mode.
- Drag-and-drop reordering of sort criteria: up/down buttons instead (the spec's P1 fallback).
- `Left`/`Right` focus moves inside a row; hover actions replace a pop-up context menu.
- Autocomplete suggestions show under the text field, not at the cursor.
- "Disable animations" is stored but has no effect, since floetask has no animations.

## Logo

`assets/logo.svg` is the source of the current logo (v2): a hexagonal ice crystal broken open by a done mark. It is the window icon, the Windows exe icon and appears in the app. After editing it, regenerate the PNG and ICO files with `cargo run -p render-logo`. The first design is kept in `assets/v1/`.

## Development

See [AGENTS.md](AGENTS.md) for the architecture and rules. In short:

```sh
cargo test --workspace
cargo clippy --workspace --all-targets
cargo fmt --all
```
