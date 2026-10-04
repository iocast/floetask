# floetask

A todo.txt manager written in Rust with an [iced](https://iced.rs) GUI.

Status: early scaffolding. See [FEATURES.md](FEATURES.md) for the full specification and milestones.

## Layout

- `crates/floetask-core`: todo.txt parser and task logic, no GUI dependency.
- `crates/floetask-app`: the iced desktop app (binary `floetask`).

## Build and run

```sh
cargo run -p floetask-app
cargo test --workspace
```
