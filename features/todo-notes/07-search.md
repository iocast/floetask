# 7. Search (P1)

## 7.1 Search bar

- `Ctrl+F` opens/focuses. `Escape` clears, a second `Escape` hides.
- Plain text does a case-insensitive substring match on the raw line.

## 7.2 Advanced filter expressions

If the input parses as an expression it is evaluated; otherwise it falls back to literal match. While typing an invalid expression, keep showing results of the last valid one.

Grammar:

- Logical: `or`/`OR`/`||`, `and`/`AND`/`&&`, `not`/`NOT`/`!`, parentheses; precedence not > and > or.
- `+` any project; `+bi` project contains `bi`; `+"big"` exact project.
- `@` / `@ho` / `@"home"` likewise for contexts.
- `due:` has a due date; `due: <op> DATE`; `due:2021-06` prefix match. Same for `t:`.
- DATE = ISO date or `today` / `tomorrow` / `yesterday`, optionally `± N(d|b|w|m|y)`, or a natural-language phrase from [§4.5](04-dates.md#45-natural-language-dates-p1) such as `friday`, `next week`, `end of month` (resolved with the week-start setting).
- Without an operator, an `end of …` phrase is a deadline and means `<=`: `due: end of week` is everything due by the end of this week. Other dates mean `==`.
- `priority` / `pri` has priority; `pri <op> A`; `(B)` shorthand for `pri == B`.
- `complete` keyword.
- `"text"` or `'text'` literal; `/regex/` regex match.
- Operators: `==`, `=`, `!=`, `<`, `<=`, `>`, `>=`.

**Rust notes:** implement with `pest` or `chumsky` (or hand-written recursive descent); `regex` crate for `/…/` (syntax differs slightly from JavaScript regex; document the difference).

## 7.3 Saved search filters

- Save the current query as a named filter (`Ctrl+Shift+F` / arrow-down opens the list).
- Pick, delete (with confirmation) saved filters; `Up`/`Down` + `Enter` in the list.
- Per filter: **suppress notifications** for todos matching it (bell icon).
- Stored in `filters.toml`, watched for external changes.
- A new installation starts with one filter, **overdue** (`due: < today+1d`: due today or earlier). Saving or deleting a filter replaces the defaults with the user's own list.

---

[Back to the overview](../FEATURES.md)
