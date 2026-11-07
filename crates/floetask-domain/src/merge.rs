//! Merging two versions of a text file line by line, for keeping a local
//! file in step with a shared copy (see `features/todo-notes/19-sync.md`).
//!
//! A todo is one line (a multi-line todo keeps its breaks inside the line),
//! so merging lines merges todos. The same merge serves Markdown notes.
//!
//! [`merge3`] compares both versions with the content they had at the last
//! sync (the base): whatever changed on one side only comes through, the
//! same change on both sides comes through once, and lines changed
//! differently on both sides are a [`Conflict`]. Lines neither side touched
//! stay byte for byte as they are.

use std::iter;
use std::ops::Range;

/// The result of a merge: clean text with the conflicts in between.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Merge {
    chunks: Vec<Chunk>,
    line_ending: &'static str,
    trailing_newline: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Chunk {
    Clean(Vec<String>),
    Conflict(Conflict),
}

/// Lines both sides changed, each in its own way. Either side may be empty:
/// a todo removed on one side and edited on the other.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Conflict {
    pub local: Vec<String>,
    pub shared: Vec<String>,
}

/// How a conflict is settled.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Choice {
    /// The local version.
    #[default]
    Mine,
    /// The shared copy's version.
    Theirs,
    /// Both, the local version first.
    Both,
}

impl Merge {
    /// The conflicts, in file order.
    pub fn conflicts(&self) -> impl Iterator<Item = &Conflict> {
        self.chunks.iter().filter_map(|chunk| match chunk {
            Chunk::Conflict(conflict) => Some(conflict),
            Chunk::Clean(_) => None,
        })
    }

    pub fn has_conflicts(&self) -> bool {
        self.conflicts().next().is_some()
    }

    /// The merged text, keeping the local version of every conflict.
    pub fn text(&self) -> String {
        self.resolve(&[])
    }

    /// The merged text with each conflict settled by the choice at its
    /// position. Conflicts without a choice keep the local version.
    pub fn resolve(&self, choices: &[Choice]) -> String {
        let mut conflict = 0;
        let mut lines: Vec<&str> = Vec::new();
        for chunk in &self.chunks {
            match chunk {
                Chunk::Clean(clean) => lines.extend(clean.iter().map(String::as_str)),
                Chunk::Conflict(Conflict { local, shared }) => {
                    let choice = choices.get(conflict).copied().unwrap_or_default();
                    conflict += 1;
                    let (mine, theirs) = match choice {
                        Choice::Mine => (true, false),
                        Choice::Theirs => (false, true),
                        Choice::Both => (true, true),
                    };
                    if mine {
                        lines.extend(local.iter().map(String::as_str));
                    }
                    if theirs {
                        lines.extend(shared.iter().map(String::as_str));
                    }
                }
            }
        }
        join(&lines, self.line_ending, self.trailing_newline)
    }
}

/// Merges `local` and `shared`, both edited from `base`. See the module
/// documentation for the rules.
pub fn merge3(base: &str, local: &str, shared: &str) -> Merge {
    let (base, local, shared) = (Text::split(base), Text::split(local), Text::split(shared));
    let mut changes: Vec<Change> = hunks(&base.lines, &local.lines)
        .into_iter()
        .map(|(base, lines)| Change {
            side: Side::Local,
            base,
            lines,
        })
        .chain(
            hunks(&base.lines, &shared.lines)
                .into_iter()
                .map(|(base, lines)| Change {
                    side: Side::Shared,
                    base,
                    lines,
                }),
        )
        .collect();
    // Insertions before changes at the same place, local before shared.
    changes.sort_by_key(|change| (change.base.start, change.base.end, change.side));

    let mut merged = Builder::default();
    let mut position = 0;
    let mut changes = changes.into_iter().peekable();
    while let Some(first) = changes.next() {
        let (start, mut end) = (first.base.start, first.base.end);
        let mut group = vec![first];
        while let Some(next) = changes.peek() {
            let overlaps = next.base.start < end || (start == end && next.base == (start..start));
            if !overlaps {
                break;
            }
            end = end.max(next.base.end);
            group.extend(changes.next());
        }
        merged.clean(&base.lines[position..start]);
        let mine = version(&group, Side::Local, start..end, &base.lines, &local.lines);
        let theirs = version(&group, Side::Shared, start..end, &base.lines, &shared.lines);
        let touched = |side: Side| group.iter().any(|change| change.side == side);
        match (touched(Side::Local), touched(Side::Shared)) {
            (true, false) => merged.clean(&mine),
            (false, true) => merged.clean(&theirs),
            _ if mine == theirs => merged.clean(&mine),
            // Both sides added lines at the same place: keep all of them.
            _ if start == end => merged.clean(&union(&mine, &theirs)),
            // Edited in place on both sides: only lines both changed conflict.
            _ if mine.len() == end - start && theirs.len() == end - start => {
                for ((original, mine), theirs) in base.lines[start..end].iter().zip(&mine).zip(&theirs) {
                    match (mine == original, theirs == original) {
                        (true, _) => merged.clean(&[theirs]),
                        (false, true) => merged.clean(&[mine]),
                        _ if mine == theirs => merged.clean(&[mine]),
                        _ => merged.conflict(&[mine], &[theirs]),
                    }
                }
            }
            _ => merged.conflict(&mine, &theirs),
        }
        position = end;
    }
    merged.clean(&base.lines[position..]);

    Merge {
        chunks: merged.chunks,
        line_ending: pick(base.line_ending(), local.line_ending(), shared.line_ending()),
        trailing_newline: pick(base.trailing_newline, local.trailing_newline, shared.trailing_newline),
    }
}

