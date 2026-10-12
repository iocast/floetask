# `note:` extension for todo.txt (informal spec)

**Purpose:** Link a todo.txt task to a separate notes file, since the format itself is one line per task.

## Syntax

```
note:<filename>
```

* A standard `key:value` pair, placed anywhere after the first word of the description.
* `<filename>` is a relative path inside the notes folder, with no spaces (use `-` or `_`).
* At most one `note:` per task.

### Notes outside the notes folder

```
note:<absolute path, percent-encoded>
```

A note can also stay wherever the user keeps it, linked by its absolute path: `/home/me/notes/plan.md`, `C:/Users/me/notes/plan.md`, or `//server/share/plan.md`. Separators are `/`. A todo.txt value cannot contain spaces or `:` and must not start with `//` (that reads as a URL), so the path is percent-encoded:

| Character | Written as |
|---|---|
| `%` | `%25` |
| `:` | `%3A` |
| space (any whitespace, as UTF-8 bytes) | `%20` |
| the first `/` of `//server/...` | `%2F` |

```
Plan offsite +work note:C%3A/Users/me/My%20Notes/plan.md
Ask IT note:%2F/fileserver/team/it-request.md
```

A value is an external path when, once decoded, it starts with `/` or a drive (`C:/`); anything else is a name in the notes folder and follows the filename rules below. External notes never move: archiving leaves them where they are and keeps the link.

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
* Must not contain `/../` (no escaping the notes folder). A leading `/` makes the value an external path (see above).

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

* Notes are edited in a WYSIWYG Markdown editor ([iced-markdown-editor](https://crates.io/crates/iced-markdown-editor), themed with the app's colours) with a toolbar and a Source mode for the raw text. It saves Markdown in its own canonical form (e.g. a blank line between blocks), so a note is only rewritten after the user changes it.
* **Note window:** each todo card with a `note:` shows a note chip (an external note shows its file name; the tooltip and window show the path). Clicking it opens the note in floetask's note window, creating `<filename>-notes/<note>` with the task text as a `#` heading first if it is missing. There is one note window: opening another note shows it in the same window, after saving the one shown before. The window saves once typing pauses, on `Ctrl+S`, and when it closes or the app quits; its header shows whether the note is saved and has a button to open the note in the system's default app. It opens where it was last moved and resized to (kept in `state.toml`).
* When the note file does not exist yet, the chip shows as a broken link; the todo stays valid and clicking still creates the note.
* An invalid value (uppercase, spaces, `..`) shows as a broken link and is never opened.
* The add/edit dialog has the editor for the note. Editing a todo with a `note:` loads the file's content into it, and saving writes the field back to the file when it changed.
* **Opening the dialog's note in the window:** a button in the dialog's notes header opens the note in the note window, taking along what was typed and not saved yet. A todo without `note:` first gets a new note named after its text, as saving would. While the window shows the note, the dialog shows "open in its own window" instead of the editor and does not write the note on save; when the window closes, the dialog shows the note as saved there.
* **Linking instead of creating:** a button in the dialog's notes header picks a Markdown or text file on disk. A file already in the notes folder is linked by its name. Any other file prompts a choice: **Move it** to the notes folder, under a valid name that no other note uses (`My Plan.md` becomes `my-plan.md`, the original is removed once the copy is written), or **Keep it where it is**, linked by its absolute path. Either way the todo's `note:` is set and the editor shows that note in place of the text there.
* **Unlinking:** while the todo has a `note:`, a button in the notes header removes the tag and empties the editor. The file is kept.
* The user never names or creates the file. Saving a todo without `note:` whose notes field has text creates `<filename>-notes/<slug>.md` from the task text (e.g. `write-report.md`, or `write-report-2.md` if that name is taken) and adds the tag. With an empty field, no file and no tag are added.
* Clearing the notes field of an existing note empties the file but keeps it and the tag.
* Deleting a todo whose note no other todo in the file uses shows a warning that the note is now orphaned. The note file is never deleted.
* Completing leaves both the tag and the file untouched.
* Archiving moves notes to the done file's notes folder as described above. Notes are copied before either todo file is written and the originals are removed last, so an interruption can at worst leave a note in both folders.
