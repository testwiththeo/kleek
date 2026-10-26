use kleek_daemon::modifier::ModifierState;
use kleek_daemon::physics::{ScrollAccumulator, ScrollPhysicsConfig};

#[test]
fn test_scroll_emission_sequence() {
    let mut acc = ScrollAccumulator::new(ScrollPhysicsConfig::default());
    acc.inject_impulse(1.0, 150, ModifierState::default());
    let mut total = 0;
    while acc.is_active() {
        total += acc.tick(0.00833);
    }
    assert!(total >= 100);
}

#[test]
fn test_scroll_with_modifiers() {
    let mut acc = ScrollAccumulator::new(ScrollPhysicsConfig::default());
    let mods = ModifierState {
        alt: true,
        ..Default::default()
    };
    acc.inject_impulse(1.0, 150, mods);
    assert!(acc.is_active());
}

#[tokio::test]
async fn test_scroll_service_spawn_with_app_filter() {
    use kleek_daemon::app_filter::AppFilter;
    use kleek_daemon::scroll_service::{ScrollImpulse, ScrollService};
    use kleek_daemon::uinput::{VirtualMouse, VirtualMouseBuilder};
    use tokio::sync::mpsc;

    let (tx, rx) = mpsc::channel(16);
    let vm = VirtualMouse::new(VirtualMouseBuilder::default()).unwrap();
    let filter = AppFilter::new(vec!["blender".into()]);

    let handle = ScrollService::spawn(rx, vm, ScrollPhysicsConfig::default(), filter);

    let impulse = ScrollImpulse {
        direction: 1.0,
        interval_ms: 100,
        modifiers: ModifierState::default(),
    };
    assert!(tx.send(impulse).await.is_ok());

    handle.abort();
}