/// Merges two versions without a base, such as a local file joining a
/// shared copy: lines both have stay, every difference is a conflict.
pub fn merge2(local: &str, shared: &str) -> Merge {
    let (local, shared) = (Text::split(local), Text::split(shared));
    let mut merged = Builder::default();
    let mut position = 0;
    for (mine, theirs) in hunks(&local.lines, &shared.lines) {
        merged.clean(&local.lines[position..mine.start]);
        merged.conflict(&local.lines[mine.clone()], &shared.lines[theirs]);
        position = mine.end;
    }
    merged.clean(&local.lines[position..]);
    let first = if local.lines.is_empty() { &shared } else { &local };
    Merge {
        chunks: merged.chunks,
        line_ending: first.line_ending(),
        trailing_newline: first.trailing_newline,
    }
}

/// The words of `to`, each marked `true` when it is not in `from`: what the
/// conflict dialog highlights. Spaces stay with the word before them.
pub fn changed_words<'a>(from: &str, to: &'a str) -> Vec<(&'a str, bool)> {
    let before: Vec<&str> = from.split_inclusive(' ').map(str::trim_end).collect();
    let after: Vec<&'a str> = to.split_inclusive(' ').collect();
    let trimmed: Vec<&str> = after.iter().map(|word| word.trim_end()).collect();
    let mut kept = vec![false; after.len()];
    for (_, index) in matches(&before, &trimmed) {
        kept[index] = true;
    }
    after.into_iter().zip(kept).map(|(word, kept)| (word, !kept)).collect()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Side {
    Local,
    Shared,
}

/// Base lines `base` replaced by the side's lines `lines`.
struct Change {
    side: Side,
    base: Range<usize>,
    lines: Range<usize>,
}

/// One side's text over the base lines `range`, with its own changes in it.
fn version<'a>(group: &[Change], side: Side, range: Range<usize>, base: &[&'a str], lines: &[&'a str]) -> Vec<&'a str> {
    let mut out = Vec::new();
    let mut cursor = range.start;
    for change in group.iter().filter(|change| change.side == side) {
        out.extend_from_slice(&base[cursor..change.base.start]);
        out.extend_from_slice(&lines[change.lines.clone()]);
        cursor = change.base.end;
    }
    out.extend_from_slice(&base[cursor..range.end]);
    out
}

/// Both versions with the lines they share once, local lines first where
/// they differ.
fn union<'a>(local: &[&'a str], shared: &[&'a str]) -> Vec<&'a str> {
    let mut out = Vec::new();
    let mut position = 0;
    for (mine, theirs) in hunks(local, shared) {
        out.extend_from_slice(&local[position..mine.start]);
        out.extend_from_slice(&local[mine.clone()]);
        out.extend_from_slice(&shared[theirs]);
        position = mine.end;
    }
    out.extend_from_slice(&local[position..]);
    out
}

/// Line ending, final newline: whichever side changed it.
fn pick<T: PartialEq>(base: T, local: T, shared: T) -> T {
    if local == base { shared } else { local }
}

#[derive(Default)]
struct Builder {
    chunks: Vec<Chunk>,
}

impl Builder {
    fn clean(&mut self, lines: &[&str]) {
        if lines.is_empty() {
            return;
        }
        let lines = lines.iter().map(|line| (*line).to_owned());
        match self.chunks.last_mut() {
            Some(Chunk::Clean(clean)) => clean.extend(lines),
            _ => self.chunks.push(Chunk::Clean(lines.collect())),
        }
    }

    /// A conflict; lines both versions start or end with are clean.
    fn conflict(&mut self, local: &[&str], shared: &[&str]) {
        let prefix = local.iter().zip(shared).take_while(|(a, b)| a == b).count();
        let (local_rest, shared_rest) = (&local[prefix..], &shared[prefix..]);
        let suffix = local_rest
            .iter()
            .rev()
            .zip(shared_rest.iter().rev())
            .take_while(|(a, b)| a == b)
            .count();
        self.clean(&local[..prefix]);
        let owned = |lines: &[&str]| lines.iter().map(|line| (*line).to_owned()).collect::<Vec<_>>();
        let (local, shared) = (
            owned(&local_rest[..local_rest.len() - suffix]),
            owned(&shared_rest[..shared_rest.len() - suffix]),
        );
        // Conflicting lines next to each other are one conflict.
        match self.chunks.last_mut() {
            Some(Chunk::Conflict(previous)) => {
                previous.local.extend(local);
                previous.shared.extend(shared);
            }
            _ => self.chunks.push(Chunk::Conflict(Conflict { local, shared })),
        }
        self.clean(&local_rest[local_rest.len() - suffix..]);
    }
}

/// A text split into lines, with how it ends them.
struct Text<'a> {
    lines: Vec<&'a str>,
    crlf: bool,
    trailing_newline: bool,
}

