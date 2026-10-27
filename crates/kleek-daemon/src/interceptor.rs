use crate::modifier::ModifierTracker;
use anyhow::Result;
use evdev::{Device, RelativeAxisType};
use libc::{c_int, ioctl};
use std::os::unix::io::AsRawFd;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::mpsc;
use tracing::{debug, info, warn};

// EVIOCGRAB ioctl request code = _IO('E', 0x90) on Linux.
// Computed from standard Linux _IOC encoding: (dir<<30)|(size<<16)|(type<<8)|(nr<<0).
// For _IO (no data transfer): dir=0, size=0, type='E'(0x45), nr=0x90 → 0x4590.
const EVIOCGRAB: u64 = 0x4590;

// EVIOCGRAB ioctl: makes the caller the exclusive recipient of events.
// When grabbed (1), events are NOT forwarded to other clients — our virtual uinput
// device becomes the sole scroll source, replacing physical REL_WHEEL entirely.
// This is critical: without grabbing, the kernel also dispatches the raw physical
// REL_WHEEL events to libinput → double scrolling (physical + virtual).
fn grab_device(dev: &Device) -> std::io::Result<()> {
    let fd = dev.as_raw_fd();
    let arg: c_int = 1; // 1 = grab, 0 = ungrab
    let ret = unsafe { ioctl(fd, EVIOCGRAB, arg) };
    if ret < 0 {
        return Err(std::io::Error::last_os_error());
    }
    Ok(())
}

fn ungrab_device(dev: &Device) -> std::io::Result<()> {
    let fd = dev.as_raw_fd();
    let arg: c_int = 0;
    let ret = unsafe { ioctl(fd, EVIOCGRAB, arg) };
    if ret < 0 {
        return Err(std::io::Error::last_os_error());
    }
    Ok(())
}

pub fn is_touchpad_device(dev: &Device) -> bool {
    let name = dev.name().unwrap_or("").to_lowercase();
    if name.contains("touchpad")
        || name.contains("trackpad")
        || name.contains("glidepoint")
        || name.contains("synaptics")
        || name.contains("alps")
        || name.contains("elan")
    {
        return true;
    }

    let props = dev.properties();
    if props.contains(evdev::PropType::BUTTONPAD)
        || props.contains(evdev::PropType::SEMI_MT)
        || props.contains(evdev::PropType::TOPBUTTONPAD)
    {
        return true;
    }

    false
}

pub fn is_mouse_device(dev: &Device) -> bool {
    let name = dev.name().unwrap_or("");
    // Skip our own virtual mouse to prevent event loops!
    if name.contains("kleek Virtual") {
        return false;
    }

    // Ignore laptop touchpads so built-in gestures remain pure libinput
    if is_touchpad_device(dev) {
        return false;
    }

    // Must have RELATIVE events with REL_WHEEL
    if let Some(rel) = dev.supported_relative_axes() {
        return rel.contains(RelativeAxisType::REL_WHEEL);
    }

    false
}

pub fn scan_mouse_event_nodes() -> Vec<PathBuf> {
    let mut nodes = Vec::new();
    if let Ok(entries) = std::fs::read_dir("/dev/input") {
        for entry in entries.flatten() {
            let path = entry.path();
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if name.starts_with("event") {
                    if let Ok(dev) = Device::open(&path) {
                        if is_mouse_device(&dev) {
                            nodes.push(path);
                        }
                    }
                }
            }
        }
    }
    nodes
}

pub fn start_mouse_interceptor(
    path: PathBuf,
    tx: mpsc::Sender<crate::scroll_service::ScrollImpulse>,
    modifier_tracker: Arc<ModifierTracker>,
    side_action: crate::gestures::SideScrollAction,
) -> tokio::task::JoinHandle<Result<()>> {
    tokio::task::spawn_blocking(move || {
        let mut dev = match Device::open(&path) {
            Ok(d) => d,
            Err(e) => {
                warn!("Cannot open mouse device {:?}: {}", path, e);
                return Err(e.into());
            }
        };

        info!("Intercepting mouse scroll wheel events on {:?}", path);
        // Grab the device exclusively so physical REL_WHEEL events are NOT
        // forwarded to libinput — preventing double-scrolling. Our virtual
        // uinput device becomes the sole scroll source.
        if let Err(e) = grab_device(&dev) {
            warn!(
                "Failed to grab {}: {} — physical events may still pass through (double scroll risk)",
                path.display(),
                e
            );
        } else {
            info!(
                "Successfully grabbed {:?} for exclusive event handling",
                path
            );
        }
        let mut last_event_time = Instant::now();
        let mut side_button_pressed = false;
        let mut last_gesture_time = Instant::now();

        loop {
            let events_result = dev.fetch_events();
            match events_result {
                Ok(events) => {
                    for ev in events {
                        if ev.event_type() == evdev::EventType::KEY {
                            let code = ev.code();
                            // BTN_SIDE (0x116 / 278) or BTN_EXTRA (0x115 / 277)
                            if code == evdev::Key::BTN_SIDE.0 || code == evdev::Key::BTN_EXTRA.0 {
                                side_button_pressed = ev.value() == 1;
                                debug!(
                                    "Side button state changed: pressed={}",
                                    side_button_pressed
                                );
                            }
                        } else if ev.event_type() == evdev::EventType::RELATIVE
                            && ev.code() == RelativeAxisType::REL_WHEEL.0
                        {
                            let value = ev.value();
                            if value != 0 {
                                let now = Instant::now();
                                if side_button_pressed
                                    && side_action != crate::gestures::SideScrollAction::Disabled
                                {
                                    // Rate limit gesture triggers to 140ms for clean notch-to-action mapping
                                    if now.duration_since(last_gesture_time).as_millis() >= 140 {
                                        last_gesture_time = now;
                                        crate::gestures::handle_side_scroll(
                                            side_action,
                                            value as f64,
                                        );
                                    }
                                    continue;
                                }

                                let interval_ms =
                                    now.duration_since(last_event_time).as_millis() as u64;
                                last_event_time = now;

                                let modifiers = modifier_tracker.current_modifiers();
                                debug!(
                                    "Hardware wheel event: value={}, interval={}ms, modifiers={:?}",
                                    value, interval_ms, modifiers
                                );
                                let impulse = crate::scroll_service::ScrollImpulse {
                                    direction: value as f64,
                                    interval_ms: interval_ms.max(1),
                                    modifiers,
                                };
                                let _ = tx.blocking_send(impulse);
                            }
                        }
                    }
                }
                Err(e) => {
                    warn!(
                        "Mouse device disconnected or read error on {:?}: {}",
                        path, e
                    );
                    break;
                }
            }
        }
        // Release the exclusive grab so the device returns to normal
        // libinput control when we disconnect.
        let _ = ungrab_device(&dev);
        Ok(())
    })
}
