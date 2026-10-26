use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tracing::debug;

#[derive(Debug, Clone)]
pub struct AppFilter {
    excluded_apps: Vec<String>,
    cache: Arc<Mutex<(Instant, bool)>>,
}

impl AppFilter {
    pub fn new(excluded_apps: Vec<String>) -> Self {
        Self {
            excluded_apps: excluded_apps
                .into_iter()
                .map(|s| s.to_lowercase())
                .collect(),
            cache: Arc::new(Mutex::new((Instant::now() - Duration::from_secs(1), false))),
        }
    }

    pub fn is_excluded(&self) -> bool {
        if self.excluded_apps.is_empty() {
            return false;
        }

        let now = Instant::now();
        if let Ok(mut cache_guard) = self.cache.lock() {
            if now.duration_since(cache_guard.0) < Duration::from_millis(250) {
                return cache_guard.1;
            }

            let is_match = match get_active_app_name() {
                Some(active) => {
                    let active_lower = active.to_lowercase();
                    let matched = self
                        .excluded_apps
                        .iter()
                        .any(|app| active_lower.contains(app));
                    if matched {
                        debug!("Active window '{}' matches exclusion list", active);
                    }
                    matched
                }
                None => false,
            };

            *cache_guard = (now, is_match);
            is_match
        } else {
            false
        }
    }
}

impl Default for AppFilter {
    fn default() -> Self {
        Self::new(vec![
            "blender".into(),
            "steam".into(),
            "factorio".into(),
            "freecad".into(),
            "kicad".into(),
        ])
    }
}

pub fn get_active_app_name() -> Option<String> {
    // 1. Try Hyprland if active
    if std::env::var_os("HYPRLAND_INSTANCE_SIGNATURE").is_some() {
        if let Ok(output) = std::process::Command::new("hyprctl")
            .args(["activewindow", "-j"])
            .output()
        {
            if output.status.success() {
                let text = String::from_utf8_lossy(&output.stdout);
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) {
                    if let Some(class) = json.get("class").and_then(|c| c.as_str()) {
                        if !class.is_empty() {
                            return Some(class.to_string());
                        }
                    }
                }
            }
        }
    }

    // 2. Try Sway if active
    if std::env::var_os("SWAYSOCK").is_some() {
        if let Ok(output) = std::process::Command::new("swaymsg")
            .args(["-t", "get_tree"])
            .output()
        {
            if output.status.success() {
                let text = String::from_utf8_lossy(&output.stdout);
                if let Some(pos) = text.find("\"focused\":true") {
                    let slice = &text[..pos];
                    if let Some(app_idx) = slice.rfind("\"app_id\":\"") {
                        let rem = &slice[app_idx + 10..];
                        if let Some(end) = rem.find('"') {
                            return Some(rem[..end].to_string());
                        }
                    }
                }
            }
        }
    }

    // 3. Try xdotool (works on X11 and Xwayland)
    if let Ok(output) = std::process::Command::new("xdotool")
        .args(["getactivewindow", "getwindowclassname"])
        .output()
    {
        if output.status.success() {
            let class = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !class.is_empty() {
                return Some(class);
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_filter_exclusion() {
        let filter = AppFilter::new(vec!["blender".into(), "steam".into()]);
        assert_eq!(filter.excluded_apps.len(), 2);
    }
}
