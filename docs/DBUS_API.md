# KLEEK D-Bus IPC Specification

This document defines the frozen D-Bus IPC contract between `kleek-daemon` (the server) and user-facing clients such as `kleekctl` (the CLI) and desktop GUIs.

---

## 1. Connection Details

* **Bus Type:** Session Bus (`DBUS_BUS_SESSION`)
* **Well-Known Name:** `org.freedesktop.Kleek1`
* **Root Object Path:** `/org/freedesktop/Kleek1`
* **Device Object Path:** `/org/freedesktop/Kleek1/Devices/{id}` (e.g. `/org/freedesktop/Kleek1/Devices/dev_046d_c539`)

---

## 2. Manager Interface: `org.freedesktop.Kleek1.Manager`

**Object Path:** `/org/freedesktop/Kleek1`

### Methods

#### `ListDevices() -> (ao: device_paths)`
Returns an array of object paths for all currently connected and recognized devices.

#### `GetActiveDevice() -> (o: device_path)`
Returns the object path of the primary active mouse device.

#### `ReloadConfig() -> ()`
Reloads `~/.config/kleek/config.toml` from disk and applies changes across all devices.

### Signals

#### `DeviceAdded(o: device_path)`
Emitted when a supported mouse is connected (USB plugged or 2.4GHz dongle handshake established).

#### `DeviceRemoved(o: device_path)`
Emitted when a device is unplugged or disconnected.

---

## 3. Device Interface: `org.freedesktop.Kleek1.Device`

**Object Path:** `/org/freedesktop/Kleek1/Devices/{id}`

### Properties

| Property Name | Type | Access | Description |
| :--- | :--- | :--- | :--- |
| `Name` | `s` | Read | Human-readable device marketing name (e.g. "Keychron M6") |
| `VendorId` | `q` | Read | USB Vendor ID (u16 hex) |
| `ProductId` | `q` | Read | USB Product ID (u16 hex) |
| `BatteryPercent` | `y` | Read | Battery charge level (0–100, or 255 if unknown / wired) |
| `IsCharging` | `b` | Read | True if device is actively charging via USB cable |
| `CurrentDpi` | `q` | Read/Write | Active DPI value (e.g. 800, 1600, 3200) |
| `PollingRate` | `q` | Read/Write | Active polling rate in Hz (125, 250, 500, 1000, 2000, 4000, 8000) |
| `SmoothScrollEnabled` | `b` | Read/Write | True if Mos-style physics smooth scrolling is active |

### Methods

#### `SetDpi(q: dpi) -> ()`
Sets the active sensor resolution. Writes the corresponding HID feature report to the mouse hardware.
* **Errors:**
  * `org.freedesktop.Kleek1.Error.InvalidParameter`: DPI out of supported sensor range.
  * `org.freedesktop.Kleek1.Error.HardwareFailure`: Device failed to acknowledge the report.

#### `SetPollingRate(q: hz) -> ()`
Sets the USB report polling interval.
* **Allowed values:** `125`, `250`, `500`, `1000`, `2000`, `4000`, `8000`.

#### `SetRgb(y: mode, y: r, y: g, y: b) -> ()`
Sets the onboard RGB lighting mode and color.

#### `SetSmoothScroll(b: enabled) -> ()`
Enables or disables the software-level inertia scroll engine for this specific device.

### Signals

#### `BatteryLevelChanged(y: percent, b: is_charging)`
Emitted whenever the hardware reports an updated battery state.

#### `DpiChanged(q: new_dpi)`
Emitted when the DPI is changed either via D-Bus method or when the user presses a physical DPI button on the mouse.
