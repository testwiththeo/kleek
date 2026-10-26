# kleek

[![Build Status](https://img.shields.io/badge/build-passing-brightgreen?style=flat-square)](https://github.com/testwiththeo/kleek/actions)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue?style=flat-square)](LICENSE-MIT)
[![Rust Edition](https://img.shields.io/badge/rust-2021%20edition-orange?style=flat-square)](https://www.rust-lang.org/)
[![Platform](https://img.shields.io/badge/platform-Linux-informational?style=flat-square)](#platform-support)

The mouse companion that gets out of your way.

`kleek` is a lightweight, open-source mouse companion designed to give you precise control over your pointer, scroll wheel, and device settings without heavy background suites.

## Features

* **Natural Scrolling**: Smooths out discrete wheel clicks with fluid, momentum-based scrolling.
* **Independent Directions**: Keep natural scrolling on your trackpad while using traditional scrolling on your mouse.
* **Hardware Control**: Configure DPI, polling rates, and check battery levels directly.
* **Per-App Profiles**: Apply different scroll speeds and button behaviors automatically depending on the active app.
* **Button Mappings**: Bind mouse buttons to keyboard shortcuts, navigation, or custom actions.
* **Minimal Footprint**: Built with Rust for immediate response times and near-zero memory overhead.

## Quick Start

### Debian / Ubuntu (.deb)

Download the latest `.deb` package from [Releases](https://github.com/testwiththeo/kleek/releases) and install it:

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

# Toggle smooth scrolling
kleekctl scroll --smooth on
```

## Platform Support

* **Linux**: Native userspace daemon via `evdev`, `/dev/uinput`, and `hidapi` (Wayland & X11).

## Documentation

* [Architecture & Engineering Specification](SPEC.md)
* [D-Bus IPC API Specification](docs/DBUS_API.md)
* [Smooth Scroll Physics Model](docs/SMOOTH_SCROLL_PHYSICS.md)

## License

MIT OR Apache-2.0
