# floetask website

The product page for floetask, built with the [Zola](https://www.getzola.org) static site generator. It is a single landing page: features, screenshots and download links to the GitHub releases.

## Run locally

Install Zola once (`winget install getzola.zola`, `brew install zola`, or see the [installation guide](https://www.getzola.org/documentation/getting-started/installation/)), then from this folder:

```sh
zola serve    # http://127.0.0.1:1111, reloads on change
zola build    # writes the site to public/ (git-ignored)
```

The deploy workflow pins Zola 0.23.6. Templates use that version's syntax, so use the same version locally.

## Layout

| Path | What |
|---|---|
| `config.toml` | Site settings. `base_url` and `extra.repo` (the GitHub repository the download links point to) |
| `content/_index.md` | The page text: tagline, lead, feature cards, screenshots and download cards, all in the `[extra]` front matter |
| `templates/base.html` | Page frame: head, navigation, footer |
| `templates/index.html` | The landing page sections |
| `templates/icon.html` | The feature card icons, picked by the card's `icon` name |
| `sass/style.scss` | All styles, with light and dark colours following the system theme |
| `static/` | Logo, icon, `screenshots/` and `lightbox.js` (opens screenshots in a dialog with previous/next), copied to the site as is |

## Common edits

- **Text or features:** edit `content/_index.md`. A new feature card needs an `icon` that `templates/icon.html` knows.
- **Logo:** copy `../assets/logo.svg` and `../assets/icon-256.png` into `static/`.
- **Screenshots:** they come from the GUI snapshot tests. Run `FLOETASK_SNAPSHOTS=/some/dir cargo test -p floetask-gui` from the repository root and copy `list`, `board`, `editor` and `dark` from that folder into `static/screenshots/` (drop the `-wgpu` suffix).

## Deploy

`.github/workflows/website.yml` builds the site and publishes it to GitHub Pages on every push to `main` that changes `website/`, or when started by hand. It sets `base_url` from the Pages URL. Before the first run, set the repository's **Settings → Pages → Source** to **GitHub Actions**.
