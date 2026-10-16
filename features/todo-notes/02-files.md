# 2. Files (P0 / P1)

| Feature | Pri | Behaviour |
|---|---|---|
| Open existing todo.txt | P0 | File picker (`rfd`), `Ctrl+O`. |
| Create new todo.txt | P0 | Save dialog, creates an empty file. |
| Drag and drop a file onto the window | P1 | Opens it. iced exposes `window::Event::FileDropped`. |
| Multiple files | P1 | Several registered files, one active at a time. |
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

**iced notes:** use `notify` (8.x) + `notify-debouncer-full`, wrapped as an iced `Subscription` (`Subscription::run` with a channel stream) that emits `Message::FileChanged(path)`. All file I/O runs in `Task::perform` so the UI never blocks.

---

[Back to the overview](../FEATURES.md)
