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
            step_size: 35.0,
            duration_seconds: 0.45,
            max_gain: 4.8,
            speed: 3.5,
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

    pub fn inject_impulse(&mut self, direction: f64, consecutive_interval_ms: u64) {
        let gain = if consecutive_interval_ms < 45 {
            self.config.max_gain
        } else if consecutive_interval_ms < 90 {
            2.4
        } else if consecutive_interval_ms < 160 {
            1.6
        } else {
            1.0
        };

        // Mac / Mos style: speed multiplier gives readable travel per notch with fluid decay
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
        acc.inject_impulse(1.0, 200);

        assert!(acc.is_active());

        let mut total_emitted = 0;
        for _ in 0..400 {
            total_emitted += acc.tick(0.008); // 8ms ticks (~120Hz)
        }

        assert!(!acc.is_active());
        assert!(total_emitted > 0);
    }
}
