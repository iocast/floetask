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

## By topic

Workarounds that don't wait on a library. None yet.
