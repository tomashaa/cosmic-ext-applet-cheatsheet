# cosmic-ext-applet-cheatsheet

A COSMIC panel applet: a searchable **keyboard-shortcut cheat sheet** with
user-defined custom shortcuts. Native [libcosmic](https://github.com/pop-os/libcosmic)
port of the GTK `cosmic-cheatsheet` from [clip-suite](https://github.com/tomashaa/clip-suite).

Click the panel icon to open a searchable list of COSMIC shortcuts and quick
actions. Clickable actions (Terminal, Files, Screenshot …) launch on click;
the rest are a reference you can filter as you type.

> Status: early scaffold. The panel button, popup, search, built-in shortcut
> data and the custom-shortcut config loader are in place; polish and theming
> are in progress.

## Custom shortcuts

Add your own entries in `~/.config/cosmic-ext-cheatsheet/custom.toml`:

```toml
[[shortcut]]
label = "My screenshot tool"
keys = "Super + Shift + M"
command = "my-tool --flag"   # optional — if present, the row is clickable
section = "My tools"          # optional grouping (defaults to "Custom")

[[shortcut]]
label = "Note: build server"
keys = "10.0.0.5"             # informational only (no command)
```

They appear under their section in the cheat sheet and update on next open.

## Build & install

```sh
cargo build --release
# install the binary + a .desktop applet entry (see justfile, TODO)
```

## License

MIT for this crate's own code (see SPDX headers). **Pending:** the effective
license of the built binary depends on the COSMIC dependency graph
(`cosmic-panel-config` and friends) — to be confirmed before publishing.
