# 19\. Sync and shared files (P2)

Keeps a local todo file in step with a **shared copy** somewhere else, so one person's devices, or several people working on the same project, see the same todos. The first store is a folder that another tool syncs (the OneDrive app, Dropbox, Syncthing, a network share); cloud stores (S3, Alibaba OSS, Azure Blob, the OneDrive API, a hosted floetask service) follow later and plug into the same sync engine.

## 19\.1 Concept

Every linked file has three copies:

| Copy | What it is | Where |
| --- | --- | --- |
| **Local** | The todo file floetask opens and edits, where the user keeps it | Anywhere, e.g. `C:\projects\acme\todo.txt` |
| **Shared** | The copy every participant links to | In a **remote store**, e.g. `C:\Users\me\OneDrive - Company\acme\acme.txt` |
| **Base** | The content of both at the last successful sync | floetask's state directory |

- The local file stays the working copy: every edit, view and the file watcher work on it as today. Files that are not linked behave exactly as today.
- The **sync engine** compares local, shared and base. Whatever changed on one side only is copied to the other. When both sides changed, it merges them line by line (§19.4) and writes the result to both.
- A **remote store** is where shared copies live. To the engine, every store looks the same: it lists files, reads a file with its **version**, and writes a file **only if it is still at the version read**, so a write never silently replaces someone else's change.
- Sharing with a colleague is the same as syncing two devices: both link their own local file to the same shared copy. With the folder store, the OneDrive app (or Dropbox, Syncthing) moves the shared copy between their computers, and each person's floetask merges it into their own local file:

```
C:\projects\acme\todo.txt         ◄─floetask─►  C:\Users\me\OneDrive - Company\acme\acme.txt
                                                         ▲
                                                    OneDrive app
                                                         ▼
D:\work\acme\todo.txt             ◄─floetask─►  C:\Users\colleague\OneDrive - Company\acme\acme.txt
```

- Only todo files and their notes are synced. Settings, saved filters, colours, layout and notification history stay personal.
- floetask manages no accounts or permissions: who can see the shared copy is decided where it lives (the OneDrive share, the bucket policy).

## 19\.2 Linking a file

| Feature | Pri | Behaviour |
| --- | --- | --- |
| Share a file | P2 | **Share…** in a file's ⋮ menu in the file drawer opens the share dialog. **Choose folder…** puts the shared copy in a folder, named like the file with `-share` added (`todo.txt` becomes `todo-share.txt`); the name can be changed. **Choose file…** makes an existing `.txt` file the shared copy. When the shared copy does not exist, **Share** copies the local file there. When it exists, the dialog says so and the button becomes **Join** (below); an empty one is filled with the local file. The shared copy cannot be the file itself. |
| Join a shared file | P2 | **Join shared file…** under **Open** and **Create** in the file drawer opens a wizard of three steps, numbered at the top, with **Back**, **Cancel** and **Next** (Enter), Next enabled once the step is complete. **1 Shared copy:** says where shared copies are found (a folder OneDrive, Dropbox, Syncthing or a network share keeps in step); **Choose shared file…** picks it. **2 Your copy:** says floetask works on a copy of its own; suggests the shared copy's name next to the active file, until the user chooses one with **New file…** or **Use an existing file…**. Below, what joining will do: create the file, or, when it exists, merge it as if the base were empty (every differing line is a conflict, §19.5). Your copy cannot be the shared copy itself. **3 Review:** both paths and what joining does; **Join** links them, and the file is registered like an opened file. A missing local file becomes a copy of the shared one. |
| Sync status | P2 | Each linked file shows a small icon in the file drawer: in sync, syncing, offline (the store cannot be reached), conflict, shared copy missing. Hovering it gives the status and the shared copy's location. Clicking it syncs now, opens the conflict dialog, or shares the file again when the shared copy is missing. |
| Sync now | P2 | In the ⋮ menu of a linked file. |
| Share again | P2 | In the ⋮ menu of a linked file whose shared copy is missing: copies the local file there under the old name. |
| Stop sharing | P2 | In the ⋮ menu of a linked file. Removes the link and its base copies; the local file and the shared copy stay as they are. |

A file can be linked to one shared copy. A done (archive) file is linked separately, if wanted.

## 19\.3 When sync runs

- After every save of a linked file, once the write finished.
- When the file watcher reports a change to the local file or, for the folder store, to the shared copy or its folder (§19.6).
- Every `sync.interval` seconds while floetask runs (Settings → Sync, 10 to 600, default 30), for stores that cannot report changes.
- On start, for every linked file, and from **Sync now**.
- Syncs of one file never overlap; a sync requested while one runs starts again when it finished.
- When the store cannot be reached, the file is marked offline and edits stay local; the next sync merges them.

## 19\.4 Merging

The engine runs a three-way merge of base, local and shared, with one todo line as the unit (a multi-line todo is one file line, §1.4):

