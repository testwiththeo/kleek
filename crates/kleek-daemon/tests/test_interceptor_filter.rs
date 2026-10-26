use kleek_daemon::interceptor::{is_touchpad_device, scan_mouse_event_nodes};

#[test]
fn test_scan_mouse_event_nodes_runs_without_panic() {
    let nodes = scan_mouse_event_nodes();
    // In any environment, function should execute without panic and filter safely
    println!("Detected mouse event nodes: {:?}", nodes);
}

#[test]
fn test_touchpad_filtering_safety() {
    // Synthetic check: verify function handles mock device check without errors
    let nodes = scan_mouse_event_nodes();
    for node in nodes {
        if let Ok(dev) = evdev::Device::open(node) {
            assert!(
                !is_touchpad_device(&dev),
                "Touchpad device was not excluded!"
            );
        }
    }
}
