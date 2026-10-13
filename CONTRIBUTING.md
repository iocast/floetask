# Contributing to floetask

Code layout, style and tests are described in [AGENTS.md](AGENTS.md); the product spec starts at [features/FEATURES.md](features/FEATURES.md).

## Translations

floetask's interface is translated with [Fluent](https://projectfluent.org). Each language has one file, `crates/floetask-gui/locales/<language>/floetask.ftl`, compiled into the binary. English (`locales/en/floetask.ftl`) is the source: new strings are added there first.

### Adding a language

1. Copy `locales/en/floetask.ftl` to `locales/<code>/floetask.ftl`, where `<code>` is the language subtag (`pt`, `nl`, `pl`).
2. Translate the values; never change the ids left of `=`.
3. Add the language to `Language` in `crates/floetask-gui/src/i18n/mod.rs`: a variant, its code, its name in itself ("Nederlands"), its file in `source`, and its place in `SHIPPED`.
4. Run `cargo test -p floetask-gui`. The tests fail when your file misses an id English has, has one English lacks, uses a placeholder English does not, or does not parse.

A language ships only when every string is translated ([§13.2](features/todo-notes/13-i18n.md)).

### Fluent in short

```ftl
# A plain string.
settings_title = Settings

# Placeholders are named, so you can move them: keep the name, change the order.
note_outside_folder = { $file } is not in { $folder }, where this list keeps its notes.

# Plurals pick a variant by the number. Use your language's plural categories
# (zero, one, two, few, many, other); `*` marks the default.
renamed =
    { $count ->
        [one] Updated 1 todo
       *[other] Updated { $count } todos
    }

# An exact number is also a variant.
notification_threshold =
    { $days ->
        [0] Notify on the due date
       *[other] Notify { $days } days ahead
    }
```

Long values may continue on indented lines. A literal `{` is written `{"{"}`.

### What stays as it is

- todo.txt syntax in examples: `due:`, `t:`, `rec:`, `status:`, `h:1`, ISO dates.
- Search keywords (`due`, `today`, `and`) and natural dates (`due:tomorrow`): they stay English in every language, so saved filters keep working.
- Shortcut keys, except the key names your keyboards print (`Strg` in German).
- The name floetask.

### Dates

Month and weekday names are ordinary strings (`month_10`, `weekday_tue`, `month_short_10`). The calendar titles `date_month_year`, `date_long` and the `date_range*` messages arrange them; write them in your language's usual order and punctuation ("Dienstag, 6. Oktober 2026"). Names may be lowercase where your language writes them so; titles get a capital first letter.

### Glossary

Keep these terms consistent within a language:

| English | Meaning |
|---|---|
| todo | one line of a todo.txt file |
| project | a `+project` tag |
| context | an `@context` tag |
| threshold | the `t:` date from which a todo shows |
| status | the `status:` workflow state (to do, doing, waiting, someday) |
| board | the status board, one column per status |
| done file | the file completed todos are archived to |

Each locale file starts with a comment listing its translations of these terms.

### Checking layouts

Debug builds offer a pseudo-language, `language = "en-XA"` in `config.toml` or *Pseudo* in Settings, which makes every string 40 % longer and wraps it in brackets. Text that is clipped, or shown without brackets because it bypasses translation, stands out.
