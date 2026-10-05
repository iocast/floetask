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
notes/
  report-outline.md
  dentist-call.md
```

The `notes/` folder sits next to `todo.txt`. Filenames are resolved relative to it.

## Filename rules

* Recommended extension: `.md` (plain text also fine).
* If the value has no extension, tools should assume `.md`.
* Lowercase, `[a-z0-9._-]` only, for portability.
* Must not contain `/../` or start with `/` (no escaping the notes folder).

## Example

```
(A) Write report +work @office due:2026-10-10 note:report-outline.md
```

Opens `notes/report-outline.md`.

## Behavior (for tools)

* **Add/open:** If the file doesn't exist, create it (optionally with the task text as a heading).
* **Writing notes:** Tools may let the user write the note's Markdown next to the task. When a task without `note:` gets note text, the tool creates the file itself and adds the tag, naming the file after the task text. It must not overwrite an existing file under that name: add `-2`, `-3`, ... before the extension instead.
* **Complete (`x`):** Leave the note file in place; the `note:` tag stays on the line in `done.txt`.
* **Delete task:** Do not auto-delete the note. Optionally warn about orphans.
* **Missing file:** Treat as a broken link, not an error. The task remains valid.
* **Archiving:** Moving a task to `done.txt` doesn't move the note.

## Optional conventions

* One note per task. Share a note across tasks by giving them the same `note:` value.
* Tools may suggest a default name by slugifying the task text, e.g. `write-report.md`.

## Status

This is not part of the official todo.txt spec. Tools must explicitly support it, or you can handle it with a small script or add-on.

## How floetask implements it

* Each todo card with a `note:` shows a note button. Clicking it opens the note in the system's default app, creating `notes/<filename>` with the task text as a `#` heading first if it is missing.
* When the note file does not exist yet, the button shows as a broken link; the todo stays valid and clicking still creates the note.
* An invalid value (uppercase, spaces, `..`, a leading `/`) shows as a broken link and is never opened.
* The add/edit dialog has a Markdown notes field. Editing a todo with a `note:` loads the file's content into it, and saving writes the field back to the file when it changed.
* The user never names or creates the file. Saving a todo without `note:` whose notes field has text creates `notes/<slug>.md` from the task text (e.g. `write-report.md`, or `write-report-2.md` if that name is taken) and adds the tag. With an empty field, no file and no tag are added.
* Clearing the notes field of an existing note empties the file but keeps it and the tag.
* Deleting a todo whose note no other todo in the file uses shows a warning that the note is now orphaned. The note file is never deleted.
* Completing and archiving leave both the tag and the file untouched.
