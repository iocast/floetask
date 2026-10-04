# AGENTS.md

Guidance for AI agents and contributors working on floetask, a todo.txt manager in Rust with an iced GUI. The product spec is [FEATURES.md](FEATURES.md); store new requirement docs next to it in the repo root.

## Architecture: clean architecture, enforced by crates

Dependencies point inward only. Cargo enforces this: a crate cannot use a layer it does not depend on.

```
floetask-app ──► floetask-gui ──► floetask-application ──► floetask-domain
     │                               ▲
     └──────► floetask-infrastructure ─┘
```

| Crate | Layer | Holds | May depend on |
|---|---|---|---|
| `floetask-domain` | Entities and business rules | `Todo` parsing and round-trip, completion, recurrence, `TodoDocument`, search language, natural and human-friendly dates, listing (filter, sort, group, drawer counts) | `chrono`, `regex`, `thiserror` only |
| `floetask-application` | Use cases and ports | `TodoFileService`, `NotificationService`, `Settings`, `AppState`, `SavedFilter`, port traits in `ports.rs` | domain |
| `floetask-infrastructure` | Adapters | Local file system with safe writes, debounced watcher, TOML stores (config, state, filters, notified, colors), XDG paths, OS notifications, opening links | domain, application, I/O crates |
| `floetask-gui` | Presentation | iced app: state, messages, update handlers, views, theme, i18n | domain, application, `iced`, `rfd` (never infrastructure) |
| `floetask-app` | Composition root | CLI (`clap`), wires infrastructure ports into services, starts the GUI | everything |

Rules:

- **Domain is pure.** No I/O, no clock, no serde. "Today" is always a parameter. New business rules go here with unit tests.
- **Application talks to the world only through `ports.rs` traits.** A new external need (a file, the OS, a service) means a new port trait here and an adapter in infrastructure.
- **Serialisation lives in infrastructure.** Each store owns a private DTO and maps to and from application types. Do not add `serde` derives to domain or application types.
- **The GUI never touches infrastructure.** It receives `Services` from the composition root and calls use cases from background tasks (`Task::perform`).
- **The GUI keeps logic thin.** Handlers call services and update presentation state. Text logic worth testing goes in a widget-free module (see `compose.rs`).

## File locations (XDG)

Same layout on every OS, relative to the home directory; `XDG_CONFIG_HOME`, `XDG_DATA_HOME` and `XDG_STATE_HOME` override the defaults. Rule of thumb: would the user lose real work if it vanished → `share`; is it history, logs or "where I left off" → `state`; settings the user edits → `config`; regenerable → `cache`.

| Directory | Files |
|---|---|
| `~/.config/floetask/` | `config.toml` (settings; `--config FILE` overrides the path), `colors.toml` (next to the config file) |
| `~/.local/share/floetask/` | `filters.toml` (saved searches) |
| `~/.local/state/floetask/` | `state.toml` (registered files, layout, view toggles, window), `notified.toml` (notification de-dup) |
| `~/.cache/floetask/` | unused so far |

Safe writes put `<file>.tmp` and `<file>.bak` next to the target file, because an atomic rename only works within one file system. `floetask --paths` prints the resolved locations.

## Code style

- Readable first: small functions, descriptive names, a doc comment on every public item and module explaining *why*, not just what.
- Match the surrounding code. Comments are sparse and only explain non-obvious decisions.
- `rustfmt.toml` sets `max_width = 120`. Run `cargo fmt --all`.
- `cargo clippy --workspace --all-targets` must stay warning-free.
- User-facing strings go through `i18n::tr` / `trf` in `floetask-gui/src/i18n.rs`; add the English text there.
- todo.txt round-trip is sacred: edits must change only the tokens they touch (`Todo::rebuild`, token spans). Add a property test when you touch parsing.

## Testing

```sh
cargo test --workspace
cargo clippy --workspace --all-targets
FLOETASK_SNAPSHOTS=/some/dir cargo test -p floetask-gui   # also writes PNG snapshots of screens
```

- Domain: unit and `proptest` tests next to the code; worked examples from FEATURES.md are acceptance tests.
- Application: use-case tests with in-memory fakes of the ports (`todo_files/tests.rs`).
- Infrastructure: tests against `tempfile` directories.
- GUI: headless `iced_test` simulator tests in `floetask-gui/src/tests.rs` (click, type, feed messages back into `update`).

To try the app without touching real settings, point the XDG variables at a scratch directory before running `cargo run -p floetask-app -- path/to/todo.txt`.

## Where things are

- Message flow: `floetask-gui/src/app/mod.rs` dispatches each `Message` to a handler module (`files.rs`, `list.rs`, `editing.rs`, `drawer.rs`, `search.rs`, `settings.rs`, `shortcuts.rs`).
- Event sources (keyboard, window, theme, watcher, tick): `floetask-gui/src/app/subscriptions.rs`.
- Views: `floetask-gui/src/view/`. The window is borderless: `title_bar.rs` draws the centred search, actions and window buttons, `files.rs` the file drawer, `popover.rs` the floating menus (a small custom widget on iced's overlay layer), `mod.rs` adds the resize edges, and `app/window_frame.rs` handles drag, resize, minimise and maximise.
- Look and feel: colours and every widget style live in `theme.rs` (light and dark `Colors`); icons are drawn in `view/icons.rs`. Views use these instead of iced's default styles.
- Known gaps against FEATURES.md are listed in the README.