| Base → local | Base → shared | Result |
| --- | --- | --- |
| unchanged | unchanged | unchanged |
| changed | unchanged | local version |
| unchanged | changed | shared version |
| same change on both sides | | that version, once |
| added | | added, at its position relative to its neighbours |
| | added | added, at its position relative to its neighbours |
| removed | unchanged | removed |
| unchanged | removed | removed |
| changed differently on both sides | | **conflict** (§19.5) |
| changed | removed (or the other way round) | **conflict** |
| added at the same place on both sides | | both, local lines first; a line both added comes once |

- Changes to neighbouring lines do not conflict: one side completing a todo while the other edits the next one merges cleanly.
- Where both sides edited the same lines in place, only the lines both changed differently conflict; the others take whichever side changed them.

- Lines that neither side touched stay byte for byte as they are, including line endings and a missing final newline (round-trip rules, §1.3).
- The merge is pure: it lives in the domain and takes the three texts and returns the result, so worked examples like those below are acceptance tests.
- Archiving on one device removes the todo from the todo file; it merges like any removed line. When the other side edited that todo meanwhile, it is a conflict.

**Example: no conflict.** Base:

```
(A) Call dentist
Buy milk
```

The laptop completes "Buy milk"; the phone (or a colleague) adds "Pay rent due:2026-11-10". After both synced, both have:

```
(A) Call dentist
x 2026-11-07 Buy milk
Pay rent due:2026-11-10
```

**Example: conflict.** From the same base, one side writes `(B) Call dentist`, the other `(A) Call dentist +health`. That line is a conflict; the "Buy milk" line merges normally.

## 19\.5 Conflicts

- Everything that merges cleanly is applied right away. For each conflicting line, the local file keeps the local version, the file's status shows **conflict**, and the file is not pushed to the shared copy until every conflict is settled, so the other side's version is not lost. A toast names the file the first time.
- Conflicts are not stored: the base copy and the shared copy stay as they were, so every following sync finds the same conflicts again.
- Clicking the status opens the conflict dialog: per file (the todo file, each note) the conflicting lines, **Mine** and **Theirs** side by side with the words that differ highlighted, and **Keep mine**, **Use theirs** or **Keep both** (default: keep mine). A removed side shows as "removed". **Apply** writes the result to both sides and syncs the rest. When either side changed since the dialog opened, Apply refuses, the file is synced again and the conflicts can be checked again.
- Closing the dialog leaves the conflicts open; the dialog can be reopened from the status at any time.

## 19\.6 Folder store

The shared copy is a file in a folder that another tool syncs, or that several computers reach (a network share).

