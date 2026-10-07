# 18. Calendar view (P1)

A third way to show the active file, next to the list and the status board: todos placed on a calendar by their **due date**.

## 18.1 Modes

| Mode | Shows | Step |
|---|---|---|
| **Month** | A grid of whole weeks covering the month, one cell per day. Days of the neighbouring months are muted. | one month |
| **Week** | Seven day columns, starting on the configured first day of the week (Settings → Dates). | one week |
| **Day** | One day with full todo cards. | one day |

- The header shows the period ("October 2026", "5 – 11 Oct 2026", "Tuesday, 6 October 2026"), **previous** / **today** / **next** buttons and the mode switch.
- Today is highlighted in every mode.
- The week view is never narrower than 760 px; its columns shrink with the window down to that. When the window is too narrow for the week and the open side panels, panels hide in this order until it fits: the filter drawer, the file drawer, the undated panel. They come back when the window grows again; their open or closed setting is kept.
- The mode is remembered in `state.toml`; the calendar always opens on today.

## 18.2 What is shown

- The same todos as the list: the search, the drawer filters and the view toggles (completed, hidden, threshold, someday) apply.
- Each todo appears on the day of its `due:` date. Open todos without a due date are not on the calendar; the header says how many there are.
- Clicking that count opens a panel beside the calendar listing those todos as cards. Each card can be dragged onto a day, or given a date with **Set due date**; either sets only its `due:`. Clicking a card opens it.
- Month cells show up to three todos and "+N more"; clicking the day number or "+N more" opens that day in Day mode. Week columns and the Day view list every todo of the day.
- Each entry shows the priority accent and the todo text; completed todos are struck through.

## 18.3 Interaction

- Clicking a todo opens the edit dialog.
- Dragging a todo to another day sets its `due:` to that day; only the `due:` token changes (round-trip rules, §1.3).
- The title bar switches between list, board and calendar; `Ctrl+Alt+C` toggles between the calendar and the list.

---

[Back to the overview](../FEATURES.md)
