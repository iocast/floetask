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
| Move a todo to another file | P1 | Drag a todo from the list, board or calendar onto a file in the file drawer, or choose **Move to file…** in its right-click menu and pick the file in the dialog's dropdown. See [§2.4](#24-moving-a-todo-to-another-file). |
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
- The drawer's *Filters* tab has a switch per file to list it in the overview or leave it out (*Files in the overview*, shown while the overview is). At least one file stays listed: its switch is disabled. A left-out file stays in the file drawer, can be opened on its own, and is still offered as a move target and in the add dialog's file picker. Persisted in the state file (`overview_hidden`, by path).
- List: one collapsible group per file, headed by its name, folder and count. Search, filters, view toggles and sorting apply inside each group; files with nothing to show are left out.
- Board: one board per file, stacked like swimlanes, each with that file's columns. The group switch and the columns button are hidden; columns are edited from the file itself.
- Calendar: every file's todos on one calendar; full cards name their file.
- Every action on a todo (complete, edit, dates, notes, archive, delete, drag) writes to the file the todo is in. Its notes live next to that file.
- The add dialog has a file picker for new todos, starting on the active file (the one last selected); the edit dialog names the todo's file. Archiving all completed todos needs a file selected.
- The drawer counts every listed file. Renaming or removing a project or context changes every listed file, and the dialog says so.
- Selecting or opening a file leaves the overview.

## 2.4 Moving a todo to another file

- The todo line is removed from its file and appended, unchanged, to the end of the other file. The other file is written first, so an interruption can at worst leave the todo in both files.
- Targets are the files the file drawer lists, except the todo's own file. Archive (done) files of another file are not offered; **Move to done file** covers them.
- **Move to file…** opens a dialog with the todo line and a dropdown of the targets in drawer order, the first one picked; a name several files share shows its folder. **Move** (or `Enter`) moves it, **Cancel** or `Escape` closes the dialog. With no other file open, the menu has no **Move to file…**.
- A dragged todo follows the cursor as a small card and outlines the file under the cursor; releasing it there moves it. Released on its own file or outside the drawer, nothing moves. `Escape` cancels the drag.
- **Notes:** a note in the todo's notes folder (`todo-notes/`) moves to the other file's notes folder (`work-notes/`), as on archiving ([`note:` extension](note-extension.md)): copied instead when a todo left behind still links to it, renamed (`-2`, ...) when the other folder holds a different file under that name. A note linked by its absolute path is not in a notes folder and stays where it is, as do missing and invalid notes. When both files share a notes folder (`todo.txt` and `todo.md`), notes stay put.
- A toast names the file the todo moved to.

**iced notes:** use `notify` (8.x) + `notify-debouncer-full`, wrapped as an iced `Subscription` (`Subscription::run` with a channel stream) that emits `Message::FileChanged(path)`. All file I/O runs in `Task::perform` so the UI never blocks.

---

[Back to the overview](../FEATURES.md)
