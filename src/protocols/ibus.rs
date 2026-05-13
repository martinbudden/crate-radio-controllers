use crate::{RxFrame, RxLinkStatus};

/// The iBUS protocol (by FlySky/Turnigy).
/// It is not inverted and uses a straightforward "Sum of Bytes" checksum.
///
/// It operates at 115,200 baud with a 32-byte packet transmitted every 7ms.
///
/// Packet Structure (32 Bytes)
/// Each packet contains 14 channels, each represented by a 16-bit value (2 bytes) in Little-Endian format.
///    Byte 0: Header 0x20
///    Byte 1: Command/Type 0x40 (for RC channels)
///    Bytes 2–29: 14 Channels (2 bytes each, Little-Endian)
///    Bytes 30–31: Checksum (2 bytes, Little-Endian).
///
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct IbusFrame {
    pub channels: [u16; Self::CHANNEL_COUNT],
}

impl IbusFrame {
    pub const fn new() -> Self {
        Self { channels: [0u16; Self::CHANNEL_COUNT] }
    }
}

impl Default for IbusFrame {
    fn default() -> Self {
        Self::new()
    }
}

impl IbusFrame {
    pub const CHANNEL_COUNT: usize = 14;
    pub const PACKET_LENGTH: usize = 32;

    /// The iBUS checksum is the one's complement of the sum of the first 30 bytes.
    /// Start with a value of 0xFFFF and subtract every byte from it.
    pub fn checksum(data: &[u8; Self::PACKET_LENGTH]) -> u16 {
        let mut checksum: u16 = 0xFFFF;
        for &byte in &data[..30] {
            checksum -= u16::from(byte);
        }
        checksum
    }

    pub fn parse(buffer: &[u8; Self::PACKET_LENGTH]) -> Option<Self> {
        // Verify Header
        if buffer[0] != 0x20 || buffer[1] != 0x40 {
            return None;
        }

        // Verify Checksum
        let received_checksum = u16::from_le_bytes([buffer[30], buffer[31]]);
        let calculated_checksum = Self::checksum(buffer);

        if received_checksum != calculated_checksum {
            return None;
        }

        // Extract 14 Channels (Little-Endian)
        let mut channels = [0u16; Self::CHANNEL_COUNT];

        // Skip the 2-byte header, then take 14 pairs (28 bytes)
        for (slot, chunk) in channels.iter_mut().zip(buffer[2..30].chunks_exact(2)) {
            *slot = u16::from_le_bytes([chunk[0], chunk[1]]);
        }

        Some(Self { channels })
    }
}

impl From<IbusFrame> for RxFrame {
    fn from(ibus: IbusFrame) -> Self {
        let mut channels = [Self::DEFAULT_CHANNEL_VALUE; Self::MAX_CHANNEL_COUNT];
        channels[..IbusFrame::CHANNEL_COUNT].copy_from_slice(&ibus.channels);

        let status = RxLinkStatus::Ok;
        let rssi = 0;
        Self { channels, status, rssi }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[allow(unused)]
    fn is_normal<T: Sized + Send + Sync + Unpin>() {}
    fn is_full<T: Sized + Send + Sync + Unpin + Copy + Clone + Default + PartialEq>() {}

    #[test]
    fn normal_types() {
        is_full::<IbusFrame>();
    }
    #[test]
    fn new() {
        let frame = IbusFrame::default();
        assert_eq!(0, frame.channels[0]);
    }
}
