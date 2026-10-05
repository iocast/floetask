# 2. Files (P0 / P1)

| Feature | Pri | Behaviour |
|---|---|---|
| Open existing todo.txt | P0 | File picker (`rfd`), `Ctrl+O`. |
| Create new todo.txt | P0 | Save dialog, creates an empty file. |
| Drag and drop a file onto the window | P1 | Opens it. iced exposes `window::Event::FileDropped`. |
| Multiple files | P1 | Several registered files, one active at a time. |
| File tabs | P1 | Tab bar to switch files; toggle tabs visibility (View menu). |
| Switch file by shortcut | P1 | `Ctrl+1`…`Ctrl+9`. |
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

**iced notes:** use `notify` (8.x) + `notify-debouncer-full`, wrapped as an iced `Subscription` (`Subscription::run` with a channel stream) that emits `Message::FileChanged(path)`. All file I/O runs in `Task::perform` so the UI never blocks.

---

[Back to the overview](../FEATURES.md)