impl<'a> Text<'a> {
    fn split(text: &'a str) -> Self {
        let trailing_newline = text.ends_with('\n');
        let body = text.strip_suffix('\n').unwrap_or(text);
        let body = body.strip_suffix('\r').unwrap_or(body);
        let lines = if text.is_empty() {
            Vec::new()
        } else {
            body.split('\n')
                .map(|line| line.strip_suffix('\r').unwrap_or(line))
                .collect()
        };
        Self {
            lines,
            crlf: text.contains("\r\n"),
            trailing_newline,
        }
    }

    fn line_ending(&self) -> &'static str {
        if self.crlf { "\r\n" } else { "\n" }
    }
}

fn join(lines: &[&str], line_ending: &str, trailing_newline: bool) -> String {
    let mut text = lines.join(line_ending);
    if trailing_newline && !lines.is_empty() {
        text.push_str(line_ending);
    }
    text
}

/// Where `a` and `b` differ: ranges of `a` replaced by ranges of `b`.
fn hunks(a: &[&str], b: &[&str]) -> Vec<(Range<usize>, Range<usize>)> {
    let mut out = Vec::new();
    let (mut i, mut j) = (0, 0);
    for (x, y) in matches(a, b).into_iter().chain(iter::once((a.len(), b.len()))) {
        if x > i || y > j {
            out.push((i..x, j..y));
        }
        i = x + 1;
        j = y + 1;
    }
    out
}

/// The pairs of equal lines of a longest common subsequence of `a` and `b`.
/// Lines both start or end with are matched first, since a sync usually
/// changes only a few lines in between.
fn matches<T: PartialEq>(a: &[T], b: &[T]) -> Vec<(usize, usize)> {
    let prefix = a.iter().zip(b).take_while(|(x, y)| x == y).count();
    let suffix = a[prefix..]
        .iter()
        .rev()
        .zip(b[prefix..].iter().rev())
        .take_while(|(x, y)| x == y)
        .count();
    let (a_end, b_end) = (a.len() - suffix, b.len() - suffix);
    (0..prefix)
        .map(|i| (i, i))
        .chain(
            myers(&a[prefix..a_end], &b[prefix..b_end])
                .into_iter()
                .map(|(x, y)| (x + prefix, y + prefix)),
        )
        .chain((0..suffix).map(|i| (a_end + i, b_end + i)))
        .collect()
}

