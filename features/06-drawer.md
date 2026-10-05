# 6. Sidebar drawer: attributes, filters, sorting (P1)

Toggled with `Ctrl+B`; `Escape` closes. Three tabs:

## 6.1 Attributes tab

- One collapsible section per attribute: priority, status, projects, contexts, due, t, rec, pm, created, completed (open/closed state persisted).
- Each value shows a count of todos having it; counts reflect the filtered set; overdue sections show a red indicator.
- Click a value to **include** filter; second action (modifier-click) to **exclude** filter. Multiple values combine (include = must match one of; exclude = must match none).
- Hide a whole category (e.g. hide every todo that has any context).
- Right-click a value: **Rename** or **Remove** across all todos in the active file, with confirmation. Case-sensitive. Not available for dates.
- Option to include attributes from hidden todos in the drawer.

## 6.2 Filters tab (view toggles)

- Show completed todos (`Ctrl+H`).
- Show hidden (`h:1`) todos.
- Show attributes from hidden todos in drawer.
- Show todos with future threshold dates.
- Show todos with future due dates.
- Show todos whose status is hidden by default (`someday`).

## 6.3 Sorting tab

- Ordered list of sort attributes (default order: status, priority, projects, contexts, due, t, completed, created, rec, pm). Reorder by drag and drop.
- Per attribute: invert direction.
- Recurrence sorts by semantic length (1d < 1w < 1m < 1y; relative before strict; missing last).
- Missing values always sort last regardless of direction.
- "File order" toggle and, under it, "sort completed last".

**iced notes:** iced 0.14 has no built-in drag-and-drop list reordering; implement with `mouse_area` + manual drag state, or provide up/down buttons as a P1 fallback and drag as P2. Collapsible sections are simple custom components.

---

[Back to the overview](../FEATURES.md)
