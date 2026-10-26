use std::process::Command;
use tracing::debug;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SideScrollAction {
    #[default]
    Workspace,
    Volume,
    Tab,
    Disabled,
}

impl From<&str> for SideScrollAction {
    fn from(s: &str) -> Self {
        match s.to_lowercase().trim() {
            "workspace" => Self::Workspace,
            "volume" => Self::Volume,
            "tab" => Self::Tab,
            _ => Self::Disabled,
        }
    }
}

pub fn handle_side_scroll(action: SideScrollAction, direction: f64) {
    if direction == 0.0 || action == SideScrollAction::Disabled {
        return;
    }

    match action {
        SideScrollAction::Volume => {
            let wp_arg = if direction > 0.0 { "5%+" } else { "5%-" };
            let pactl_arg = if direction > 0.0 { "+5%" } else { "-5%" };

            let wp_status = Command::new("wpctl")
                .args(["set-volume", "@DEFAULT_AUDIO_SINK@", wp_arg])
                .status();

            let success = wp_status.map(|s| s.success()).unwrap_or(false);
            if !success {
                let _ = Command::new("pactl")
                    .args(["set-sink-volume", "@DEFAULT_SINK@", pactl_arg])
                    .status();
            }
            debug!("Gesture: adjusted system volume ({})", wp_arg);
        }
        SideScrollAction::Workspace => {
            let hypr_arg = if direction > 0.0 { "e+1" } else { "e-1" };
            let hypr_status = Command::new("hyprctl")
                .args(["dispatch", "workspace", hypr_arg])
                .status();

            let success = hypr_status.map(|s| s.success()).unwrap_or(false);
            if !success {
                let sway_arg = if direction > 0.0 { "next" } else { "prev" };
                let _ = Command::new("swaymsg")
                    .args(["workspace", sway_arg])
                    .status();
            }
            debug!("Gesture: dispatched workspace switch");
        }
        SideScrollAction::Tab => {
            let is_next = direction > 0.0;
            if Command::new("which")
                .arg("wtype")
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false)
            {
                if is_next {
                    let _ = Command::new("wtype")
                        .args(["-M", "ctrl", "-k", "Tab", "-m", "ctrl"])
                        .status();
                } else {
                    let _ = Command::new("wtype")
                        .args([
                            "-M", "ctrl", "-M", "shift", "-k", "Tab", "-m", "shift", "-m", "ctrl",
                        ])
                        .status();
                }
            } else if Command::new("which")
                .arg("xdotool")
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false)
            {
                let shortcut = if is_next {
                    "ctrl+Tab"
                } else {
                    "ctrl+shift+Tab"
                };
                let _ = Command::new("xdotool").args(["key", shortcut]).status();
            }
        }
        SideScrollAction::Disabled => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_side_action_parsing() {
        assert_eq!(
            SideScrollAction::from("workspace"),
            SideScrollAction::Workspace
        );
        assert_eq!(SideScrollAction::from("VOLUME"), SideScrollAction::Volume);
        assert_eq!(SideScrollAction::from("tab"), SideScrollAction::Tab);
        assert_eq!(SideScrollAction::from("none"), SideScrollAction::Disabled);
    }

    #[test]
    fn test_handle_side_scroll_disabled() {
        // Should not panic or execute anything
        handle_side_scroll(SideScrollAction::Disabled, 1.0);
        handle_side_scroll(SideScrollAction::Workspace, 0.0);
    }
}
