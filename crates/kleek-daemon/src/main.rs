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
            tracing::warn!("Virtual mouse initialization failed: {}. Continuing in passive telemetry mode.", e);
            VirtualMouse::new(VirtualMouseBuilder::default()).unwrap_or_else(|_| {
                panic!("Unrecoverable virtual mouse initialization failure");
            })
        }
    };
    let (tx_scroll, rx_scroll) = mpsc::channel(256);

    // Spawn 120Hz smooth scroll physics runner
    let _scroll_handle = ScrollService::spawn(rx_scroll, virtual_mouse);
    info!("Smooth scroll 120Hz physics loop active.");

    // Detect and intercept hardware mice
    let mice = scan_mouse_event_nodes();
    info!("Found {} hardware mouse event nodes.", mice.len());
    for mouse_node in mice {
        info!("Spawning scroll interceptor for {:?}", mouse_node);
        kleek_daemon::interceptor::start_mouse_interceptor(mouse_node, tx_scroll.clone());
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
