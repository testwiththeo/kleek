use kleek_driver::{filter_mouse_devices, RawDeviceInfo};

#[test]
fn test_filter_mouse_devices_empty() {
    let devices = filter_mouse_devices(&[]);
    assert!(devices.is_empty());
}

#[test]
fn test_filter_mouse_devices_selection() {
    let inputs = vec![
        RawDeviceInfo {
            vendor_id: 0x1234,
            product_id: 0x5678,
            usage_page: 0x01,
            usage: 0x02, // Mouse
            manufacturer: Some("Test".into()),
            product: Some("Mouse".into()),
            serial_number: None,
        },
        RawDeviceInfo {
            vendor_id: 0x1234,
            product_id: 0x5679,
            usage_page: 0x01,
            usage: 0x06, // Keyboard
            manufacturer: Some("Test".into()),
            product: Some("Keyboard".into()),
            serial_number: None,
        },
    ];

    let filtered = filter_mouse_devices(&inputs);
    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].product_id, 0x5678);
}
