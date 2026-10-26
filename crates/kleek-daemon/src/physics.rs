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
    velocity: f64,
    fractional_remainder: f64,
    damping_factor: f64,
}

#[allow(dead_code)]
impl ScrollAccumulator {
    pub fn new(config: ScrollPhysicsConfig) -> Self {
        // λ = 1.0 / duration ensures total integrated distance = step_size
        let damping_factor = 1.0 / config.duration_seconds.max(0.01);
        Self {
            config,
            velocity: 0.0,
            fractional_remainder: 0.0,
            damping_factor,
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
            let impulse = direction * self.config.step_size * self.damping_factor * 1.0;
            self.velocity = impulse;
            return;
        }

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

        // Controlled 1:1 speed with smooth fluid decay for productivity
        let impulse =
            direction * self.config.step_size * self.config.speed * self.damping_factor * gain;
        self.velocity += impulse;
    }

    pub fn tick(&mut self, delta_seconds: f64) -> i32 {
        if self.velocity.abs() < 0.5 {
            self.velocity = 0.0;
            self.fractional_remainder = 0.0;
            return 0;
        }

        // Apply decay factor to velocity
        let decay = (-self.damping_factor * delta_seconds).exp();
        let current_v = self.velocity;
        self.velocity *= decay;

        // Integral distance traveled during this interval
        let distance = current_v * (1.0 - decay) / self.damping_factor;

        let total_units = (distance * (120.0 / self.config.step_size)) + self.fractional_remainder;
        let emit_units = total_units.trunc() as i32;
        self.fractional_remainder = total_units.fract();

        emit_units
    }

    pub fn is_active(&self) -> bool {
        self.velocity.abs() >= 0.5
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
            total_emitted += acc.tick(0.008); // 8ms ticks (~120Hz)
        }

        assert!(!acc.is_active());
        assert!(total_emitted > 0);
    }

    #[test]
    fn test_precision_mode_alt_modifier() {
        let mut normal_acc = ScrollAccumulator::new(ScrollPhysicsConfig::default());
        normal_acc.inject_impulse(1.0, 150, crate::modifier::ModifierState::default());

        let mut precision_acc = ScrollAccumulator::new(ScrollPhysicsConfig::default());
        let precision_mod = crate::modifier::ModifierState {
            alt: true,
            ..Default::default()
        };
        precision_acc.inject_impulse(1.0, 150, precision_mod);

        assert!(precision_acc.velocity < normal_acc.velocity);
        assert!((precision_acc.velocity / normal_acc.velocity - 0.30).abs() < 0.05);
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
    }
}
