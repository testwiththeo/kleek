use evdev::{Device, Key};
use std::path::PathBuf;
use tracing::debug;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ModifierState {
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
}

impl ModifierState {
    pub fn is_empty(&self) -> bool {
        !self.shift && !self.ctrl && !self.alt
    }
}

pub fn is_keyboard_device(dev: &Device) -> bool {
    let name = dev.name().unwrap_or("");
    if name.contains("kleek Virtual") {
        return false;
    }

    if let Some(keys) = dev.supported_keys() {
        return keys.contains(Key::KEY_LEFTSHIFT)
            || keys.contains(Key::KEY_LEFTCTRL)
            || keys.contains(Key::KEY_LEFTALT);
    }

    false
}

pub fn scan_keyboard_nodes() -> Vec<PathBuf> {
    let mut nodes = Vec::new();
    if let Ok(entries) = std::fs::read_dir("/dev/input") {
        for entry in entries.flatten() {
            let path = entry.path();
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if name.starts_with("event") {
                    if let Ok(dev) = Device::open(&path) {
                        if is_keyboard_device(&dev) {
                            nodes.push(path);
                        }
                    }
                }
            }
        }
    }
    nodes
}

pub struct ModifierTracker {
    keyboard_nodes: Vec<PathBuf>,
}

impl ModifierTracker {
    pub fn new() -> Self {
        Self {
            keyboard_nodes: scan_keyboard_nodes(),
        }
    }

    pub fn rescan(&mut self) {
        self.keyboard_nodes = scan_keyboard_nodes();
    }

    pub fn current_modifiers(&self) -> ModifierState {
        let mut state = ModifierState::default();

        for path in &self.keyboard_nodes {
            if let Ok(dev) = Device::open(path) {
                if let Ok(keys) = dev.get_key_state() {
                    if keys.contains(Key::KEY_LEFTSHIFT) || keys.contains(Key::KEY_RIGHTSHIFT) {
                        state.shift = true;
                    }
                    if keys.contains(Key::KEY_LEFTCTRL) || keys.contains(Key::KEY_RIGHTCTRL) {
                        state.ctrl = true;
                    }
                    if keys.contains(Key::KEY_LEFTALT) || keys.contains(Key::KEY_RIGHTALT) {
                        state.alt = true;
                    }
                }
            }
        }

        if !state.is_empty() {
            debug!(
                "Active modifiers: shift={}, ctrl={}, alt={}",
                state.shift, state.ctrl, state.alt
            );
        }

        state
    }
}

impl Default for ModifierTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_modifier_state_defaults() {
        let state = ModifierState::default();
        assert!(state.is_empty());
        assert!(!state.shift);
        assert!(!state.ctrl);
        assert!(!state.alt);
    }
}
