use kleek_daemon::uinput::VirtualMouseBuilder;

#[test]
fn test_virtual_mouse_builder_capabilities() {
    let builder = VirtualMouseBuilder::default();
    assert!(builder.supports_hi_res_scroll());
}
