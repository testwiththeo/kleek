use crate::modifier::ModifierState;
use crate::physics::{ScrollAccumulator, ScrollPhysicsConfig};
use crate::uinput::VirtualMouse;
use anyhow::Result;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;

pub struct ScrollImpulse {
    pub direction: f64,
    pub interval_ms: u64,
    pub modifiers: ModifierState,
}

pub struct ScrollService;

impl ScrollService {
    pub fn spawn(
        mut rx: mpsc::Receiver<ScrollImpulse>,
        mut virtual_mouse: VirtualMouse,
        config: ScrollPhysicsConfig,
        app_filter: crate::app_filter::AppFilter,
    ) -> JoinHandle<Result<()>> {
        tokio::spawn(async move {
            let mut v_accumulator = ScrollAccumulator::new(config.clone());
            let mut h_accumulator = ScrollAccumulator::new(config);
            let mut interval = tokio::time::interval(Duration::from_micros(8333)); // ~120 Hz

            loop {
                tokio::select! {
                    Some(impulse) = rx.recv() => {
                        if app_filter.is_excluded() {
                            // Excluded app active (e.g. game, CAD, terminal): bypass smooth momentum gliding!
                            // Emit immediate discrete notch 1:1
                            let units = (impulse.direction * 120.0) as i32;
                            if impulse.modifiers.shift {
                                let _ = virtual_mouse.emit_hscroll_hi_res(units);
                            } else {
                                let _ = virtual_mouse.emit_scroll_hi_res(units);
                            }
                        } else if impulse.modifiers.shift {
                            // Shift + Scroll -> Convert vertical wheel impulse into smooth horizontal scroll
                            h_accumulator.inject_impulse(impulse.direction, impulse.interval_ms, impulse.modifiers);
                        } else {
                            v_accumulator.inject_impulse(impulse.direction, impulse.interval_ms, impulse.modifiers);
                        }
                    }
                    _ = interval.tick() => {
                        if v_accumulator.is_active() {
                            let units = v_accumulator.tick(0.008333);
                            if units != 0 {
                                virtual_mouse.emit_scroll_hi_res(units)?;
                            }
                        }
                        if h_accumulator.is_active() {
                            let units = h_accumulator.tick(0.008333);
                            if units != 0 {
                                virtual_mouse.emit_hscroll_hi_res(units)?;
                            }
                        }
                    }
                }
            }
        })
    }
}
