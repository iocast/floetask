# 10. Settings (P0 / P1)

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

Config-file-only settings: `exclude_lines_with_prefix` (list), watcher options ([§2.1](02-files.md#21-file-watching)).

Persisted UI state (not in the dialog): registered files and active file, sorting, drawer/navigation/search open state, accordion states, file tabs visibility, window size/position/maximised, view toggles from [§6.2](06-drawer.md#62-filters-tab-view-toggles).

**Rust notes:** config as TOML or JSON via `serde` in the platform config dir (`directories` crate), e.g. `~/.config/floetask/`. Files: `config`, `colors`, `filters`, `notified`. Versioned with a schema number and migrations.

---

[Back to the overview](../FEATURES.md)
