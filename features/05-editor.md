# 5. Add / edit dialog (P0)

- Opened by `Ctrl+N`, the add button, or opening a row.
- Multi-line text field holding the raw todo.txt line. `Ctrl+Enter` saves, `Escape` closes.
- **Autocomplete**: typing `+` or `@` shows suggestions from all known projects/contexts (including from hidden `h:1` todos). `Up`/`Down` to move, `Enter` to insert.
- **Pickers** that edit the text in place:
  - Priority picker (none, A–Z).
  - Due date picker.
  - Threshold date picker.
  - Recurrence picker (unit + count + strict toggle).
  - Pomodoro picker (number).
- **Notes** (P1): a Markdown field for the todo's notes. Saving writes them to the `note:` file, creating the file and the tag from the task text when the todo has none (see [note-extension.md](note-extension.md)).
- **Bulk creation** (setting): each line in the field becomes a separate todo; the Add button shows the count, e.g. `Add (3)`.
- Empty input shows a non-blocking warning (toast).
- Creating a todo from the search box: `Ctrl+Enter` in the search field saves its text as a new todo.

**iced notes:** use `text_editor` for the multi-line field. iced has no built-in autocomplete or date picker: autocomplete is a custom overlay anchored to the cursor position (track the token at the cursor from `text_editor::Content`), and the date picker can come from `iced_aw` (check 0.14 compatibility) or be a small custom calendar widget. Dialogs are modal overlays built with `stack` + `opaque` + `mouse_area`.

---

[Back to the overview](../FEATURES.md)
