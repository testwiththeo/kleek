use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensorConfig {
    pub dpi: u16,
    pub polling_rate: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScrollConfig {
    pub smooth: bool,
    pub natural_direction: bool,
    pub step_size: f64,
    pub duration: f64,
    pub max_gain: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MouseConfigFile {
    pub sensor: SensorConfig,
    pub scroll: ScrollConfig,
}

pub const TEMPLATE_DEFAULT: &str = r#"# kleek default profile
[sensor]
dpi = 1600
polling_rate = 1000

[scroll]
smooth = true
natural_direction = true
step_size = 40.0
duration = 0.25
max_gain = 2.5
"#;

pub const TEMPLATE_GAMING: &str = r#"# kleek esports / gaming profile
[sensor]
dpi = 800
polling_rate = 1000

[scroll]
smooth = false
natural_direction = false
step_size = 120.0
duration = 0.05
max_gain = 1.0
"#;

pub const TEMPLATE_PRODUCTIVITY: &str = r#"# kleek productivity profile
[sensor]
dpi = 1200
polling_rate = 500

[scroll]
smooth = true
natural_direction = true
step_size = 28.0
duration = 0.35
max_gain = 3.2
"#;

pub fn get_template_content(name: &str) -> Option<&'static str> {
    match name {
        "default" => Some(TEMPLATE_DEFAULT),
        "gaming" => Some(TEMPLATE_GAMING),
        "productivity" => Some(TEMPLATE_PRODUCTIVITY),
        _ => None,
    }
}
