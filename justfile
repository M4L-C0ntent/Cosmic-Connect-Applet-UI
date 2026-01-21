# justfile

# Variables
prefix := "/usr/local"
flatpak_prefix := "/app"
appid := "io.github.M4LC0ntent.Cosmic-Connect-UI"

# Default recipe to display available commands
default:
    @just --list

# ================================
# Build Commands
# ================================

# Build all binaries in debug mode
build:
    cargo build

# Build all binaries in release mode
build-release:
    cargo build --release

# Build a specific binary in release mode
build-bin BIN:
    cargo build --release --bin {{BIN}}

# Clean build artifacts
clean:
    cargo clean

# ================================
# Run Commands
# ================================

# Run the applet in debug mode
run-applet:
    cargo run --bin cosmic-connect-applet

# Run the settings app in debug mode
run-settings:
    cargo run --bin cosmic-connect-settings

# Run the SMS app in debug mode
run-sms:
    cargo run --bin cosmic-connect-sms

# Run in release mode
run-release:
    cargo run --release --bin cosmic-connect-applet

# ================================
# Check Commands
# ================================

# Check code without building
check:
    cargo check

# Format code using rustfmt
fmt:
    cargo fmt

# Check code formatting
fmt-check:
    cargo fmt -- --check

# Run clippy lints
clippy:
    cargo clippy -- -D warnings

# Run all checks (fmt, clippy, build)
test-all: fmt-check clippy build
    @echo "✓ All checks passed!"

# ================================
# Install Commands
# ================================

# Install to system (requires root/sudo)
install:
    @echo "Installing cosmic-connect to {{prefix}}..."
    install -Dm0755 ./target/release/cosmic-connect-applet {{prefix}}/bin/cosmic-connect-applet
    install -Dm0755 ./target/release/cosmic-connect-settings {{prefix}}/bin/cosmic-connect-settings
    install -Dm0755 ./target/release/cosmic-connect-sms {{prefix}}/bin/cosmic-connect-sms
    install -Dm0644 ./resources/{{appid}}.desktop {{prefix}}/share/applications/{{appid}}.desktop
    install -Dm0644 ./resources/{{appid}}.metainfo.xml {{prefix}}/share/metainfo/{{appid}}.metainfo.xml
    @echo "✓ Installation complete!"

# Uninstall from system (requires root/sudo)
uninstall:
    @echo "Uninstalling cosmic-connect from {{prefix}}..."
    rm -f {{prefix}}/bin/cosmic-connect-applet
    rm -f {{prefix}}/bin/cosmic-connect-settings
    rm -f {{prefix}}/bin/cosmic-connect-sms
    rm -f {{prefix}}/share/applications/{{appid}}.desktop
    rm -f {{prefix}}/share/metainfo/{{appid}}.metainfo.xml
    @echo "✓ Uninstall complete!"

# ================================
# Flatpak Commands
# ================================

# Install to flatpak prefix
install-flatpak: build-release
    @echo "Installing cosmic-connect to flatpak {{flatpak_prefix}}..."
    install -Dm0755 ./target/release/cosmic-connect-applet {{flatpak_prefix}}/bin/cosmic-connect-applet
    install -Dm0755 ./target/release/cosmic-connect-settings {{flatpak_prefix}}/bin/cosmic-connect-settings
    install -Dm0755 ./target/release/cosmic-connect-sms {{flatpak_prefix}}/bin/cosmic-connect-sms
    install -Dm0644 ./resources/{{appid}}.desktop {{flatpak_prefix}}/share/applications/{{appid}}.desktop
    install -Dm0644 ./resources/{{appid}}.metainfo.xml {{flatpak_prefix}}/share/metainfo/{{appid}}.metainfo.xml
    @echo "✓ Flatpak installation complete!"

# Build flatpak (requires flatpak-builder)
build-flatpak:
    @echo "Building flatpak..."
    flatpak-builder --user --install --force-clean build-dir {{appid}}.json
    @echo "✓ Flatpak build complete!"

# Clean flatpak build artifacts
clean-flatpak:
    rm -rf build-dir .flatpak-builder

# Run flatpak
run-flatpak:
    flatpak run {{appid}}