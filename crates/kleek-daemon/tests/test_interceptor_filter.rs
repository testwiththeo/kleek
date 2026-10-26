use kleek_daemon::interceptor::scan_mouse_event_nodes;

#[test]
fn test_scan_mouse_event_nodes_runs_without_panic() {
    let nodes = scan_mouse_event_nodes();
    // In any environment, function should execute without panic and filter safely
    println!("Detected mouse event nodes: {:?}", nodes);
}
