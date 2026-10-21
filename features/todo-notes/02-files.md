# 2. Files (P0 / P1)

| Feature | Pri | Behaviour |
|---|---|---|
| Open existing todo.txt | P0 | File picker (`rfd`), `Ctrl+O`. |
| Create new todo.txt | P0 | Save dialog, creates an empty file. |
| Drag and drop a file onto the window | P1 | Opens it. iced exposes `window::Event::FileDropped`. |
| Multiple files | P1 | Several registered files, one active at a time. |
| Overview of all files | P1 | With two files or more, an *Overview* entry tops the file drawer: the todos of every file in one view, grouped by file. Persisted in the state file (`overview`). See [§2.3](#23-overview). |
| File tabs | P1 | Tab bar to switch files; toggle tabs visibility (View menu). |
| Switch file by shortcut | P1 | `Ctrl+1`…`Ctrl+9`, in the order the file drawer shows. |
| Sort the file drawer | P1 | Sort menu next to the drawer's *Files* caption: custom order (default), name, or folder path (then name). Persisted in the state file (`file_sort`). See [§2.2](#22-file-drawer-order). |
| Drag files into a custom order | P1 | Drag a file onto another one to take its place; a line shows where it lands. Dragging while sorted by name or folder switches to the custom order, starting from the order shown. |
| Tab context menu | P1 | Change done-file location, reveal todo file, reveal done file (open in OS file manager), remove from list (with confirmation). |
| Done file per todo file | P1 | Each todo file can link a `done.txt` used for archiving. |
| File watcher | P0 | External changes reload the list automatically (see [§2.1](#21-file-watching)). |
| Safe writes | P1 | Write to `.tmp`, verify, create `.bak`, atomically replace, delete both on success. Toggle in settings, default on. |
| Remember files and active file | P0 | Persisted in config. |

## 2.1 File watching

- Watch all registered todo files and the saved-filters file.
- Debounce events until the file has been stable for about 100 ms so sync tools (Syncthing, Dropbox) do not cause flicker or false deletions.
- Watcher options are configurable (e.g. debounce ms, polling on/off, poll interval).
- Ignore events caused by the app's own writes.

## 2.2 File drawer order

- An archive (done) file always belongs to its todo file: it is listed right under it, never sorts on its own, and moves with it when the todo file is dragged. This holds when the archive file is also open as a registered file.
- The custom order is the order of the registered files in the state file, so switching to name and back keeps it.
- Pressing and releasing a file without moving off it selects it, as a click; releasing outside the file list, or `Escape`, cancels the drag.

## 2.3 Overview

- Lists the files the drawer lists, in its order; archive files of another file are left out.
- List: one collapsible group per file, headed by its name, folder and count. Search, filters, view toggles and sorting apply inside each group; files with nothing to show are left out.
- Board: one board per file, stacked like swimlanes, each with that file's columns. The group switch and the columns button are hidden; columns are edited from the file itself.
- Calendar: every file's todos on one calendar; full cards name their file.
- Every action on a todo (complete, edit, dates, notes, archive, delete, drag) writes to the file the todo is in. Its notes live next to that file.
- The add dialog has a file picker for new todos, starting on the active file (the one last selected); the edit dialog names the todo's file. Archiving all completed todos needs a file selected.
- The drawer counts every file. Renaming or removing a project or context changes every file, and the dialog says so.
- Selecting or opening a file leaves the overview.

**iced notes:** use `notify` (8.x) + `notify-debouncer-full`, wrapped as an iced `Subscription` (`Subscription::run` with a channel stream) that emits `Message::FileChanged(path)`. All file I/O runs in `Task::perform` so the UI never blocks.

---

[Back to the overview](../FEATURES.md)