- **Version** is a hash of the content.
- **Writing only if unchanged:** the store reads the shared copy again just before writing and writes only if its hash still matches; otherwise the engine merges again and retries. The write goes through safe writes (§2), so the sync tool never picks up a half-written file. The check and the rename are not one atomic step; a change slipping in between is caught by the next sync because the base copy is kept.
- **Conflict copies:** when two computers push before the sync tool passed the change on, the tool keeps both and names one by its own pattern, e.g. `acme-DESKTOP-AB12.txt` (OneDrive: the computer's name, upper case, so `acme-old.txt` is never taken for one), `acme (conflicted copy 2026-11-07).txt` (Dropbox, Nextcloud), `acme.sync-conflict-20261107-101500-ABCD.txt` (Syncthing). The store finds these next to the shared copy of the todo file, the engine merges each into the shared copy like one more side, and the store deletes it once merged. A copy that would conflict stays where it is and a toast names it, to be merged by hand.
- Safe-write leftovers (`.tmp`, `.bak`) in the shared folder are ignored.
- **The shared copy disappears or is renamed:** the local file is never deleted. The file's status shows that the shared copy is missing, with **Share again** (push the local file to the old name) and **Stop sharing**.
- **Files On-Demand:** a cloud-only shared copy is downloaded when read; that download is not a change.

## 19\.7 Notes

- The notes the todos link to by name are synced: a note in the local file's notes folder (`todo-notes/`) with the note of the same name in the shared copy's notes folder (`acme-notes/` next to `acme.txt`), using the same three-way merge on Markdown lines and the same conflict dialog.
- A new note on either side is copied to the other. A sync never deletes a note.
- A note open in the note window or the edit dialog is left alone until it is closed, so the editor never writes over a merge.
- Notes linked by an absolute path are not synced.

## 19\.8 Stored data

| File | Holds |
| --- | --- |
| `~/.local/state/floetask/sync.toml` | One `[[link]]` per linked file: local path, name of the shared copy, store kind and location (e.g. the folder) |
| `~/.local/state/floetask/sync/` | Base copies: one folder per linked file (its name and a hash of its path) with `file.txt` and `notes/` |
| `config.toml` | `[sync]` settings: `interval` (seconds). Store credentials (later stores) go to the OS keychain, never to a file. |

Losing the base copies is safe: the next sync treats every differing line as a conflict instead of guessing.

## 19\.9 Implementation

Dependencies keep pointing inward (see `AGENTS.md`).

**New:**

- **Domain** (`merge.rs`): `merge3(base, local, shared) -> Merge`, the merged text with the conflicts in between (local and shared version), `merge2(local, shared)` for files without a base (every difference a conflict), `Merge::resolve(choices)`, and `changed_words` for the dialog's highlighting. The line diff is Myers' algorithm. Unit and property tests: merging a side with itself changes nothing, a change on one side only always comes through, the diff finds a longest common subsequence; the examples in §19.4 are tests.
- **Application:**
  - `RemoteStore` port: `get(name) -> Option<(content, Version)>`, `put(name, content, expected: Option<&Version>) -> Version` failing with `AppError::Changed` (with `None`: only if it does not exist), `conflict_copies(name)`, `remove(name)` (conflict copies), `watch_path()` (a folder the file watcher can follow). `RemoteStores` opens the store of a `StoreLocation`.
  - `SyncLinkStore` port: loads and saves the links and base copies (§19.8).
  - `SyncService`: `share`, `join`, `sync(file, skip)`, `conflicts(file)`, `resolve(file, choices, skip)`, `unlink`, `watch_paths`. It reads local and shared, merges, writes the local file only over what it read, then pushes with the version read, and when either side changed meanwhile reads, merges and tries again (three times). Syncs never overlap.
  - `SyncSettings` in `Settings` (`interval`), with `#[serde(default)]`.
- **Infrastructure:** `FolderRemoteStore` and `LocalStores` (`remote.rs`, §19.6), `TomlSyncLinkStore` (`stores/sync_links.rs`), the conflict-copy patterns of OneDrive, Dropbox, Nextcloud and Syncthing. An end-to-end test plays two people sharing a folder.
- **GUI:** a `sync/` feature folder: the share dialog, the join wizard, the status icon in the file drawer, the conflict dialog, Settings → Sync, and its messages. Every text goes through `i18n` in all locales.
- **Composition root:** `ports()` builds the stores; `Services` holds `SyncService`.

**Changed:**

- **`FileSystem` port and `LocalFileSystem`** (`fs.rs`): a conditional write, `write_if_unchanged(path, content, expected, safe)`, which writes only if the file still holds the content it was read with and fails with `AppError::Changed` otherwise. The safe write checks just before its final rename; `safe_write` itself behaves as before. The port's default implementation checks just before writing, so test fakes need nothing new.
- **`TodoFileService`** (`todo_files.rs`): every change writes with `write_if_unchanged`, passing the content it loaded; on "changed" it loads again, applies the edit again and retries once. Without this, an edit that read the file before a sync wrote its merge would overwrite the merge. Archiving and moving a todo to another file refuse to overwrite a changed file instead (the moved todos are then in both files, never lost). Public functions stay the same.
- **`TodoDocument::locate`** (domain): an edit finds its todo by line and raw text; when lines above it were added or removed (by a sync, another app), it is still found by its text as long as no other todo has the same text.
- **`NoteService`** stays as it is: instead of conditional writes, the sync leaves notes open in an editor alone (§19.7).
- **`FileWatcher`** (`watcher.rs`): a watched folder reports a change of any file directly in it under the folder's path, so changes of shared copies and new conflict copies are seen.
- **GUI file flow** (`files/`): starts a sync after a linked file was saved and when the watcher reports a change of a linked file or of a shared copy's folder. Its own subscription drives the interval. A sync that changes the local file reloads it.

The todo.txt format, parsing and unlinked files do not change. Paths stay as they are: the local file is still the file floetask works on, so ports and views keep using `Path`.

## 19\.10 Later

- **Cloud stores** through the `object_store` crate: AWS S3 and S3-compatible stores (Alibaba OSS, MinIO, Cloudflare R2), Azure Blob, Google Cloud Storage. Version is the ETag; writing only if unchanged uses `If-Match` where the store supports it, and falls back to checking just before the write where it does not (to be tested per store). Changes are found by polling (§19.3).
- **OneDrive and Google Drive APIs**, for devices without the desktop app: their own stores, with browser sign-in (OAuth) and the token in the OS keychain.
- **Hosted floetask service:** a server that reuses the domain and application crates, is one more `RemoteStore`, signs users in (OpenID Connect) and pushes changes instead of being polled. A web app (iced on WebAssembly, or a separate front end) builds on it.

## 19\.11 Not planned

- Live co-editing (seeing another person type). Changes arrive with the next sync.
- Syncing settings, saved filters, colours or layout.
- Managing users, sharing or permissions inside floetask.

___

[Back to the overview](../FEATURES.md)
