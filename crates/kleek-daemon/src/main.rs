use anyhow::Result;
use kleek_daemon::physics::{ScrollAccumulator, ScrollPhysicsConfig};
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    info!("Starting kleek daemon (kleekd)...");

    // Initialize smooth scroll physics engine
    let _scroll_engine = ScrollAccumulator::new(ScrollPhysicsConfig::default());
    info!("Smooth scroll physics engine initialized.");

    info!("kleekd ready.");
    Ok(())
}
