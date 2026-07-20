# cosmic-ext-applet-cheatsheet

A COSMIC panel applet + Super-key overlay that shows your **actual keyboard
shortcuts** — read live from COSMIC's own config — as a searchable, learnable
cheat sheet.

Unlike a static list, it reads `com.system76.CosmicSettings.Shortcuts` (system
defaults + your custom bindings), so it always reflects *your* real keybindings,
per machine. Your app-launch (`Spawn`) bindings stay clickable, and you can add
your own extra shortcuts and notes on top.

## Features

- **Reads your real COSMIC shortcuts** — defaults + custom, merged, grouped
  (Applications, System, Windows, Focus & move, Workspaces, Monitors, Zoom).
- **Top-anchored overlay** that drops from the top; opens from the panel icon or
  a keybind (e.g. `Super + C`). Esc or a click outside closes it; a second press
  toggles it shut.
- **Search** with autofocus, **arrow-key navigation** + Enter to launch the
  selected app binding.
- **Learning mode** — tick shortcuts you've memorised to hide them, so the list
  shrinks to what you're still learning (persisted).
- **Your own shortcuts & notes** — add/edit/delete from the ⚙ editor, stored in
  `~/.config/cosmic-ext-cheatsheet/custom.toml`.
- **Remembers** your last search + scroll across opens (toggle in settings).

## Build & install

Requires the Rust toolchain and a COSMIC session.

```sh
just build      # cargo build --release
just install    # install the binary + .desktop to ~/.local
```

Then add the applet to your panel (COSMIC Settings → Desktop → Panel → Configure
applets), and optionally bind a key (COSMIC Settings → Keyboard → Shortcuts) to:

```
cosmic-ext-applet-cheatsheet --window
```

## Config files

- `~/.config/cosmic-ext-cheatsheet/custom.toml` — your extra shortcuts/notes
  (see `custom.toml.example`).
- `~/.config/cosmic-ext-cheatsheet/settings.toml` — remember / learning toggles.
- `~/.config/cosmic-ext-cheatsheet/learned.toml` — shortcuts marked as learned.

## Status & notes

Early but functional. Known rough edges before it's fully upstream-ready:

- The shortcut labels are English (the search UI has i18n scaffolding; full
  localisation via fluent is a TODO).
- Corner-radius is disabled in code so it never sends a `cosmic_corner_radius_*`
  request, staying robust across compositor protocol versions; the panel content
  is rounded in-app instead.
- Tracks upstream `pop-os/libcosmic` (rev pinned in `Cargo.lock`). Builds and
  runs against current libcosmic master.

## License

GPL-3.0-only — see [`LICENSE`](LICENSE). The binary links COSMIC crates
(`cosmic-protocols`, `cosmic-client-toolkit`) that are GPL-3.0-only, so the
whole work is distributed under the GPL.
