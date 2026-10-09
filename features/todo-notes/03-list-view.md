# 3. Todo list view (P0)

## 3.1 Rows

Each row shows:

- Completion checkbox (`Space` toggles when focused).
- Priority badge with colour bar (A–C coloured, others grey). The badge is hidden when the list is already grouped by priority.
- Body text, rendered with Markdown ([§3.4](#34-markdown-in-descriptions-p1)).
- Attribute chips for projects, contexts, due, t, rec, pm. Clicking a chip toggles it as a filter. Overdue/today due chips are red with a dot.
- Created and completed date indicators (tooltip with the date).
- Inline date picker on the due/t chip to change the date directly.
- Completed todos are visually muted/struck through.

Row interactions:

- Click or `Enter` opens the edit dialog.
- Right-click context menu: **Copy** (raw line to clipboard), **Archive** (move this single todo to the done file, completed or not), **Delete** (with confirmation showing the raw line).
- `Delete` or `Ctrl+Backspace` deletes the selected row (with confirmation).
- `Up`/`Down` moves row focus; `Left`/`Right` moves between buttons inside a row.

## 3.2 Grouping

- The list is grouped by the **first** active sort attribute (e.g. by priority, then sorted within groups by the rest).
- Each group has a header with its value (attribute values in headers are right-clickable for rename/remove, [§6](06-drawer.md#6-sidebar-drawer-attributes-filters-sorting-p1)).
- Groups whose todos are all hidden are not shown.
- "File order" mode (`fileSorting`) disables grouping and shows todos in file order; in that mode an optional "sort completed last" applies.

**Collapsing groups:** clicking a group header (chevron, name and count) collapses the group to its header; clicking again expands it. Keyboard selection skips the rows of collapsed groups. Collapsed groups are remembered in `state.toml` by attribute and value (e.g. `status:todo`), separately from the board's.

## 3.3 Header counts

Show visible / total counts and completed count.

## 3.4 Markdown in descriptions (P1)

- Bold, italic, strikethrough, inline code, lists, blockquotes, headings, tables, links.
- Bare URLs are linkified.
- Links support `http(s)://`, `file://` and custom schemes (`joplin://`, `cbthunderlink://`, …), opened via the OS handler (`open` crate) through an explicit open icon.

**iced notes:** iced 0.14 ships a `markdown` widget (`iced::widget::markdown`, pulldown-cmark based) that covers most of this; table support needs verifying. Link clicks arrive as messages, which keeps link opening explicit. Rendering hundreds of markdown rows can be heavy: parse once per todo into `markdown::Item`s, cache in state, and use `lazy` or a virtualised list for large files.

## 3.5 Empty states (P0)

Splash screens for: no file open (open/create buttons), file has no todos (add button), and no todos visible because of filters (reset filters button).

## 3.6 Layout options (P1)

- Compact mode (denser rows), toggle in settings and View menu. On the status board and the calendar's week and day views it also tightens the cards and puts the attributes under the checkbox, at the card's full width.
- Zoom 50–150 % in 10 % steps. iced: apply via the application `scale_factor`.
- Disable animations toggle (relevant only if floetask adds animations).
- The main view (list, board, calendar) is never narrower than 760 px. When the window is too narrow for it and the open side panels, panels hide in this order until it fits: the filter drawer, then the file drawer (then the calendar's undated panel). They come back when the window grows; their open or closed setting is kept.
- Clicking the button of a hidden panel (or opening a closed one while there is no room) shows it over the main view on its own side instead of squeezing the view. A click beside it, the button again or `Escape` hides it.

## 3.7 Status board (P1)

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

[Back to the overview](../FEATURES.md)
