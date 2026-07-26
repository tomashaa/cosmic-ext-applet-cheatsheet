# cosmic-ext-applet-cheatsheet

A COSMIC panel applet + Super-key overlay that shows your **actual keyboard
shortcuts** — read live from COSMIC's own config — as a searchable, learnable
cheat sheet.

Unlike a static list, it reads `com.system76.CosmicSettings.Shortcuts` (system
defaults + your custom bindings), so it always reflects *your* real keybindings
per machine. App-launch (`Spawn`) bindings stay clickable, and you can add your
own extra shortcuts and notes on top.

## Features

- **Live COSMIC shortcuts** — defaults + custom, merged and grouped (Apps,
  System, Windows, Focus & move, Workspaces, Monitors, Zoom).
- **Non-modal right-edge panel** — slides in from the right; the rest of the
  desktop stays clickable and typable. Close with Esc or Super+C (toggle).
- **Panel applet + keybind** — panel icon opens the sheet **in-process**;
  `cosmic-ext-applet-cheatsheet --window` (e.g. Super+C) toggles the running
  applet when present, otherwise opens a short-lived standalone sheet.
- **Search** with autofocus, **arrow-key navigation** + Enter to launch.
- **Compact overview (default)** — merges arrow/workspace duplicates and hides
  media keys; toggle **Show all** for the full dump.
- **Cosmic symbolic icons** on rows for faster scanning.
- **Fluent i18n** (9 languages) with a language picker in ⚙; stored in
  `settings.toml` (falls back to system locale, then legacy
  `~/.config/cosmic-clip/ui-lang` if present).
- **Learning mode** — mark shortcuts as learned to hide them (persisted).
- **Custom shortcuts & notes** — add/edit/delete from the ⚙ editor
  (`custom.toml`).
- **Remembers** last search + scroll across opens (optional).

## Build & install

Requires the Rust toolchain and a COSMIC session. [`just`](https://github.com/casey/just) is optional but convenient.

```sh
just build      # cargo build --release
just install    # binary + .desktop + metainfo → ~/.local
just check      # clippy
```

Then add the applet to your panel (COSMIC Settings → Desktop → Panel → Configure
applets), and optionally bind a key to:

```
cosmic-ext-applet-cheatsheet --window
```

## Config files

| Path | Purpose |
|------|---------|
| `~/.config/cosmic-ext-cheatsheet/custom.toml` | Extra shortcuts / notes (`custom.toml.example`) |
| `~/.config/cosmic-ext-cheatsheet/settings.toml` | remember / learning / compact / **lang** |
| `~/.config/cosmic-ext-cheatsheet/learned.toml` | Shortcuts marked as learned |

## Status

Functional as a **community extension** (`cosmic-ext-*`). Not aimed at a PR into
`pop-os/cosmic-applets` (first-party applets); distribution path is this repo /
eventual Flatpak or COSMIC Store packaging.

Still rough for “upstream polish”:

- Shortcut config live-reloads via `cosmic-config` subscription.
- Corner-radius protocol is disabled; the panel is rounded in-app instead.
- Standalone `--window` without a running applet still uses a PID-file toggle.
- CI builds on GitHub Actions; vendor recipe available via `just vendor`.

## License

GPL-3.0-only — see [`LICENSE`](LICENSE). The binary links COSMIC crates
(`cosmic-protocols`, `cosmic-client-toolkit`) that are GPL-3.0-only, so the
whole work is distributed under the GPL.
