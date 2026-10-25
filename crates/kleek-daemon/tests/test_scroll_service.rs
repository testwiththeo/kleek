use kleek_daemon::physics::{ScrollAccumulator, ScrollPhysicsConfig};

#[test]
fn test_scroll_emission_sequence() {
    let mut acc = ScrollAccumulator::new(ScrollPhysicsConfig::default());
    acc.inject_impulse(1.0, 150);
    let mut total = 0;
    while acc.is_active() {
        total += acc.tick(0.00833);
    }
    assert!(total >= 100);
}
