# 8. Archiving (P1)

- Archive all completed todos of the active file to its done file (`Ctrl+Alt+A`, navigation button). Appends to done.txt and removes from todo.txt.
- Archive a single todo from the row context menu, whether completed or not.
- If no done file is linked, prompt to choose or create one.
- Optional view of the archive.

Both writes must be done so that a crash cannot lose todos: append to done first (safe write), then rewrite todo.

---

[Back to the overview](../FEATURES.md)
