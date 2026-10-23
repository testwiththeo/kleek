# KLEEK: Architecture & Engineering Specification

> **Project Name:** `kleek`  
> **Tagline:** *"Make your fancy mouse kleek on Linux. A blazingly-fast userspace driver & daemon written in Rust."*  
> **Target Platform:** Linux (Wayland / X11, systemd-based distributions like CachyOS/Arch, Fedora, Ubuntu)  
> **Implementation Language:** Rust (2024 Edition, Workspace Multi-Crate)

---

## 1. Problem Statement

Modern high-end gaming and productivity mice (such as Keychron, Pulsar, Darmoshark, VXE/ATK, and Apple peripherals) lack native Linux companion software. Proprietary configuration utilities are exclusively shipped for macOS and Windows. 

Furthermore, existing macOS open-source mouse utilities (like **LinearMouse** and **Mos**) implement critical UX features—such as per-device acceleration curves, smooth physics-based scrolling, and button remapping—using macOS-specific APIs (`CGEventTap`, `CVDisplayLink`, `IOHIDManager`). These cannot run on Linux, especially under Wayland compositors where global event tapping is restricted.

Linux users are currently forced to:
1. Boot Windows/macOS to alter hardware settings (DPI, polling rates, onboard profiles).
2. Suffer from rigid, discrete mouse-wheel step increments that feel jarring compared to trackpad inertia.
3. Deal with high CPU overhead or cursor jitter if attempting userspace pointer filtering at high polling rates (1000Hz–8000Hz).

---

## 2. The Solution: `kleek`

`kleek` is an open-source, dual-layer Linux mouse companion:
1. **Hardware Control Layer:** Communicates directly with mouse controllers via raw USB/Bluetooth HID reports (`/dev/hidraw*`) to query battery levels and configure onboard memory (DPI stages, polling rate, debounce, and lighting).
2. **Software Enhancement Layer:** Intercepts scroll and button events via Linux `evdev` and emits smooth, interpolated trackpad-like scrolling and remapped keystrokes via `/dev/uinput`, completely bypassing pointer motion events (`REL_X`/`REL_Y`) to maintain zero latency and 0% CPU consumption at 8000Hz polling rates.

---

## 3. High-Level System Architecture

```text
┌────────────────────────────────────────────────────────┐
│             User Interfaces (CLI / GUI)                │
│   • kleekctl (CLI via clap)                            │
│   • Desktop GUI (Tauri / D-Bus clients)                │
└───────────────────────────▲────────────────────────────┘
                            │ IPC: D-Bus (zbus)
                            │ org.freedesktop.Kleek1
┌───────────────────────────▼────────────────────────────┐
│                    kleek-daemon                        │
│  • Session State & Configuration Manager (TOML)        │
│  • Smooth Scroll Physics Loop (Inertia & Damping)      │
│  • Virtual Input Sink (/dev/uinput)                    │
└───────┬────────────────────────────────────────┬───────┘
        │ Internal API                           │ Internal API
┌───────▼────────────────────────┐ ┌─────────────▼───────┐
│          kleek-driver          │ │     kleek-proto     │
│  • udev Hotplug Watcher        │ │  • Zero-I/O Codecs  │
│  • hidapi Transport Manager    │ │  • Packet Builders  │
│  • evdev Grabber (Scroll/Keys) │ │  • CRC / Checksums  │
└───────┬────────────────────────┘ └─────────────────────┘
        │
┌───────▼────────────────────────┐
│     Linux Kernel Subsystem     │
│  /dev/hidraw* & /dev/input/*   │
└────────────────────────────────┘
```

---

## 4. Cargo Workspace Decomposition

To support parallel, agentic multi-agent development without file collisions, the project is strictly partitioned into four isolated crates:

