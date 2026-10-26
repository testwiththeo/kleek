#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ScrollPhysicsConfig {
    pub step_size: f64,
    pub duration_seconds: f64,
    pub max_gain: f64,
    pub speed: f64,
}

impl Default for ScrollPhysicsConfig {
    fn default() -> Self {
        Self {
            step_size: 28.0,
            duration_seconds: 0.22,
            max_gain: 1.8,
            speed: 1.15,
        }
    }
}

#[allow(dead_code)]
pub struct ScrollAccumulator {
    config: ScrollPhysicsConfig,
    target_distance: f64,
    elapsed_seconds: f64,
    last_progress: f64,
    fractional_remainder: f64,
    active_direction: f64,
}

#[allow(dead_code)]
impl ScrollAccumulator {
    pub fn new(config: ScrollPhysicsConfig) -> Self {
        Self {
            config,
            target_distance: 0.0,
            elapsed_seconds: 0.0,
            last_progress: 0.0,
            fractional_remainder: 0.0,
            active_direction: 0.0,
        }
    }

    pub fn inject_impulse(
        &mut self,
        direction: f64,
        consecutive_interval_ms: u64,
        modifiers: crate::modifier::ModifierState,
    ) {
        if modifiers.ctrl {
            // Controlled Zoom: clamp momentum to discrete single step to prevent runaway zoom
            self.target_distance = direction * 120.0;
            self.elapsed_seconds = 0.0;
            self.last_progress = 0.0;
            self.active_direction = direction;
            return;
        }

        // Check for instant direction reversal (snappy braking)
        let is_reversal = self.active_direction != 0.0 && (direction * self.active_direction) < 0.0;
        let remaining = if self.is_active() && !is_reversal {
            self.target_distance * (1.0 - self.last_progress)
        } else {
            0.0
        };
        self.active_direction = direction;

        let mut gain = if consecutive_interval_ms < 50 {
            self.config.max_gain
        } else if consecutive_interval_ms < 100 {
            1.35
        } else if consecutive_interval_ms < 180 {
            1.15
        } else {
            1.0
        };

        if modifiers.alt {
            // Precision Mode: micro speed (0.3x) for fine inspection
            gain *= 0.30;
        }

        // Standard discrete step: 120 units per notch (Mos / LinearMouse calibrated)
        let impulse_distance = direction * 120.0 * self.config.speed * gain;

        self.target_distance = remaining + impulse_distance;
        self.elapsed_seconds = 0.0;
        self.last_progress = 0.0;
    }

    pub fn tick(&mut self, delta_seconds: f64) -> i32 {
        if !self.is_active() {
            return 0;
        }

        let duration = self.config.duration_seconds.max(0.01);
        self.elapsed_seconds = (self.elapsed_seconds + delta_seconds).min(duration);

        let tau = self.elapsed_seconds / duration;
        // Cubic-bezier ease-out curve: f(tau) = 1.0 - (1.0 - tau)^3
        // Starts smoothly upon notch and decelerates with quadratic falloff to 0 terminal velocity
        let current_progress = 1.0 - (1.0 - tau).powi(3);

        let delta_progress = current_progress - self.last_progress;
        self.last_progress = current_progress;

        let delta_units = (self.target_distance * delta_progress) + self.fractional_remainder;
        let emit_units = delta_units.trunc() as i32;
        self.fractional_remainder = delta_units.fract();

        if self.elapsed_seconds >= duration {
            self.target_distance = 0.0;
            self.last_progress = 0.0;
            self.fractional_remainder = 0.0;
            self.active_direction = 0.0;
        }

        emit_units
    }

    pub fn is_active(&self) -> bool {
        self.elapsed_seconds < self.config.duration_seconds && self.target_distance.abs() > 0.01
    }

    pub fn target_distance(&self) -> f64 {
        self.target_distance
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scroll_physics_decay() {
        let mut acc = ScrollAccumulator::new(ScrollPhysicsConfig::default());
        acc.inject_impulse(1.0, 200, crate::modifier::ModifierState::default());

        assert!(acc.is_active());

        let mut total_emitted = 0;
        for _ in 0..400 {
            total_emitted += acc.tick(0.00833); // 8.33ms ticks (~120Hz)
        }

        assert!(!acc.is_active());
        assert!(total_emitted > 0);
    }

    #[test]
    fn test_speed_gain_scaling() {
        let mut slow_acc = ScrollAccumulator::new(ScrollPhysicsConfig::default());
        slow_acc.inject_impulse(1.0, 200, crate::modifier::ModifierState::default());

        let mut fast_acc = ScrollAccumulator::new(ScrollPhysicsConfig::default());
        fast_acc.inject_impulse(1.0, 40, crate::modifier::ModifierState::default());

        assert!(
            fast_acc.target_distance() > slow_acc.target_distance(),
            "Fast flicking must produce higher velocity gain than slow precision scrolling"
        );
    }

    #[test]
    fn test_direction_reversal_braking() {
        let mut acc = ScrollAccumulator::new(ScrollPhysicsConfig::default());
        acc.inject_impulse(1.0, 40, crate::modifier::ModifierState::default());
        assert!(acc.target_distance() > 0.0);

        // Immediate reverse direction
        acc.inject_impulse(-1.0, 30, crate::modifier::ModifierState::default());
        assert!(
            acc.target_distance() < 0.0,
            "Reversing wheel direction must instantly cancel previous opposite momentum"
        );
    }

    #[test]
    fn test_precision_mode_alt_modifier() {
        let mut normal_acc = ScrollAccumulator::new(ScrollPhysicsConfig::default());
        normal_acc.inject_impulse(1.0, 250, crate::modifier::ModifierState::default());

        let mut precision_acc = ScrollAccumulator::new(ScrollPhysicsConfig::default());
        let precision_mod = crate::modifier::ModifierState {
            alt: true,
            ..Default::default()
        };
        precision_acc.inject_impulse(1.0, 250, precision_mod);

        assert!(precision_acc.target_distance < normal_acc.target_distance);
        assert!((precision_acc.target_distance / normal_acc.target_distance - 0.30).abs() < 0.05);
    }

    #[test]
    fn test_controlled_zoom_ctrl_modifier() {
        let mut ctrl_acc = ScrollAccumulator::new(ScrollPhysicsConfig::default());
        let ctrl_mod = crate::modifier::ModifierState {
            ctrl: true,
            ..Default::default()
        };
        ctrl_acc.inject_impulse(1.0, 150, ctrl_mod);

        assert!(ctrl_acc.is_active());
        assert_eq!(ctrl_acc.target_distance, 120.0);
    }
}
