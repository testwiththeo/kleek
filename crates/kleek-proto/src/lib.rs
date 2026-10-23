use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ProtoError {
    #[error("packet length mismatch: expected {expected}, got {actual}")]
    LengthMismatch { expected: usize, actual: usize },
    #[error("invalid checksum: expected 0x{expected:04x}, got 0x{actual:04x}")]
    InvalidChecksum { expected: u16, actual: u16 },
    #[error("unknown command opcode: 0x{0:02x}")]
    UnknownOpcode(u8),
    #[error("invalid parameter: {0}")]
    InvalidParameter(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PollingRate {
    Hz125,
    Hz250,
    Hz500,
    Hz1000,
    Hz2000,
    Hz4000,
    Hz8000,
}

impl PollingRate {
    pub fn as_hz(&self) -> u16 {
        match self {
            Self::Hz125 => 125,
            Self::Hz250 => 250,
            Self::Hz500 => 500,
            Self::Hz1000 => 1000,
            Self::Hz2000 => 2000,
            Self::Hz4000 => 4000,
            Self::Hz8000 => 8000,
        }
    }

    pub fn from_hz(hz: u16) -> Result<Self, ProtoError> {
        match hz {
            125 => Ok(Self::Hz125),
            250 => Ok(Self::Hz250),
            500 => Ok(Self::Hz500),
            1000 => Ok(Self::Hz1000),
            2000 => Ok(Self::Hz2000),
            4000 => Ok(Self::Hz4000),
            8000 => Ok(Self::Hz8000),
            other => Err(ProtoError::InvalidParameter(format!(
                "unsupported polling rate: {other} Hz"
            ))),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RgbMode {
    Off,
    Static,
    Breathing,
    SpectrumCycle,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MouseCommand {
    SetDpi(u16),
    SetPollingRate(PollingRate),
    SetRgb { r: u8, g: u8, b: u8, mode: RgbMode },
    QueryBattery,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MouseResponse {
    Ack,
    Battery { percent: u8, is_charging: bool },
    CurrentDpi(u16),
}

pub const REPORT_SIZE: usize = 64;

impl MouseCommand {
    pub fn encode(&self) -> [u8; REPORT_SIZE] {
        let mut buf = [0u8; REPORT_SIZE];
        match self {
            Self::SetDpi(dpi) => {
                buf[0] = 0x05; // Feature report ID
                buf[1] = 0x01; // Opcode: Set DPI
                buf[2] = (dpi & 0xFF) as u8;
                buf[3] = ((dpi >> 8) & 0xFF) as u8;
            }
            Self::SetPollingRate(rate) => {
                buf[0] = 0x05;
                buf[1] = 0x02; // Opcode: Set Polling Rate
                let val: u8 = match rate {
                    PollingRate::Hz125 => 1,
                    PollingRate::Hz250 => 2,
                    PollingRate::Hz500 => 3,
                    PollingRate::Hz1000 => 4,
                    PollingRate::Hz2000 => 5,
                    PollingRate::Hz4000 => 6,
                    PollingRate::Hz8000 => 7,
                };
                buf[2] = val;
            }
            Self::SetRgb { r, g, b, mode } => {
                buf[0] = 0x05;
                buf[1] = 0x03; // Opcode: Set RGB
                buf[2] = match mode {
                    RgbMode::Off => 0,
                    RgbMode::Static => 1,
                    RgbMode::Breathing => 2,
                    RgbMode::SpectrumCycle => 3,
                };
                buf[3] = *r;
                buf[4] = *g;
                buf[5] = *b;
            }
            Self::QueryBattery => {
                buf[0] = 0x05;
                buf[1] = 0x04; // Opcode: Query Battery
            }
        }

        // Standard 16-bit word checksum over payload bytes (offset 0..62 -> 62..64)
        let checksum = calculate_checksum(&buf[..62]);
        buf[62] = (checksum & 0xFF) as u8;
        buf[63] = ((checksum >> 8) & 0xFF) as u8;

        buf
    }

    pub fn decode_response(buf: &[u8; REPORT_SIZE]) -> Result<MouseResponse, ProtoError> {
        let expected_cs = calculate_checksum(&buf[..62]);
        let actual_cs = (buf[62] as u16) | ((buf[63] as u16) << 8);
        if expected_cs != actual_cs {
            return Err(ProtoError::InvalidChecksum {
                expected: expected_cs,
                actual: actual_cs,
            });
        }

        match buf[1] {
            0x00 => Ok(MouseResponse::Ack),
            0x04 => {
                let percent = buf[2];
                let is_charging = buf[3] == 0x01;
                Ok(MouseResponse::Battery {
                    percent,
                    is_charging,
                })
            }
            0x05 => {
                let dpi = (buf[2] as u16) | ((buf[3] as u16) << 8);
                Ok(MouseResponse::CurrentDpi(dpi))
            }
            other => Err(ProtoError::UnknownOpcode(other)),
        }
    }
}

pub fn calculate_checksum(bytes: &[u8]) -> u16 {
    let mut sum: u32 = 0;
    for &byte in bytes {
        sum = sum.wrapping_add(byte as u32);
    }
    (sum & 0xFFFF) as u16
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_set_dpi_encoding() {
        let cmd = MouseCommand::SetDpi(1600);
        let encoded = cmd.encode();

        assert_eq!(encoded[0], 0x05);
        assert_eq!(encoded[1], 0x01);
        assert_eq!(encoded[2], (1600 & 0xFF) as u8);
        assert_eq!(encoded[3], ((1600 >> 8) & 0xFF) as u8);

        let cs = calculate_checksum(&encoded[..62]);
        assert_eq!(encoded[62], (cs & 0xFF) as u8);
        assert_eq!(encoded[63], ((cs >> 8) & 0xFF) as u8);
    }

    #[test]
    fn test_polling_rate_conversions() {
        assert_eq!(PollingRate::from_hz(1000).unwrap(), PollingRate::Hz1000);
        assert_eq!(PollingRate::from_hz(8000).unwrap(), PollingRate::Hz8000);
        assert!(PollingRate::from_hz(999).is_err());
    }

    #[test]
    fn test_decode_battery_response() {
        let mut buf = [0u8; REPORT_SIZE];
        buf[0] = 0x05;
        buf[1] = 0x04;
        buf[2] = 85; // 85%
        buf[3] = 0x01; // Charging

        let cs = calculate_checksum(&buf[..62]);
        buf[62] = (cs & 0xFF) as u8;
        buf[63] = ((cs >> 8) & 0xFF) as u8;

        let res = MouseCommand::decode_response(&buf).unwrap();
        assert_eq!(
            res,
            MouseResponse::Battery {
                percent: 85,
                is_charging: true
            }
        );
    }
}
