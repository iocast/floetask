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
cargo packager --release -p floetask-app                  # every format of the current OS
cargo packager --release -p floetask-app --formats nsis   # just the setup .exe
```

The GitHub Actions release workflow (`.github/workflows/release.yml`) runs the same command on Windows, macOS and Linux runners when a version tag is pushed and attaches every installer to the GitHub release.

## 14.2 About page and updates (implemented)

Settings → About shows the logo, name, version and a one-line description, and the update check:

- **Check for updates on start** (setting, default on): a release build asks for a newer release when it starts and shows a toast if there is one. Failures stay silent.
- **Check now**: the same check on request, with the result on the page (up to date, new version with its release notes, or the error).
- **Install and restart**: downloads the newest installer, verifies its signature and runs it. Windows uses the setup `.exe` (progress bar only, the app restarts), macOS replaces the `.app`, Linux replaces the AppImage. A `.deb` or `.msi` install updates through its package manager instead.

How it works: [cargo-packager-updater](https://docs.rs/cargo-packager-updater) behind the `Updater` port (`floetask-infrastructure/src/updater.rs`). The release workflow signs every installer and publishes `latest.json` (written by `scripts/update-manifest.py`) next to them; the app reads `https://github.com/<owner>/<repo>/releases/latest/download/latest.json`. Which manifest to read and which public key to trust are compiled in from `FLOETASK_UPDATE_ENDPOINT` and `FLOETASK_UPDATE_PUBKEY`, so local builds say "This build cannot update itself" and never call out.

One-time setup before the first release that should update:

1. Generate the update signing key pair: `cargo packager signer generate` (choose a password). Keep the private key safe: losing it means existing installs can no longer verify updates.
2. In the GitHub repository settings, add the secrets `FLOETASK_UPDATE_PRIVATE_KEY` (the private key) and `FLOETASK_UPDATE_KEY_PASSWORD` (its password), and the variable `FLOETASK_UPDATE_PUBKEY` (the public key).
3. Push a version tag. Releases built before this setup cannot update themselves; the first configured release has to be installed by hand once.

This signature only proves an update comes from this project. It is separate from the OS code signing below.

Not done yet: code signing. Unsigned Windows installers trigger SmartScreen ("unknown publisher") and unsigned macOS apps are blocked by Gatekeeper until the user allows them in System Settings. Signing needs a Windows code-signing certificate and an Apple Developer ID with notarization; cargo-packager supports both through its `windows.certificate-thumbprint` and `macos.signing-identity` settings plus CI secrets.

---

[Back to the overview](../FEATURES.md)
