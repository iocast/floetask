# 9. Notifications, badges, tray (P1 / P2)

| Feature | Pri | Behaviour |
|---|---|---|
| Due-date notifications | P1 | OS notification for incomplete todos due today through the threshold (0–10 days, default 2). Not for overdue or completed todos. Toggle in settings. |
| De-duplication | P1 | Store a hash of (today's date + todo text) in `notifiedTodoObjects.json`; skip if seen. |
| Suppression via saved filter | P1 | See [§7.3](07-search.md#73-saved-search-filters). |
| Overdue badges in UI | P1 | Red dot on overdue due chips; drawer counts and sections turn red. |
| Dock badge count (macOS) | P2 | Number of todos due within the threshold. |
| System tray | P2 | Toggle; tray menu shows "show floetask", file list to switch, quit. Options: invert tray icon colour, start minimised to tray. Closing the window keeps the app in the tray when enabled. |

**iced notes:** `notify-rust` for notifications; a periodic `iced::time::every` subscription re-checks due dates (e.g. on file change and every few minutes / at midnight). Tray via `tray-icon` (0.26), which needs its own event loop integration: on Linux it requires a GTK loop, so run it on a dedicated thread and forward events into iced through a subscription channel. Window hide/show via `iced::window` tasks. Dock badge has no cross-platform crate; mark macOS-only.

---

[Back to the overview](../FEATURES.md)
