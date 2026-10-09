# 12. Navigation, menus, keyboard (P0 / P1)

## 12.1 Navigation sidebar

Vertical icon bar: add todo, toggle drawer, archive (when a done file is set and completed todos exist), open file, settings. Toggle with `Ctrl+Alt+H`.

## 12.2 Application menu (P2)

An application menu (File, Edit, View, Todos, Window, Help). iced has no native menu bar; options are an in-app menu bar (`iced_aw` menu or custom) or `muda` for native menus. All menu actions must also be reachable via shortcuts or the UI so the menu stays optional.

## 12.3 Keyboard shortcuts

| Action | Shortcut (Cmd on macOS where applicable) |
|---|---|
| New todo | Ctrl+N |
| Find | Ctrl+F |
| Show saved search filters | Ctrl+Shift+F |
| Toggle completed | Ctrl+H |
| Reset search and filters | Ctrl+0 |
| Archive completed | Ctrl+Alt+A |
| Open file | Ctrl+O |
| Switch to file 1–9 | Ctrl+1…9 |
| Settings | Ctrl+, |
| Toggle drawer | Ctrl+B |
| Toggle navigation | Ctrl+Alt+H |
| Toggle theme | Ctrl+Alt+D |
| Close window / Quit | Ctrl+W / Ctrl+Q |
| Save in dialog / save search as todo | Ctrl+Enter |
| Close dialog / drawer / clear search | Escape |
| Open selected todo | Enter |
| Delete selected todo | Delete or Ctrl+Backspace |
| Move focus in list | Up / Down, Left / Right |
| Toggle complete | Space |

**iced notes:** global shortcuts via `keyboard::listen` / `keyboard::on_key_press` subscription mapping to messages; row focus and selection must be app state since iced's focus model only covers text inputs. Use `widget::operation` (focus, scroll_to) to keep the selected row visible.

## 12.4 Clipboard and undo

- Copy todo to clipboard (`iced::clipboard`).
- Undo/redo inside text fields. P2: app-level undo of the last file change.

---

[Back to the overview](../FEATURES.md)
