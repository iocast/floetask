# 14. Platform and distribution (P2)

- Linux, Windows, macOS builds (`cargo-dist` or `cargo-packager`).
- Packaging targets: AppImage, Flatpak, Snap, AUR, Homebrew cask, Windows/Mac stores. Sandboxed builds need portal-based file access (Flatpak document portal, macOS security-scoped bookmarks); `rfd` with the `xdg-portal` backend covers the Flatpak picker.
- Single-instance behaviour and window state restore.

---

[Back to the overview](../FEATURES.md)
