use crate::modifier::ModifierTracker;
use anyhow::Result;
use evdev::{Device, RelativeAxisType};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::mpsc;
use tracing::{debug, info, warn};

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
        let mut last_event_time = Instant::now();

        loop {
            match dev.fetch_events() {
                Ok(events) => {
                    for ev in events {
                        if ev.event_type() == evdev::EventType::RELATIVE
                            && ev.code() == RelativeAxisType::REL_WHEEL.0
                        {
                            let value = ev.value();
                            if value != 0 {
                                let now = Instant::now();
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
        Ok(())
    })
}
