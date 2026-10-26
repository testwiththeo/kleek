use kleek_cli::config::{get_template_content, MouseConfigFile};

#[test]
fn test_default_template_content() {
    let content = get_template_content("default").unwrap();
    assert!(content.contains("[sensor]"));
    assert!(content.contains("dpi = 1600"));

    let parsed: MouseConfigFile = toml::from_str(content).unwrap();
    assert_eq!(parsed.sensor.dpi, 1600);
    assert_eq!(parsed.sensor.polling_rate, 1000);
    assert_eq!(
        parsed.pointer.and_then(|p| p.accel_profile),
        Some("flat".into())
    );
    assert!(parsed.scroll.smooth);
    assert!(parsed.apps.is_some());
    assert!(parsed.apps.unwrap().exclude.contains(&"blender".into()));
}

#[test]
fn test_gaming_template_content() {
    let content = get_template_content("gaming").unwrap();
    let parsed: MouseConfigFile = toml::from_str(content).unwrap();
    assert_eq!(parsed.sensor.dpi, 800);
    assert!(!parsed.scroll.smooth);
}

#[test]
fn test_productivity_template_content() {
    let content = get_template_content("productivity").unwrap();
    let parsed: MouseConfigFile = toml::from_str(content).unwrap();
    assert_eq!(parsed.sensor.dpi, 1000);
    assert_eq!(parsed.scroll.step_size, 28.0);
    assert_eq!(parsed.scroll.speed, Some(1.15));
    assert!(parsed.scroll.smooth);
}
