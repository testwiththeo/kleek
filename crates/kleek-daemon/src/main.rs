use anyhow::Result;
use kleek_daemon::physics::{ScrollAccumulator, ScrollPhysicsConfig};
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    info!("Starting kleek daemon (kleekd)...");

    let _scroll_engine = ScrollAccumulator::new(ScrollPhysicsConfig::default());
    info!("Smooth scroll physics engine initialized.");

    let manager = kleek_daemon::dbus::KleekManager::new();
    let _conn = zbus::connection::Builder::session()?
        .name("org.freedesktop.Kleek1")?
        .serve_at("/org/freedesktop/Kleek1", manager)?
        .build()
        .await?;

    info!("kleekd D-Bus endpoint org.freedesktop.Kleek1 registered successfully.");
    Ok(())
}
