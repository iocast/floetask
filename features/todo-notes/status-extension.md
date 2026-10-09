# `status:` extension for todo.txt (informal spec)

**Purpose:** Express the workflow state of an open task (for example "in progress" or "waiting"), since the todo.txt format only distinguishes open from done.

## Syntax

```
status:<value>
```

- A standard `key:value` pair, placed anywhere after the first word of the description.
- `<value>` is a single lowercase token with no spaces. Allowed characters: `[a-z0-9_-]`.
- At most one `status:` per task.

## Defined values

| Value     | Meaning                                              |
|-----------|------------------------------------------------------|
| `todo`    | Not started. Same as having no `status:` at all.     |
| `doing`   | Actively being worked on.                            |
| `waiting` | Blocked on someone or something external.            |
| `someday` | Parked, not committed to. Hide from the default list.|

The four values above are the defaults. See "Custom statuses" below for adding more.

## Custom statuses

The set of statuses is open. Users and tools may define additional values to fit their workflow, for example `review`, `blocked`, `testing` or `delegated`.

- **Format:** Custom values follow the same rules as defined values: a single lowercase token using `[a-z0-9_-]`, no spaces.
- **Preservation:** Tools must preserve unknown values unchanged when reading and writing a line. A tool must never strip or rewrite a status it doesn't recognize.
- **Treated as open:** An unrecognized status is handled as an open task. It is shown in the default list unless the tool or user configuration says otherwise.
- **Configuration (optional):** Tools may let users declare their own status list, along with its display order and which statuses are hidden by default (as `someday` is).
- **Naming:** Prefer short, lowercase, descriptive names. Use `-` or `_` to join words, for example `in-review`.
- **Avoid collisions:** Do not redefine the meaning of the four default values. If a workflow needs a different meaning, choose a new name.
- **Portability:** Custom values only carry meaning in tools or scripts that know about them. Other tools will still display the line correctly, as plain text.

Example:

```
Review pull request +work status:in-review
Ship release notes +work status:testing
Order replacement part status:delegated who:bob
```

## Relationship to the core format

- **No `status:`** means `todo`.
- **Done is not a status.** Completion is expressed only by the leading `x ` and completion date. Do not write `status:done`.
- **Priority is independent.** `(A)` describes importance, `status:` describes state. Both can appear on the same line.

## Examples

```
(A) Write report +work @office due:2026-10-10 status:doing
Waiting for legal review +work status:waiting
Learn Rust +learning status:someday
```

## Behavior (for tools)

- **Set/change:** Replace the existing `status:` token in place. Never add a second one.
- **Set to `todo`:** Remove the token rather than writing `status:todo`.
- **Complete (`x`):** Remove `status:` when marking a task done. If a tool keeps it, it must be ignored for filtering.
- **Reopen:** A task restored from `done.txt` returns with no `status:`, which means `todo`.
- **Listing:** Show `todo` and `doing` by default. Hide `someday` unless requested. Show `waiting` but may style it distinctly.
- **Sorting (optional):** `doing` first, then `todo`, then `waiting`, then `someday`.
- **Filtering:** Matching `status:doing` is a plain text search, so it works with existing tools such as `todo.sh list status:doing`.

## Optional conventions

- **Waiting detail:** Pair `status:waiting` with a free-text reason or a `who:` tag, for example `status:waiting who:alice`.
- **Timing:** Combine `status:someday` with a threshold date `t:` to resurface the task later.

## Status

This is not part of the official todo.txt spec. Tools must explicitly support it, or you can handle it with filters, a small script, or an add-on.

---

## How floetask implements it

- **Parsing** (`floetask-domain`, `status.rs` and `todo/`): the first `status:` tag with a valid value is the status; an invalid value such as `status:Doing` is kept as text and the todo counts as `todo`. Completed todos have no status, whatever their line says. `Todo::with_status` replaces the tag in place and removes it for `todo`; completing a todo removes it, and the next occurrence of a recurring todo starts without one.
- **Configuration** (`config.toml`):

  ```toml
  [statuses]
  order = ["doing", "todo", "waiting", "someday"]  # display and sort order; add custom ones here
  hidden = ["someday"]                             # hidden from the default list
  ```

  Built-in statuses missing from `order` are appended, invalid names are dropped, and `todo` can never be hidden. Statuses found in a file but not in `order` sort after the known ones and are always shown.
- **Listing:** the default sort groups by status in the configured order (`doing`, `todo`, `waiting`, then custom statuses), with completed todos last under "Done". Status is a normal sort criterion, so it can be moved down or inverted in the Sorting tab. A sort list saved by an older version gets status inserted at the top.
- **Hidden statuses** (`someday` by default) are shown when the Filters tab toggle is on, when the drawer's Status section includes that value, or when the search contains `status:<value>`.
- **GUI:** the add/edit dialog has a status picker next to the priority. Cards show a chip for every status except `todo`: `doing` in the accent colour with a play icon, `waiting` in amber with an hourglass, others muted. Clicking a chip filters by that status like any other chip.
