pub mod hotplug;
pub use hotplug::{filter_mouse_devices, DeviceEvent, RawDeviceInfo};

use kleek_proto::{MouseCommand, MouseResponse, ProtoError, REPORT_SIZE};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DriverError {
    #[error("HID API error: {0}")]
    Hid(#[from] hidapi::HidError),
    #[error("Protocol error: {0}")]
    Proto(#[from] ProtoError),
    #[error("Device not found: VID 0x{vid:04x}, PID 0x{pid:04x}")]
    NotFound { vid: u16, pid: u16 },
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeviceInfo {
    pub vendor_id: u16,
    pub product_id: u16,
    pub manufacturer: Option<String>,
    pub product_name: Option<String>,
    pub serial_number: Option<String>,
}

pub struct MouseDevice {
    info: DeviceInfo,
    handle: Option<hidapi::HidDevice>,
}

impl MouseDevice {
    pub fn new(info: DeviceInfo, handle: hidapi::HidDevice) -> Self {
        Self {
            info,
            handle: Some(handle),
        }
    }

    pub fn info(&self) -> &DeviceInfo {
        &self.info
    }

    pub fn send_command(&mut self, cmd: &MouseCommand) -> Result<MouseResponse, DriverError> {
        let handle = self.handle.as_mut().ok_or(DriverError::NotFound {
            vid: self.info.vendor_id,
            pid: self.info.product_id,
        })?;

        let report = cmd.encode();
        handle.send_feature_report(&report)?;

        let mut read_buf = [0u8; REPORT_SIZE];
        read_buf[0] = report[0];
        handle.get_feature_report(&mut read_buf)?;

        let response = MouseCommand::decode_response(&read_buf)?;
        Ok(response)
    }
}