### 4.1 `crates/kleek-proto` (Pure Protocol & Packet Codec)
* **Scope:** Zero I/O, pure data transformation.
* **Responsibilities:**
  * Defines domain enums: `DpiStage`, `PollingRate` (125Hz–8000Hz), `RgbMode`, `ButtonMapping`.
  * Serializes commands into exact 64-byte HID feature report buffers (`[u8; 64]`).
  * Deserializes hardware responses (e.g., parsing battery percentage and active profile).
  * Implements device-specific checksums (16-bit word sum, CRC-8, XOR).
* **Dependencies:** `zerocopy`, `thiserror`, `bitflags`.
* **Agent Suitability:** Small/fast models (Qwen-Coder, Flash) can develop and unit-test this crate using synthetic byte vectors extracted from Wireshark.

### 4.2 `crates/kleek-driver` (Hardware Transport & Event Hooking)
* **Scope:** System I/O and hardware interaction.
* **Responsibilities:**
  * Enumerates USB/Bluetooth devices via `hidapi` filtering on `vendor_id` and `product_id`.
  * Monitors device connection/disconnection via `udev`.
  * Grabs specific hardware event nodes via `evdev` (strictly grabbing scroll and button axes, while ungrabbing motion axes).
* **Dependencies:** `hidapi`, `udev`, `evdev`, `nix`.
* **Agent Suitability:** Systems-programming reasoning models.

### 4.3 `crates/kleek-daemon` (Orchestration & Background Service)
* **Scope:** Stateful daemon process.
* **Responsibilities:**
  * Exposes the D-Bus interface `org.freedesktop.Kleek1` via `zbus`.
  * Manages active configuration (`~/.config/kleek/config.toml`).
  * Physics engine for smooth scrolling: takes raw scroll clicks, applies quadratic/exponential ease-out damping, and emits high-resolution sub-pixel scroll events to a virtual `/dev/uinput` mouse.
* **Dependencies:** `zbus`, `tokio`, `serde`, `toml`, `tracing`.
* **Agent Suitability:** High-reasoning models for concurrency, async state machines, and timer loops.

### 4.4 `crates/kleek-cli` (User Control Interface `kleekctl`)
* **Scope:** Command-line executable.
* **Responsibilities:**
  * Formulates commands: `kleekctl dpi set <value>`, `kleekctl battery`, `kleekctl profile load <name>`.
  * Connects to the daemon over D-Bus as an unprivileged client (no `sudo` required).
  * Formats human-readable and JSON output.
* **Dependencies:** `clap` (derive), `zbus`, `colored`.

---

## 5. Architectural Invariants (Lessons from LinearMouse & Mos)

1. **Zero Pointer Motion Tapping:**  
   `kleek` MUST NEVER intercept or filter `REL_X` and `REL_Y` (cursor movement) in userspace. High-polling-rate mice (1000Hz–8000Hz) will suffer micro-stutters and high CPU utilization if passed through an event loop. Pointer speed and acceleration must be delegated natively to `libinput` / compositor configs.
2. **Deterministic Display/Tick Synchronization:**  
   Smooth scroll interpolation must use a fixed high-resolution timer (matching display refresh rate: 120Hz/144Hz/240Hz, typically 8ms or 4ms frames) to prevent scroll jitter.
3. **Privilege Separation:**  
   Hardware access permissions are managed entirely via udev rules (`/etc/udev/rules.d/99-kleek.rules`), placing devices in the `input` or `uaccess` group so `kleekd` runs safely under the standard user session (`systemd --user`).

---

## 6. Testing & Verification Seams

* **Seam 1 (Protocol Packet Testing):** `kleek-proto` must have 100% test coverage against recorded USB PCAP payloads (e.g., asserting that `SetDpi(1600)` produces the exact 64-byte payload observed in macOS captures).
* **Seam 2 (Virtual Device Verification):** `kleek-daemon` virtual device output is verified using `evemu-record` / `libinput debug-events` without requiring physical mouse hardware.
* **Seam 3 (IPC Interface Contract):** `kleekctl` tests run against a mock D-Bus service generated by `zbus`.
