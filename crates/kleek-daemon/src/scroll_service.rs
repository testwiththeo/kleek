use crate::physics::{ScrollAccumulator, ScrollPhysicsConfig};
use crate::uinput::VirtualMouse;
use anyhow::Result;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;

pub struct ScrollImpulse {
    pub direction: f64,
    pub interval_ms: u64,
}

pub struct ScrollService;

impl ScrollService {
    pub fn spawn(
        mut rx: mpsc::Receiver<ScrollImpulse>,
        mut virtual_mouse: VirtualMouse,
        config: ScrollPhysicsConfig,
    ) -> JoinHandle<Result<()>> {
        tokio::spawn(async move {
            let mut accumulator = ScrollAccumulator::new(config);
            let mut interval = tokio::time::interval(Duration::from_micros(8333)); // ~120 Hz

            loop {
                tokio::select! {
                    Some(impulse) = rx.recv() => {
                        accumulator.inject_impulse(impulse.direction, impulse.interval_ms);
                    }
                    _ = interval.tick() => {
                        if accumulator.is_active() {
                            let units = accumulator.tick(0.008333);
                            if units != 0 {
                                virtual_mouse.emit_scroll_hi_res(units)?;
                            }
                        }
                    }
                }
            }
        })
    }
}
