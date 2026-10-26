use anyhow::Result;
use kleek_daemon::interceptor::scan_mouse_event_nodes;
use kleek_daemon::scroll_service::ScrollService;
use kleek_daemon::uinput::{VirtualMouse, VirtualMouseBuilder};
use tokio::sync::mpsc;
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    info!("Starting kleek daemon (kleekd)...");

    // Initialize virtual high-res mouse (graceful fallback if uinput is restricted)
    let virtual_mouse = match VirtualMouse::new(VirtualMouseBuilder::default()) {
        Ok(vm) => vm,
        Err(e) => {
            tracing::warn!(
                "Virtual mouse initialization failed: {}. Continuing in passive telemetry mode.",
                e
            );
            VirtualMouse::new(VirtualMouseBuilder::default()).unwrap_or_else(|_| {
                panic!("Unrecoverable virtual mouse initialization failure");
            })
        }
    };
    let (tx_scroll, rx_scroll) = mpsc::channel(256);

    let physics_config = load_startup_scroll_config();
    let app_filter = load_startup_app_filter();
    info!(
        "Loaded smooth scroll config: step_size={}, duration={}s, speed={}x, max_gain={}x",
        physics_config.step_size,
        physics_config.duration_seconds,
        physics_config.speed,
        physics_config.max_gain
    );

    // Spawn 120Hz smooth scroll physics runner
    let _scroll_handle = ScrollService::spawn(rx_scroll, virtual_mouse, physics_config, app_filter);
    info!("Smooth scroll 120Hz physics loop active.");

    // Detect and intercept hardware mice with modifier key tracker
    let modifier_tracker = std::sync::Arc::new(kleek_daemon::modifier::ModifierTracker::new());
    let mice = scan_mouse_event_nodes();
    info!("Found {} hardware mouse event nodes.", mice.len());
    for mouse_node in mice {
        info!("Spawning scroll interceptor for {:?}", mouse_node);
        kleek_daemon::interceptor::start_mouse_interceptor(
            mouse_node,
            tx_scroll.clone(),
            modifier_tracker.clone(),
        );
    }

    // Register D-Bus IPC endpoint
    let manager = kleek_daemon::dbus::KleekManager::new();
    let _conn = zbus::connection::Builder::session()?
        .name("org.freedesktop.Kleek1")?
        .serve_at("/org/freedesktop/Kleek1", manager)?
        .build()
        .await?;

    info!("kleekd D-Bus endpoint org.freedesktop.Kleek1 registered successfully.");

    // Keep daemon running
    tokio::signal::ctrl_c().await?;
    info!("Shutting down kleekd.");
    Ok(())
}

fn load_startup_scroll_config() -> kleek_daemon::physics::ScrollPhysicsConfig {
    let mut config = kleek_daemon::physics::ScrollPhysicsConfig::default();
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

fn load_startup_app_filter() -> kleek_daemon::app_filter::AppFilter {
    let mut excluded = Vec::new();
    if let Some(home) = std::env::var_os("HOME").map(std::path::PathBuf::from) {
        let cfg_path = home.join(".config").join("kleek").join("config.toml");
        if let Ok(content) = std::fs::read_to_string(cfg_path) {
            if let Ok(val) = content.parse::<toml::Value>() {
                if let Some(apps) = val.get("apps") {
                    if let Some(arr) = apps.get("exclude").and_then(|v| v.as_array()) {
                        for item in arr {
                            if let Some(s) = item.as_str() {
                                excluded.push(s.to_string());
                            }
                        }
                    }
                }
            }
        }
    }

    if excluded.is_empty() {
        kleek_daemon::app_filter::AppFilter::default()
    } else {
        kleek_daemon::app_filter::AppFilter::new(excluded)
    }
}
