# kleek

[![Build Status](https://img.shields.io/badge/build-passing-brightgreen?style=flat-square)](https://github.com/testwiththeo/kleek/actions)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue?style=flat-square)](LICENSE-MIT)
[![Rust Edition](https://img.shields.io/badge/rust-2021%20edition-orange?style=flat-square)](https://www.rust-lang.org/)
[![Platform](https://img.shields.io/badge/platform-Linux-informational?style=flat-square)](#platform-support)

The mouse companion that gets out of your way.

`kleek` is a lightweight, open-source mouse companion designed to give you precise control over your pointer, scroll wheel, and device settings without heavy background suites.

## Features

* **Natural Scrolling**: Smooths out discrete wheel clicks with fluid, momentum-based scrolling using a cubic-bezier ease-out curve that lands with zero lingering slip.
* **Modifier Key Actions**:
  - `Shift` + Scroll: Automatically converts vertical wheel rolls into smooth horizontal scrolling (`REL_HWHEEL_HI_RES`).
  - `Alt` + Scroll: Precision mode (0.3× micro speed) for fine navigation and technical reading.
  - `Ctrl` + Scroll: Controlled zoom increments without erratic runaway momentum.
* **Per-Device Independence**: Automatically detects and isolates laptop touchpads, keeping native trackpad gestures intact while applying smooth gliding exclusively to external mice.
* **App-Specific Exclusions**: Automatically bypasses inertia for games, 3D software (Blender, FreeCAD), and terminals so wheel clicks stay immediate and discrete.
* **Independent Directions**: Keep natural scrolling on your trackpad while using traditional scrolling on your mouse.
* **Hardware Control**: Configure DPI, polling rates, and check battery levels directly.
* **Per-App Profiles**: Apply different scroll speeds and button behaviors automatically depending on the active app.
* **Button Mappings**: Bind mouse buttons to keyboard shortcuts, navigation, or custom actions.
* **Minimal Footprint**: Built with Rust for immediate response times and near-zero memory overhead.

## Quick Start

### One-Line Install (Debian / Ubuntu / Mint / Pop!_OS)

Install and activate `kleek` in one command:

```bash
curl -fsSL https://raw.githubusercontent.com/testwiththeo/kleek/main/install.sh | bash
```

### Manual (.deb)

Download the `.deb` package directly from [Releases](https://github.com/testwiththeo/kleek/releases) and install it:

```bash
sudo dpkg -i kleek_*.deb
systemctl --user start kleek
```

### Build from Source

Run the automated installer script:

```bash
git clone https://github.com/testwiththeo/kleek.git
cd kleek
./install.sh
```

### Basic Usage

Use `kleekctl` in your terminal or bind commands to system shortcuts:

```bash
# Check device status and battery
kleekctl status
kleekctl battery

# Change sensor resolution
kleekctl dpi 1600

# Set polling rate
kleekctl rate 1000

# Toggle smooth scrolling and direction
kleekctl scroll --smooth on
kleekctl scroll --natural on       # macOS natural direction (or off for traditional)

# Manage app exclusions (bypass smooth scroll in games & CAD)
kleekctl apps list
kleekctl apps add blender
kleekctl apps remove steam

# Configure Touchpad independently (Mac-like trackpad experience)
kleekctl touchpad                 # View mouse vs touchpad independence status
kleekctl touchpad --mac-preset    # Enable natural scrolling & tap-to-click
kleekctl touchpad --natural on    # Independent natural scroll toggle

# Manage mouse profiles and templates
kleekctl config list
kleekctl config init gaming       # Generates ~/.config/kleek/config.toml
kleekctl config apply productivity

# Check and update to latest release
kleekctl update
```

## Platform Support

* **Linux**: Native userspace daemon via `evdev`, `/dev/uinput`, and `hidapi` (Wayland & X11).

## Documentation

* [Architecture & Engineering Specification](SPEC.md)
* [D-Bus IPC API Specification](docs/DBUS_API.md)
* [Smooth Scroll Physics Model](docs/SMOOTH_SCROLL_PHYSICS.md)

## License

MIT OR Apache-2.0
