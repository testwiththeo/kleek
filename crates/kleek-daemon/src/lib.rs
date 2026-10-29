pub mod app_filter;
pub mod dbus;
pub mod gestures;
pub mod interceptor;
pub mod modifier;
pub mod physics;
pub mod scroll_service;
pub mod uinput;

pub fn load_startup_scroll_config() -> physics::ScrollPhysicsConfig {
    let mut config = physics::ScrollPhysicsConfig::default();
    if let Some(home) = std::env::var_os("HOME").map(std::path::PathBuf::from) {
        let cfg_path = home.join(".config").join("kleek").join("config.toml");
        if let Ok(content) = std::fs::read_to_string(cfg_path) {
            if let Ok(val) = content.parse::<toml::Value>() {
                if let Some(scroll) = val.get("scroll") {
                    if let Some(step) = scroll.get("step_size").and_then(|v| v.as_float()) {
                        config.step_size = step;
                    }
                    if let Some(dur) = scroll.get("duration").and_then(|v| v.as_float()) {
                        config.duration_seconds = dur;
                    }
                    if let Some(gain) = scroll.get("max_gain").and_then(|v| v.as_float()) {
                        config.max_gain = gain;
                    }
                    if let Some(spd) = scroll.get("speed").and_then(|v| v.as_float()) {
                        config.speed = spd;
                    }
                }
            }
        }
    }
    config
}
