# 4. Dates, deferral, recurrence (P0 / P1)

## 4.1 Completion

Marking complete:

1. Prepend `x ` and today's completion date.
2. If no creation date exists, set creation date to today (so the line becomes `x 2023-09-26 2023-09-26 …`).
3. Move priority into `pri:X`.
4. If `rec:` exists, create the next occurrence ([§4.3](#43-recurrence)).

Un-completing reverses 1 and 3 (restores priority from `pri:`).

## 4.2 Creation date

Setting `appendCreationDate`: new todos get today's date as creation date.

## 4.3 Recurrence

- Units: `d` daily, `b` business days (Mon–Fri), `w` weekly, `m` monthly, `y` yearly; optional count `N ≥ 1` (`rec:3m`).
- **Non-strict** (default): next due = completion date + interval.
- **Strict** (`rec:+…`): next due = previous due + interval; if there was no due date, falls back to completion date + interval.
- A todo with `rec:` but no due date still gets a due date on the new copy.
- With both `due:` and `t:`:
  - strict: add the interval to both;
  - non-strict: add the interval to `due` from completion date, then set `t` so the original `due − t` gap is preserved.
- The new copy gets today's creation date and keeps all other attributes.

Worked examples like this one are unit tests, e.g. `Water plants … due:2021-07-19 t:2021-07-09 rec:14d` completed 2021-07-13 → `due:2021-07-27 t:2021-07-17`.

## 4.4 Threshold (deferred) todos

`t:` in the future hides the todo unless the drawer toggle "show todos with future threshold date" is on. A matching toggle exists for future due dates.

## 4.5 Natural-language dates (P1)

When `convertRelativeToAbsoluteDates` is on, `due:tomorrow`, `t:in one week`, `due:next tuesday`, `due:end of february` are rewritten to ISO dates on save. Week start (Monday / Saturday / Sunday) is a setting.

**Rust notes:** candidates are `chrono-english`, `two_timer`, or a small own grammar covering the common phrases. The spec target is the common English phrases; a broader grammar is a stretch goal.

## 4.6 Human-friendly date display (P1)

Optional setting that displays dates as `overdue` / `elapsed`, `last week`, `today`, `tomorrow`, `next week`, `this month`, `next month`, else ISO. In the drawer, a date contributes to all buckets it falls in (e.g. due today shows under `today`, `this week`, `this month`).

---

[Back to the overview](../FEATURES.md)