/// Myers' O(ND) difference algorithm: the matched pairs of a shortest
/// edit script. Keeps one row of furthest points per edit, so memory grows
/// with the number of differences, not with the file size.
fn myers<T: PartialEq>(a: &[T], b: &[T]) -> Vec<(usize, usize)> {
    let (n, m) = (a.len() as isize, b.len() as isize);
    if n == 0 || m == 0 {
        return Vec::new();
    }
    let max = n + m;
    let offset = max + 1;
    let at = |k: isize| (k + offset) as usize;
    let mut furthest = vec![0isize; (2 * max + 3) as usize];
    let mut trace: Vec<Vec<isize>> = Vec::new();
    for d in 0..=max {
        for k in (-d..=d).step_by(2) {
            let mut x = if k == -d || (k != d && furthest[at(k - 1)] < furthest[at(k + 1)]) {
                furthest[at(k + 1)]
            } else {
                furthest[at(k - 1)] + 1
            };
            let mut y = x - k;
            while x < n && y < m && a[x as usize] == b[y as usize] {
                x += 1;
                y += 1;
            }
            furthest[at(k)] = x;
            if x >= n && y >= m {
                return backtrack(&trace, n, m, d);
            }
        }
        trace.push(furthest[at(-d)..=at(d)].to_vec());
    }
    Vec::new()
}

