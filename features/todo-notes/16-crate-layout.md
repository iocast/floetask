# 16. Suggested crate layout (for later, not yet created)

```
floetask/
  Cargo.toml            # workspace, edition = "2024"
  crates/
    floetask-core/        # todo.txt parser, recurrence, dates, filter language, sort/group, archive, safe write
    floetask-app/         # iced application: state, messages, views, subscriptions
```

Keeping all todo logic in `floetask-core` with no iced dependency lets it be fully unit-tested, with worked examples as acceptance tests.

---

[Back to the overview](../FEATURES.md)
