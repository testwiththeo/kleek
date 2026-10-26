use std::sync::atomic::{AtomicU16, AtomicU8, Ordering};
use std::sync::Arc;
use zbus::interface;

pub struct KleekManager {
    device_count: usize,
    active_dpi: Arc<AtomicU16>,
    battery_level: Arc<AtomicU8>,
}

impl Default for KleekManager {
    fn default() -> Self {
        Self::new()
    }
}

impl KleekManager {
    pub fn new() -> Self {
        Self {
            device_count: 0,
            active_dpi: Arc::new(AtomicU16::new(1600)),
            battery_level: Arc::new(AtomicU8::new(100)),
        }
    }

    pub fn device_count(&self) -> usize {
        self.device_count
    }
}

#[interface(name = "org.freedesktop.Kleek1.Manager")]
impl KleekManager {
    async fn set_dpi(&self, dpi: u16) -> zbus::fdo::Result<()> {
        self.active_dpi.store(dpi, Ordering::SeqCst);
        Ok(())
    }

    async fn get_dpi(&self) -> zbus::fdo::Result<u16> {
        Ok(self.active_dpi.load(Ordering::SeqCst))
    }

    async fn get_battery(&self) -> zbus::fdo::Result<(u8, bool)> {
        Ok((self.battery_level.load(Ordering::SeqCst), false))
    }

    async fn set_smooth_scroll(&self, _enabled: bool) -> zbus::fdo::Result<()> {
        Ok(())
    }
}