/// Walks the furthest points of each edit back from the end, collecting
/// the diagonal steps (matched lines). `trace[d]` holds the points after
/// `d` edits, for diagonals `-d..=d`.
fn backtrack(trace: &[Vec<isize>], n: isize, m: isize, edits: isize) -> Vec<(usize, usize)> {
    let (mut x, mut y) = (n, m);
    let mut pairs = Vec::new();
    for d in (1..=edits).rev() {
        let previous = &trace[(d - 1) as usize];
        let at = |k: isize| previous[(k + d - 1) as usize];
        let k = x - y;
        let previous_k = if k == -d || (k != d && at(k - 1) < at(k + 1)) {
            k + 1
        } else {
            k - 1
        };
        let previous_x = at(previous_k);
        let previous_y = previous_x - previous_k;
        while x > previous_x && y > previous_y {
            x -= 1;
            y -= 1;
            pairs.push((x as usize, y as usize));
        }
        x = previous_x;
        y = previous_y;
    }
    while x > 0 && y > 0 {
        x -= 1;
        y -= 1;
        pairs.push((x as usize, y as usize));
    }
    pairs.reverse();
    pairs
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    const BASE: &str = "(A) Call dentist\nBuy milk\n";

    #[test]
    fn spec_example_without_conflict() {
        let laptop = "(A) Call dentist\nx 2026-11-07 Buy milk\n";
        let phone = "(A) Call dentist\nBuy milk\nPay rent due:2026-11-10\n";
        let expected = "(A) Call dentist\nx 2026-11-07 Buy milk\nPay rent due:2026-11-10\n";
        assert_eq!(merge3(BASE, laptop, phone).text(), expected);
        assert_eq!(merge3(BASE, phone, laptop).text(), expected);
    }

    #[test]
    fn spec_example_with_conflict() {
        let mine = "(B) Call dentist\nBuy milk\nPay rent\n";
        let theirs = "(A) Call dentist +health\nx Buy milk\n";
        let merge = merge3(BASE, mine, theirs);
        let conflicts: Vec<_> = merge.conflicts().collect();
        assert_eq!(
            conflicts,
            [&Conflict {
                local: vec!["(B) Call dentist".to_owned()],
                shared: vec!["(A) Call dentist +health".to_owned()],
            }]
        );
        assert_eq!(merge.text(), "(B) Call dentist\nx Buy milk\nPay rent\n");
        assert_eq!(
            merge.resolve(&[Choice::Theirs]),
            "(A) Call dentist +health\nx Buy milk\nPay rent\n"
        );
        assert_eq!(
            merge.resolve(&[Choice::Both]),
            "(B) Call dentist\n(A) Call dentist +health\nx Buy milk\nPay rent\n"
        );
    }

    #[test]
    fn removed_on_one_side_edited_on_the_other_is_a_conflict() {
        let merge = merge3(BASE, "(A) Call dentist\n", "(A) Call dentist\nBuy oat milk\n");
        let conflict = merge.conflicts().next().unwrap();
        assert!(conflict.local.is_empty());
        assert_eq!(conflict.shared, ["Buy oat milk"]);
    }

    #[test]
    fn removed_on_one_side_unchanged_on_the_other_is_removed() {
        assert_eq!(merge3(BASE, "(A) Call dentist\n", BASE).text(), "(A) Call dentist\n");
        assert_eq!(merge3(BASE, BASE, "Buy milk\n").text(), "Buy milk\n");
    }

    #[test]
    fn lines_added_at_the_same_place_are_kept_local_first() {
        let merge = merge3(
            BASE,
            "one\n(A) Call dentist\nBuy milk\n",
            "two\n(A) Call dentist\nBuy milk\n",
        );
        assert!(!merge.has_conflicts());
        assert_eq!(merge.text(), "one\ntwo\n(A) Call dentist\nBuy milk\n");
        let same = merge3("", "a\nb\n", "a\nc\n").text();
        assert_eq!(same, "a\nb\nc\n");
    }

    #[test]
    fn changes_next_to_each_other_do_not_conflict() {
        let base = "a\nb\nc\n";
        assert_eq!(merge3(base, "A\nb\nc\n", "a\nB\nc\n").text(), "A\nB\nc\n");
    }

    #[test]
    fn keeps_line_endings_and_the_missing_final_newline() {
        let base = "a\r\nb";
        assert_eq!(merge3(base, "a\r\nb", "a\r\nb\r\nc").text(), "a\r\nb\r\nc");
        assert_eq!(merge3("a\n", "a\nb", "a\n").text(), "a\nb");
    }

    #[test]
    fn multi_line_todos_merge_as_one_line() {
        let base = "List\u{10}- milk\nOther\n";
        let mine = "List\u{10}- milk\u{10}- bread\nOther\n";
        let theirs = "List\u{10}- milk\nOther +home\n";
        assert_eq!(
            merge3(base, mine, theirs).text(),
            "List\u{10}- milk\u{10}- bread\nOther +home\n"
        );
    }

    #[test]
    fn without_a_base_every_difference_is_a_conflict() {
        let merge = merge2("a\nb\nc\n", "a\nB\nc\nd\n");
        let conflicts: Vec<_> = merge.conflicts().cloned().collect();
        assert_eq!(conflicts.len(), 2);
        assert_eq!(conflicts[0].local, ["b"]);
        assert_eq!(conflicts[0].shared, ["B"]);
        assert!(conflicts[1].local.is_empty());
        assert_eq!(merge.resolve(&[Choice::Theirs, Choice::Theirs]), "a\nB\nc\nd\n");
        assert_eq!(merge2("", "x\n").resolve(&[Choice::Theirs]), "x\n");
    }

    #[test]
    fn changed_words_marks_what_differs() {
        let words = changed_words("(A) Call dentist", "(A) Call dentist +health");
        assert_eq!(
            words,
            [
                ("(A) ", false),
                ("Call ", false),
                ("dentist ", false),
                ("+health", true)
            ]
        );
    }

    /// The textbook dynamic-programming LCS, to check Myers against.
    fn lcs_length(a: &[&str], b: &[&str]) -> usize {
        let mut table = vec![vec![0; b.len() + 1]; a.len() + 1];
        for i in (0..a.len()).rev() {
            for j in (0..b.len()).rev() {
                table[i][j] = if a[i] == b[j] {
                    table[i + 1][j + 1] + 1
                } else {
                    table[i + 1][j].max(table[i][j + 1])
                };
            }
        }
        table[0][0]
    }

    fn texts() -> impl Strategy<Value = String> {
        proptest::collection::vec("[abc]{0,2}", 0..8).prop_map(|lines| {
            let mut text = lines.join("\n");
            if !lines.is_empty() {
                text.push('\n');
            }
            text
        })
    }

    proptest! {
        #[test]
        fn a_change_on_one_side_always_comes_through(base in texts(), changed in texts()) {
            prop_assert_eq!(merge3(&base, &changed, &base).text(), changed.clone());
            prop_assert_eq!(merge3(&base, &base, &changed).text(), changed);
        }

        #[test]
        fn merging_a_side_with_itself_changes_nothing(base in texts(), both in texts()) {
            let merge = merge3(&base, &both, &both);
            prop_assert!(!merge.has_conflicts());
            prop_assert_eq!(merge.text(), both);
        }

        #[test]
        fn matches_are_a_common_subsequence(a in texts(), b in texts()) {
            let (a, b): (Vec<&str>, Vec<&str>) = (a.lines().collect(), b.lines().collect());
            let pairs = matches(&a, &b);
            for window in pairs.windows(2) {
                prop_assert!(window[0].0 < window[1].0 && window[0].1 < window[1].1);
            }
            for &(x, y) in &pairs {
                prop_assert_eq!(a[x], b[y]);
            }
            prop_assert_eq!(pairs.len(), lcs_length(&a, &b));
        }
    }
}
