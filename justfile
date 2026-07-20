name := "cosmic-ext-applet-cheatsheet"
appid := "io.github.tomashaa.CosmicExtCheatsheet"
bindir := env_var('HOME') / ".local/bin"
appsdir := env_var('HOME') / ".local/share/applications"

# Build the release binary.
build:
    cargo build --release

# Install the binary + .desktop applet entry into ~/.local.
install: build
    install -Dm755 target/release/{{name}} {{bindir}}/{{name}}
    install -Dm644 data/{{appid}}.desktop {{appsdir}}/{{appid}}.desktop
    @echo "Installed. Add the applet to your panel in COSMIC Settings, and bind"
    @echo "'{{name}} --window' to a key (e.g. Super+C)."

# Remove the installed files.
uninstall:
    rm -f {{bindir}}/{{name}} {{appsdir}}/{{appid}}.desktop

# Run the standalone overlay for testing.
run:
    cargo run -- --window
