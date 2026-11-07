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
| Share a file | P2 | **Share…** in a file's context menu in the file drawer. Pick a store (for the folder store: a folder) and a name for the shared copy, prefilled with the file's name. If no shared copy of that name exists, the local file is copied there. If one exists, the dialog says so and offers **Join** (below) or another name. |
| Join a shared file | P2 | **Join shared file…** next to **Open** in the file drawer. Pick the shared copy and where the local file goes. If the local file exists, both are merged as if the base were empty (every differing line is a conflict, §19.5); otherwise the shared copy is copied to it. The file is registered like an opened file. |
| Sync status | P2 | Each linked file shows a small icon in the file drawer: in sync, syncing, offline (the store cannot be reached), conflict. Hovering it names the shared copy and the time of the last sync. |
| Sync now | P2 | In the context menu of a linked file. |
| Stop sharing | P2 | In the context menu of a linked file. Removes the link and its base copy; the local file and the shared copy stay as they are. |

A file can be linked to one shared copy. A done (archive) file is linked separately, if wanted.

## 19\.3 When sync runs

- After every save of a linked file, once the write finished.
- When the file watcher reports a change to the local file or, for the folder store, to the shared copy or its folder (§19.6).
- Every `sync.interval` seconds while floetask runs (Settings → Sync, default 30), for stores that cannot report changes.
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
| added at the same place on both sides | | both lines, local first |

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

- Everything that merges cleanly is applied right away. For each conflicting line, the local file keeps the local version, the file's status shows **conflict**, and the file is not pushed to the shared copy until every conflict is settled, so the other side's version is not lost.
- Clicking the status (or a toast after the sync) opens the conflict dialog: one entry per conflicting line with **Mine**, **Theirs** and **Keep both**, and the two versions side by side with the differing tokens highlighted. A removed line shows as "removed". **Apply** writes the result and syncs.
- Closing the dialog leaves the conflicts open; the dialog can be reopened from the status at any time.

## 19\.6 Folder store

The shared copy is a file in a folder that another tool syncs, or that several computers reach (a network share).

- **Version** is a hash of the content.
- **Writing only if unchanged:** the store reads the shared copy again just before writing and writes only if its hash still matches; otherwise the engine merges again and retries. The write goes through safe writes (§2), so the sync tool never picks up a half-written file. The check and the rename are not one atomic step; a change slipping in between is caught by the next sync because the base copy is kept.
- **Conflict copies:** when two computers push before the sync tool passed the change on, the tool keeps both and names one by its own pattern, e.g. `acme-PC-NAME.txt` (OneDrive), `acme (conflicted copy 2026-11-07).txt` (Dropbox), `acme.sync-conflict-20261107-101500-ABCD.txt` (Syncthing). The store finds these next to the shared copy, the engine merges each into the shared copy like one more side, and the store deletes it once merged. A merge that conflicts leaves the copy in place until the conflict is settled.
- Safe-write leftovers (`.tmp`, `.bak`) in the shared folder are ignored.
- **The shared copy disappears or is renamed:** the local file is never deleted. The file's status shows that the shared copy is missing, with **Share again** (push the local file to the old name) and **Stop sharing**.
- **Files On-Demand:** a cloud-only shared copy is downloaded when read; that download is not a change.

## 19\.7 Notes

- A note in the local file's notes folder (`todo-notes/`) is synced with the note of the same name in the shared copy's notes folder (`acme-notes/` next to `acme.txt`), using the same three-way merge on Markdown lines and the same conflict dialog.
- A new note on either side is copied to the other. A note is removed on the other side only when no todo links to it any more on either side.
- Notes linked by an absolute path are not synced.

## 19\.8 Stored data

| File | Holds |
| --- | --- |
| `~/.local/state/floetask/sync.toml` | One entry per linked file: local path, store kind and location (e.g. the folder), name of the shared copy, last version seen, time of the last sync, open conflicts |
| `~/.local/state/floetask/sync/` | Base copies of linked files and their notes, one per link |
| `config.toml` | `[sync]` settings: `interval` (seconds). Store credentials (later stores) go to the OS keychain, never to a file. |

Losing the base copies is safe: the next sync treats every differing line as a conflict instead of guessing.

## 19\.9 Implementation

Dependencies keep pointing inward (see `AGENTS.md`).

**New:**

- **Domain:** `merge3(base, local, shared) -> Merge`, the merged text plus a list of conflicts (line, local version, shared version), and applying the choices from the conflict dialog. Unit and property tests: a merge never changes untouched lines, merging a side with itself changes nothing, and a change on one side only always comes through.
- **Application:**
  - `RemoteStore` port: `list`, `get(name) -> (content, Version)`, `put(name, content, expected: Option<Version>) -> Result<Version>` failing with a version conflict, `remove(name)` (conflict copies), plus `conflict_copies(name)`.
  - `SyncLinkStore` port: loads and saves the links and base copies (§19.8).
  - `SyncService`: `share`, `join`, `sync(file)`, `resolve(file, choices)`, `unlink`. It reads local and shared, merges, writes the local file, then pushes with the version read, and on a version conflict reads, merges and retries a few times.
  - `SyncSettings` in `Settings` (`interval`), with `#[serde(default)]`.
- **Infrastructure:** `FolderRemoteStore` (§19.6), a TOML `SyncLinkStore`, the conflict-copy patterns of OneDrive, Dropbox and Syncthing.
- **GUI:** a `sync/` feature folder: the share and join dialogs, the status icon in the file drawer, the conflict dialog, Settings → Sync, and its messages. Every text goes through `i18n` in all locales.
- **Composition root:** builds the stores and `SyncService` and passes them in `Services`.

**Changed:**

- **`FileSystem` port and `LocalFileSystem`** (`fs.rs`): a conditional write, `write_if_unchanged(path, content, expected)`, which writes only if the file still holds the content it was read with. It reads the file just before the final rename of `safe_write` and fails with a "changed" error otherwise. `safe_write` itself stays as it is.
- **`TodoFileService::modify`** (`todo_files.rs`) and **`NoteService`**: write with `write_if_unchanged`, passing the content they loaded; on "changed", load again, apply the edit again and retry once. Without this, an edit that read the file before a sync wrote its merge would overwrite the merge. Their public functions stay the same; edits already find their todo by line and raw text, so they work on a freshly merged file.
- **`FileWatcher`** (`watcher.rs`): watches folders as well as single files, so conflict copies next to a shared copy are seen, and reports changes of linked files to the sync engine as well as to the view.
- **GUI file flow** (`files/`): starts a sync after a linked file was saved; the existing tick subscription drives the interval. A sync that changes the local file reloads it through the existing watcher path.

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
