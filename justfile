name := "cosmic-ext-applet-cheatsheet"
appid := "io.github.tomashaa.CosmicExtCheatsheet"
bindir := env_var('HOME') / ".local/bin"
appsdir := env_var('HOME') / ".local/share/applications"
metainfodir := env_var('HOME') / ".local/share/metainfo"
iconsdir := env_var('HOME') / ".local/share/icons/hicolor/scalable/apps"
export CARGO_TARGET_DIR := env_var_or_default('CARGO_TARGET_DIR', justfile_directory() / "target")

# Default: release build.
build: build-release

build-release *args:
    cargo build --release {{args}}

build-debug *args:
    cargo build {{args}}

# Install the binary + desktop + icon + metainfo into ~/.local.
install: build-release
    install -Dm755 {{CARGO_TARGET_DIR}}/release/{{name}} {{bindir}}/{{name}}
    install -Dm644 data/{{appid}}.desktop {{appsdir}}/{{appid}}.desktop
    install -Dm644 data/{{appid}}.metainfo.xml {{metainfodir}}/{{appid}}.metainfo.xml
    install -Dm644 data/icons/hicolor/scalable/apps/{{appid}}-symbolic.svg {{iconsdir}}/{{appid}}-symbolic.svg
    -gtk-update-icon-cache -f {{env_var('HOME')}}/.local/share/icons/hicolor 2>/dev/null
    @echo "Installed. Add the applet in COSMIC Settings, and bind"
    @echo "'{{name}} --window' to a key (e.g. Super+C)."

uninstall:
    rm -f {{bindir}}/{{name}} {{appsdir}}/{{appid}}.desktop {{metainfodir}}/{{appid}}.metainfo.xml {{iconsdir}}/{{appid}}-symbolic.svg

# Run the standalone overlay for testing.
run:
    cargo run -- --window

# Clippy (same spirit as other COSMIC projects).
check *args:
    cargo clippy --all-features -- -W clippy::all {{args}}

check-json: (check "--message-format=json")

# Vendor dependencies for offline / distro builds.
vendor:
    #!/usr/bin/env bash
    set -euo pipefail
    cargo vendor --synced vendor
    echo -e '\n[source.crates-io]\nreplace-with = "vendored-sources"\n\n[source.vendored-sources]\ndirectory = "vendor"' >> .cargo/config.toml

clean:
    cargo clean
