use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensorConfig {
    pub dpi: u16,
    pub polling_rate: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PointerConfig {
    pub accel_profile: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScrollConfig {
    pub smooth: bool,
    pub natural_direction: bool,
    pub step_size: f64,
    pub duration: f64,
    pub max_gain: f64,
    #[serde(default)]
    pub speed: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MouseConfigFile {
    pub sensor: SensorConfig,
    pub pointer: Option<PointerConfig>,
    pub scroll: ScrollConfig,
}

pub const TEMPLATE_DEFAULT: &str = r#"# kleek default profile
[sensor]
dpi = 1600
polling_rate = 1000

[pointer]
accel_profile = "flat"

[scroll]
smooth = true
natural_direction = true
step_size = 30.0
duration = 0.24
max_gain = 2.0
speed = 1.2
"#;

pub const TEMPLATE_GAMING: &str = r#"# kleek esports / gaming profile
[sensor]
dpi = 800
polling_rate = 1000

[pointer]
accel_profile = "flat"

[scroll]
smooth = false
natural_direction = false
step_size = 120.0
duration = 0.05
max_gain = 1.0
speed = 1.0
"#;

pub const TEMPLATE_PRODUCTIVITY: &str = r#"# kleek productivity profile
[sensor]
dpi = 1000
polling_rate = 500

[pointer]
accel_profile = "flat"

[scroll]
smooth = true
natural_direction = true
step_size = 28.0
duration = 0.22
max_gain = 1.8
speed = 1.15
"#;

pub fn get_template_content(name: &str) -> Option<&'static str> {
    match name {
        "default" => Some(TEMPLATE_DEFAULT),
        "gaming" => Some(TEMPLATE_GAMING),
        "productivity" => Some(TEMPLATE_PRODUCTIVITY),
        _ => None,
    }
}
