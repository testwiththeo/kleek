use kleek_daemon::dbus::KleekManager;

#[test]
fn test_dbus_manager_initial_state() {
    let mgr = KleekManager::new();
    assert_eq!(mgr.device_count(), 0);
}
