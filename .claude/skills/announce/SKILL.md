---
name: announce
description: Write social media announcements for a floetask release from its CHANGELOG.md section, in three versions - short (X), normal (Mastodon) and a Ko-fi post. Use when the user wants to announce, post about, promote or share a release, asks for a release post, tweet, toot or Ko-fi update, or invokes /announce, optionally with a version such as 0.4.0.
allowed-tools: Read, Grep, Bash(git describe:*), Bash(git tag:*), Bash(git show:*), Bash(python:*)
---

# Release announcements

Writes three ready-to-paste posts for one floetask release. Nothing is posted: Ko-fi has no
API for posts, and publishing is the user's job.

## 1. Pick the release

- Use the version the user names (`0.4.0` or `v0.4.0`).
- Otherwise use the latest tag: `git describe --tags --abbrev=0 --match "v*"`.
- Read its section `## [X.Y.Z] - <date>` in `CHANGELOG.md`. If there is none (for example the
  changes are still under `[Unreleased]`), say so and stop; do not invent changes.
- A final release (`X.Y.Z`) collects the changes of its release candidates; if its own section
  is short, read the `X.Y.Z-rc.N` sections too and announce what is new since the last final
  release.
- For a release candidate (`X.Y.Z-rc.N`) call it a release candidate (a preview) and say how to
  try it: download it from the release page, or in an installed floetask switch Settings →
  About → Release channel to *Preview*.

## 2. Choose what to say

Changelog entries are detailed and too long for social media. Pick the one or two changes
users care about most (usually from Added, or a notable fix) and say them in plain words, as
what you can now do with your todo.txt. Keyboard shortcuts, config keys and todo.txt tokens
(`due:`, `note:`) only where they help, e.g. in the Ko-fi post, not in the X post. Never claim
anything the section does not say.

## 3. Write the three versions

Facts to use:

- The project: floetask, a fast, keyboard-friendly todo.txt manager for Windows, macOS and
  Linux, written in Rust with the iced GUI library. Your todos stay in a plain todo.txt file;
  floetask adds a list, a status board, a calendar, saved searches, Markdown notes and
  reminders on top. Free and open source (MIT / Apache 2.0).
- Website: https://iocast.github.io/floetask
- Download (installers for Windows, macOS and Linux): https://github.com/iocast/floetask/releases
- Release page: https://github.com/iocast/floetask/releases/tag/vX.Y.Z (use the real tag)
- Repository: https://github.com/iocast/floetask
- Ko-fi: https://ko-fi.com/iocast
- Existing users get the update in the app: Settings → About → Install and restart.

**Short (X)** - at most 280 characters as X counts them (every link counts as 23, an emoji
as 2). Version, one sentence with the highlight, the release page link,
`#todotxt #rustlang`.

**Normal (Mastodon)** - at most 500 characters (every link counts as 23). One line on what
floetask is, the notable changes as `•` bullets, website and release page links, hashtags
`#todotxt #Productivity #Rust #RustLang #OpenSource`. Mention that it also fits Bluesky only if
it is at most 300 characters; otherwise say the bullets have to go for Bluesky.

**Ko-fi** - at most 800 characters, links counted in full. A friendly post of a few short
paragraphs in the first person, with no title (Ko-fi posts have none), so the first sentence
names the version: what floetask is, what is new and why it matters, how to get it (download
link, or the in-app update for existing users), the website link and a short closing thank-you
to supporters. Smaller changes and the repository link only if they fit. No hashtags, no
Markdown (Ko-fi shows it as plain text).

Tone: plain and friendly, no hype words ("revolutionary", "game-changing"). At most one or two
emoji per post.

## 4. Check the length

Count the short and normal versions before showing them, e.g.:

```bash
python -c "import re,sys; t=sys.stdin.read().rstrip(); print(len(re.sub(r'https?://\S+','x'*23,t)))" <<'EOF'
...post...
EOF
```

Emoji count as 2 on X; add that by hand. Count the Ko-fi post with plain `len(t)` (links in
full). Shorten until they fit.

## 5. Output

Show each version in its own `text` code block, headed with the platform and its count (e.g.
"X - 198 / 280", "Ko-fi - 774 / 800"). Do not write files, commit, or post anything.
