# 14. Platform and distribution (P2)

- Linux, Windows, macOS builds (`cargo-dist` or `cargo-packager`).
- Packaging targets: AppImage, Flatpak, Snap, AUR, Homebrew cask, Windows/Mac stores. Sandboxed builds need portal-based file access (Flatpak document portal, macOS security-scoped bookmarks); `rfd` with the `xdg-portal` backend covers the Flatpak picker.
- Single-instance behaviour and window state restore.

## 14.1 Installers (implemented)

Release builds are GUI apps on Windows (`windows_subsystem = "windows"`): a double click opens only the window, no console. Started from a terminal, `floetask --help` and `--paths` still print there, because the app attaches to the parent console.

Installers are built with [cargo-packager](https://github.com/crabnebula-dev/cargo-packager). Its configuration is `[package.metadata.packager]` in `crates/floetask-app/Cargo.toml` (name, identifier, publisher, icons from `assets/`). Each OS can only build its own formats:

| OS | Formats | Output |
|---|---|---|
| Windows | NSIS installer (`.exe`, per-user, no admin prompt), MSI (WiX) | `target/packages/*-setup.exe`, `*.msi` |
| macOS | `.app` bundle, `.dmg` disk image | `target/packages/*.app`, `*.dmg` |
| Linux | `.deb`, AppImage | `target/packages/*.deb`, `*.AppImage` |

Build locally:

```sh
cargo install cargo-packager --locked
cargo packager --release            # every format of the current OS
cargo packager --release --formats nsis
```

The GitHub Actions release workflow (`.github/workflows/release.yml`) runs the same command on Windows, macOS and Linux runners when a version tag is pushed and attaches every installer to the GitHub release.

Not done yet: code signing. Unsigned Windows installers trigger SmartScreen ("unknown publisher") and unsigned macOS apps are blocked by Gatekeeper until the user allows them in System Settings. Signing needs a Windows code-signing certificate and an Apple Developer ID with notarization; cargo-packager supports both through its `windows.certificate-thumbprint` and `macos.signing-identity` settings plus CI secrets.

---

[Back to the overview](../FEATURES.md)
