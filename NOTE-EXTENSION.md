# `note:` extension for todo.txt (informal spec)

**Purpose:** Link a todo.txt task to a separate notes file, since the format itself is one line per task.

## Syntax

```
note:<filename>
```

* A standard `key:value` pair, placed anywhere after the first word of the description.
* `<filename>` is a relative path inside the notes folder, with no spaces (use `-` or `_`).
* At most one `note:` per task.

## Location

```
todo.txt
done.txt
todo-notes/
  report-outline.md
  dentist-call.md
```

Each todo file has its own notes folder next to it, named `<filename>-notes`, where `<filename>` is the todo file's name without its extension: `todo.txt` uses `todo-notes/`, `work.txt` uses `work-notes/`. Filenames in `note:` are resolved relative to that folder, so two todo files never share notes.

When a task moves to another todo file, such as `done.txt` on archiving, its note moves with it into that file's notes folder (`todo-notes/` to `done-notes/`), so the link keeps working.

## Filename rules

* Recommended extension: `.md` (plain text also fine).
* If the value has no extension, tools should assume `.md`.
* Lowercase, `[a-z0-9._-]` only, for portability.
* Must not contain `/../` or start with `/` (no escaping the notes folder).

## Example

```
(A) Write report +work @office due:2026-10-10 note:report-outline.md
```

In `todo.txt`, this opens `todo-notes/report-outline.md`.

## Behavior (for tools)

* **Add/open:** If the file doesn't exist, create it (optionally with the task text as a heading).
* **Writing notes:** Tools may let the user write the note's Markdown next to the task. When a task without `note:` gets note text, the tool creates the file itself and adds the tag, naming the file after the task text. It must not overwrite an existing file under that name: add `-2`, `-3`, ... before the extension instead.
* **Complete (`x`):** Leave the note file in place and keep the `note:` tag on the line.
* **Delete task:** Do not auto-delete the note. Optionally warn about orphans.
* **Missing file:** Treat as a broken link, not an error. The task remains valid.
* **Archiving:** Moving a task to `done.txt` moves its note from `todo-notes/` to `done-notes/`. If a task left in `todo.txt` still links to the same note, the note is copied instead, so both links keep working (the two copies are then edited separately). If `done-notes/` already holds a different file under that name, the note gets a free name (`-2`, `-3`, ... before the extension) and the archived line's `note:` is updated to match. A missing or invalid note is left alone.

## Optional conventions

* One note per task. Share a note across tasks by giving them the same `note:` value.
* Tools may suggest a default name by slugifying the task text, e.g. `write-report.md`.

## Status

This is not part of the official todo.txt spec. Tools must explicitly support it, or you can handle it with a small script or add-on.

## How floetask implements it

* Each todo card with a `note:` shows a note button. Clicking it opens the note in the system's default app, creating `<filename>-notes/<note>` with the task text as a `#` heading first if it is missing.
* When the note file does not exist yet, the button shows as a broken link; the todo stays valid and clicking still creates the note.
* An invalid value (uppercase, spaces, `..`, a leading `/`) shows as a broken link and is never opened.
* The add/edit dialog has a Markdown notes field. Editing a todo with a `note:` loads the file's content into it, and saving writes the field back to the file when it changed.
* The user never names or creates the file. Saving a todo without `note:` whose notes field has text creates `<filename>-notes/<slug>.md` from the task text (e.g. `write-report.md`, or `write-report-2.md` if that name is taken) and adds the tag. With an empty field, no file and no tag are added.
* Clearing the notes field of an existing note empties the file but keeps it and the tag.
* Deleting a todo whose note no other todo in the file uses shows a warning that the note is now orphaned. The note file is never deleted.
* Completing leaves both the tag and the file untouched.
* Archiving moves notes to the done file's notes folder as described above. Notes are copied before either todo file is written and the originals are removed last, so an interruption can at worst leave a note in both folders.
