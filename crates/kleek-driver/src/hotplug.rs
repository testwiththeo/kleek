use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeviceEvent {
    Connected(crate::DeviceInfo),
    Disconnected(PathBuf),
}

pub struct RawDeviceInfo {
    pub vendor_id: u16,
    pub product_id: u16,
    pub usage_page: u16,
    pub usage: u16,
    pub manufacturer: Option<String>,
    pub product: Option<String>,
    pub serial_number: Option<String>,
}

pub fn filter_mouse_devices(devices: &[RawDeviceInfo]) -> Vec<crate::DeviceInfo> {
    devices
        .iter()
        .filter(|d| {
            // Usage Page 0x01 = Generic Desktop Controls, Usage 0x02 = Mouse
            (d.usage_page == 0x01 && d.usage == 0x02) || (d.usage_page == 0x00 && d.usage == 0x00)
        })
        .map(|d| crate::DeviceInfo {
            vendor_id: d.vendor_id,
            product_id: d.product_id,
            manufacturer: d.manufacturer.clone(),
            product_name: d.product.clone(),
            serial_number: d.serial_number.clone(),
        })
        .collect()
}
