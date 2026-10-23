# KLEEK: Agentic Engineering Ticket Breakdown

This ticket registry adheres to Agentic Multi-Agent Engineering standards:
* **Directory Partitioning:** Every ticket is strictly constrained to its designated crate directory.
* **Model Capability Matching:** Tasks are labeled with the minimum recommended model class.
* **Deterministic Verification:** Every ticket defines an exact command to verify completion.

---

## Ticket Overview

| Ticket ID | Target Crate | Title | Recommended Model | Status |
| :--- | :--- | :--- | :--- | :--- |
| **KLK-001** | Root / Workspace | Cargo Workspace Initialization & Udev Rules | Fast / Small Model | Completed ✓ |
| **KLK-002** | `crates/kleek-proto` | Packet Codec & Checksum Engine | Fast / Coding Model | Completed ✓ |
| **KLK-003** | `crates/kleek-driver` | HID Transport & Hotplug Detection | Reasoning Model | Ready |
| **KLK-004** | `crates/kleek-daemon` | Virtual Uinput & Smooth Scroll Physics Engine | Frontier Reasoning Model | Ready |
| **KLK-005** | `crates/kleek-daemon` | D-Bus Service Interface (`zbus`) | Reasoning Model | Ready |
| **KLK-006** | `crates/kleek-cli` | `kleekctl` CLI Implementation | Fast / Small Model | Ready |

---

### Ticket KLK-001: Cargo Workspace Initialization & Udev Rules
* **Target Path:** `/home/theo/kleek/Cargo.toml`, `/home/theo/kleek/udev/99-kleek.rules`
* **Model Class:** Fast / Small Model (e.g. Qwen-Flash / GLM-Flash)
* **Objective:** Set up the root `Cargo.toml` workspace, define member crates, configure shared workspace dependencies, and create standard Linux udev rules for non-root `/dev/hidraw*` access.
* **Acceptance Criteria:**
  1. `cargo check --workspace` passes without errors.
  2. `udev/99-kleek.rules` contains `TAG+="uaccess"` and `MODE="0660"` rules for the target device VID/PID.

---

### Ticket KLK-002: Packet Codec & Checksum Engine
* **Target Path:** `crates/kleek-proto/`
* **Model Class:** Fast / Coding Model (e.g. Qwen-2.5-Coder)
* **Objective:** Implement pure data structures and byte serialization/deserialization for mouse HID feature reports.
* **Contract:**
  ```rust
  pub enum MouseCommand {
      SetDpi(u16),
      SetPollingRate(u16),
      SetRgb { r: u8, g: u8, b: u8, mode: u8 },
      QueryBattery,
  }

  impl MouseCommand {
      pub fn encode(&self) -> [u8; 64];
      pub fn decode_response(buf: &[u8; 64]) -> Result<MouseResponse, ProtoError>;
  }
  ```
* **Acceptance Criteria:**
  1. No I/O or system call dependencies allowed.
  2. 100% test coverage asserting byte encoding matches golden hex test vectors.
  3. `cargo test -p kleek-proto` passes.

---

### Ticket KLK-003: HID Transport & Hotplug Detection
* **Target Path:** `crates/kleek-driver/`
* **Model Class:** Reasoning Model (e.g. Claude 3.7 Sonnet / DeepSeek Reasoner)
* **Objective:** Connect to `/dev/hidraw*` using `hidapi`, expose asynchronous read/write streams, and detect plug/unplug events via `udev` monitor.
* **Acceptance Criteria:**
  1. Safely handles sudden device disconnects without panicking.
  2. Emits strongly typed events: `DeviceConnected(DeviceInfo)` and `DeviceDisconnected(String)`.
  3. `cargo clippy -p kleek-driver` produces 0 warnings.

---

### Ticket KLK-004: Virtual Uinput & Smooth Scroll Physics Engine
* **Target Path:** `crates/kleek-daemon/src/physics.rs`, `crates/kleek-daemon/src/uinput.rs`
* **Model Class:** Frontier Reasoning Model
* **Objective:** Build the physics-based scrolling interpolation inspired by Mos:
  * Grabs discrete scroll events from physical mouse via `evdev`.
  * Computes an ease-out inertia trajectory (decay curve).
  * Emits high-resolution scroll deltas via `/dev/uinput` virtual device synchronized to display refresh intervals (e.g. 8ms / 120Hz).
* **Constraints:**
  * Must never touch or grab `REL_X` / `REL_Y` axes.
* **Acceptance Criteria:**
  1. Mathematical simulation unit tests confirming decay smoothly reaches zero within target duration.
  2. Virtual mouse successfully creates `/dev/uinput` node with scroll capabilities.

---

### Ticket KLK-005: D-Bus Service Interface (`zbus`)
* **Target Path:** `crates/kleek-daemon/src/dbus.rs`
* **Model Class:** Reasoning Model
* **Objective:** Expose the `org.freedesktop.Kleek1` D-Bus interface so external unprivileged processes can query mouse status and alter parameters.
* **Methods:**
  * `SetDpi(dpi: u16) -> Result<(), Error>`
  * `GetBattery() -> Result<u8, Error>`
  * `SetSmoothScroll(enabled: bool) -> Result<(), Error>`
* **Acceptance Criteria:**
  1. Daemon binds to the D-Bus user session bus.
  2. Interface introspectable via standard `busctl --user introspect`.

---

### Ticket KLK-006: `kleekctl` CLI Implementation
* **Target Path:** `crates/kleek-cli/`
* **Model Class:** Fast / Small Model
* **Objective:** Implement the user-facing CLI using `clap` derive, calling the D-Bus methods from KLK-005.
* **Commands:**
  * `kleekctl dpi <value>`
  * `kleekctl battery`
  * `kleekctl scroll --smooth on|off`
* **Acceptance Criteria:**
  1. Clean help text and `--json` output flag.
  2. Returns exit code 0 on success, non-zero with formatted error message if daemon is unreachable.
