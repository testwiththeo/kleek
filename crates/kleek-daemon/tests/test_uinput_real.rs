use kleek_daemon::uinput::VirtualMouseBuilder;

#[test]
fn test_uinput_real_fallback_and_builder() {
    let builder = VirtualMouseBuilder::default();
    assert!(builder.supports_hi_res_scroll());

    let mut vm = kleek_daemon::uinput::VirtualMouse::new(builder).unwrap();
    // In CI or rootless environments without uinput access, fallback initializes gracefully
    let _ = vm.is_real_device_connected();
    let res = vm.emit_hscroll_hi_res(120);
    assert!(res.is_ok());
}
