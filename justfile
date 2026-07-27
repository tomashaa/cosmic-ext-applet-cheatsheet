name := "cosmic-ext-applet-cheatsheet"
appid := "io.github.tomashaa.CosmicExtCheatsheet"

# Default local install prefix; Flatpak uses `just prefix=/app install`.
rootdir := ""
prefix := env_var_or_default("PREFIX", home / ".local")
base-dir := absolute_path(clean(rootdir / prefix))

bindir := base-dir / "bin"
appsdir := base-dir / "share" / "applications"
metainfodir := base-dir / "share" / "metainfo"
iconsdir := base-dir / "share" / "icons" / "hicolor" / "scalable" / "apps"

export CARGO_TARGET_DIR := env_var_or_default("CARGO_TARGET_DIR", justfile_directory() / "target")

build: build-release

build-release *args:
    cargo build --release {{args}}

build-debug *args:
    cargo build {{args}}

# Install binary + desktop + icon + metainfo (PREFIX=~/.local by default).
install: build-release
    install -Dm755 {{CARGO_TARGET_DIR}}/release/{{name}} {{bindir}}/{{name}}
    install -Dm644 data/{{appid}}.desktop {{appsdir}}/{{appid}}.desktop
    install -Dm644 data/{{appid}}.metainfo.xml {{metainfodir}}/{{appid}}.metainfo.xml
    install -Dm644 data/icons/hicolor/scalable/apps/{{appid}}-symbolic.svg {{iconsdir}}/{{appid}}-symbolic.svg
    -gtk-update-icon-cache -f {{base-dir}}/share/icons/hicolor 2>/dev/null
    @echo "Installed to {{base-dir}}. Add the applet in COSMIC Settings, and bind"
    @echo "'{{name}} --window' (or flatpak run {{appid}} --window) to a key."

uninstall:
    rm -f {{bindir}}/{{name}} {{appsdir}}/{{appid}}.desktop {{metainfodir}}/{{appid}}.metainfo.xml {{iconsdir}}/{{appid}}-symbolic.svg

run:
    cargo run -- --window

check *args:
    cargo clippy --all-features -- -W clippy::all {{args}}

check-json: (check "--message-format=json")

# Local Flatpak build + user install (needs flatpak-builder + Cosmic.BaseApp).
flatpak-install:
    #!/usr/bin/env bash
    set -euo pipefail
    command -v flatpak-builder >/dev/null || {
      echo "Install flatpak-builder first (e.g. sudo apt install flatpak-builder)" >&2
      exit 1
    }
    flatpak remote-add --user --if-not-exists flathub https://dl.flathub.org/repo/flathub.flatpakrepo || true
    flatpak remote-add --user --if-not-exists cosmic https://apt.pop-os.org/cosmic/cosmic.flatpakrepo || true
    flatpak install -y --user flathub org.freedesktop.Sdk//25.08 org.freedesktop.Platform//25.08 \
      org.freedesktop.Sdk.Extension.rust-stable//25.08 || true
    flatpak install -y --user cosmic com.system76.Cosmic.BaseApp//stable || true
    flatpak-builder --user --install --force-clean build-dir flatpak/{{appid}}.json

flatpak-uninstall:
    flatpak uninstall -y --user {{appid}} || true

vendor:
    #!/usr/bin/env bash
    set -euo pipefail
    cargo vendor --synced vendor
    echo -e '\n[source.crates-io]\nreplace-with = "vendored-sources"\n\n[source.vendored-sources]\ndirectory = "vendor"' >> .cargo/config.toml

clean:
    cargo clean
    rm -rf build-dir .flatpak-builder
