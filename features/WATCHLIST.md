# Watch list

Workarounds and limits in floetask that we can remove or improve once something outside our code changes. Check this list when a dependency is upgraded.

Each entry has:

- **Topic**: the area of the app it affects.
- **Limit**: what we can't do today, and why.
- **Workaround**: what the code does instead, and where.
- **Watch for**: the change that would let us remove the workaround.
- **Then**: what to change once that happens.

## Waiting on a library upgrade

### iced (now 0.14, with iced_winit 0.14.1 and winit 0.30.13)

#### Custom mouse cursor while dragging a todo

- **Topic**: drag and drop (list, board, calendar).
- **Limit**:
  - The operating system draws the mouse pointer straight to the screen.
  - The app's frames go through rendering and the compositor, so they appear one to three frames later.
  - A card drawn at the cursor position therefore always trails the system pointer.
  - The fix would be to make the card part of the system cursor, but iced doesn't allow that:
    - `iced::mouse::Interaction` is a fixed list of standard cursors. It has no variant for a custom image.
    - The winit window is private to iced, so the app can't set the cursor itself.
    - winit 0.30 does support custom cursors (`CustomCursor::from_rgba`); iced just doesn't expose them.
- **Workaround**:
  - While a todo is dragged, the system cursor is hidden (`Interaction::Hidden`).
  - The app draws its own arrow and the card in the same frame, so they move together.
  - The whole drag can still sit a frame or so behind the real mouse.
  - Code: `crates/floetask-gui/src/view/drag_layer.rs`.
- **Watch for**: an iced release that adds custom cursor images, for example an `Interaction` variant that takes an image, or a cursor API on `iced::window`.
- **Then**:
  - Render the dragged card into an image once, when the drag starts.
  - Set that image as the cursor, with the hotspot at the pointer's tip.
  - Drop the drawn pointer and card from `drag_layer.rs`, and drop the redraw on every mouse move.
  - Check how Windows, macOS and Linux limit cursor size, since a card is larger than a normal cursor.

### cargo-packager (now 0.11.8, built in CI with `cargo install cargo-packager --locked`)

#### Desktop shortcut question in the MSI

- **Topic**: Windows installers.
- **Limit**:
  - The built-in WiX template always creates a desktop shortcut, with no setting to ask or skip it.
  - The NSIS setup `.exe` already asks, with a checkbox on its finish page.
- **Workaround**:
  - `crates/floetask-app/wix/main.wxs` is a copy of the 0.11.8 template (set as `wix.template` in `crates/floetask-app/Cargo.toml`).
  - It adds a "Create a desktop shortcut" dialog after the folder choice and a condition on the shortcut component (the changes are listed in the comment at its top).
  - Because the template is a copy, upstream changes to it don't reach floetask by themselves.
- **Watch for**: a WiX setting in cargo-packager to make the desktop shortcut optional, or changes to the built-in template in a new release.
- **Then**: drop the copy and use the setting; until then, after upgrading cargo-packager, diff the new built-in template against the copy and carry its changes over.

## By topic

Workarounds that don't wait on a library. None yet.
