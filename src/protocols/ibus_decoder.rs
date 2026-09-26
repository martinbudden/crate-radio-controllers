use crate::{RxFrame, RxFrameType, RxLinkStatus};

/// The iBUS protocol (by FlySky/Turnigy).
/// It is not inverted and uses a straightforward "Sum of Bytes" checksum.
///
/// It operates at 115,200 baud with a 32-byte packet transmitted every 7ms.
///
/// Packet Structure (32 Bytes)
/// Each packet contains 14 channels, each represented by a 16-bit value (2 bytes) in Little-Endian format.
///    Byte 0: Header 0x20 (Size)
///    Byte 1: Command/Type 0x40 (for RC channels)
///    Bytes 2–29: 14 Channels (2 bytes each, Little-Endian)
///    Bytes 30–31: Checksum (2 bytes, Little-Endian).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum State {
    #[default]
    WaitForSizeByte,
    WaitForCommandByte,
    ReadPayload {
        index: usize,
        checksum: u16,
    },
    ValidateChecksum {
        checksum: u16,
        is_first_byte: bool,
        checksum_first_byte: u8,
    },
}

impl State {
    const SIZE_BYTE: u8 = 0x20;
    const COMMAND_BYTE: u8 = 0x40;
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct IbusDecoder {
    state: State,
    buffer: [u8; Self::BUFFER_SIZE],
}

impl IbusDecoder {
    pub const CHANNEL_COUNT: usize = 14;
    const BUFFER_SIZE: usize = Self::CHANNEL_COUNT * 2;

    #[must_use]
    pub const fn new() -> Self {
        Self { state: State::WaitForSizeByte, buffer: [0; Self::BUFFER_SIZE] }
    }
}

impl IbusDecoder {
    /// Processes a single byte incoming from the UART interface.
    /// Returns `Some(&[u16; 14])` only when a valid frame passes checksum validation.
    pub fn on_byte_received(&mut self, byte: u8) -> Option<RxFrame> {
        let mut complete = false;

        self.state = match core::mem::take(&mut self.state) {
            State::WaitForSizeByte => {
                if byte == State::SIZE_BYTE {
                    State::WaitForCommandByte
                } else {
                    State::WaitForSizeByte
                }
            }
            State::WaitForCommandByte => {
                if byte == State::COMMAND_BYTE {
                    // The iBUS checksum is the one's complement of the sum of the first 30 bytes.
                    // Start with a value of 0xFFFF and subtract every byte from it.
                    // So we subtract the size and command bytes here.
                    let checksum = 0xFFFF - u16::from(State::SIZE_BYTE) - u16::from(State::COMMAND_BYTE);
                    State::ReadPayload { index: 0, checksum }
                } else {
                    State::WaitForSizeByte
                }
            }
            State::ReadPayload { index, mut checksum } => {
                checksum -= u16::from(byte);
                self.buffer[index] = byte;
                let index = index + 1;

                if index >= self.buffer.len() {
                    State::ValidateChecksum { checksum, is_first_byte: true, checksum_first_byte: 0 }
                } else {
                    State::ReadPayload { index, checksum }
                }
            }
            State::ValidateChecksum { checksum, is_first_byte, checksum_first_byte } => {
                if is_first_byte {
                    State::ValidateChecksum { checksum, is_first_byte: false, checksum_first_byte: byte }
                } else {
                    // Captured the second byte.
                    let checksum_received = u16::from_le_bytes([checksum_first_byte, byte]);
                    if checksum == checksum_received {
                        complete = true;
                    }
                    // Reset state to WaitForSizeByte for the next packet.
                    // This happens regardless of whether the checksum succeeds or fails.
                    State::WaitForSizeByte
                }
            }
        };
        if complete {
            const THROTTLE_CHANNEL: usize = 2;

            let mut channels = [0u16; RxFrame::MAX_CHANNEL_COUNT];
            // .as_chunks::<2>().0 gives a slice of [u8; 2] arrays
            for (ii, &chunk) in self.buffer.as_chunks::<2>().0.iter().enumerate() {
                channels[ii] = u16::from_le_bytes(chunk);
            }
            let link_status = if channels[THROTTLE_CHANNEL] < 950 { RxLinkStatus::Failsafe } else { RxLinkStatus::Ok };

            Some(RxFrame { channels, frame_type: RxFrameType::RcChannels, link_status, rssi: 0 })
        } else {
            None
        }
    }
}

#[cfg(test)]
mod test_traits {
    use super::*;

    fn is_full_eq<T: Sized + Send + Sync + Unpin + Copy + Clone + Default + PartialEq>() {}

    #[test]
    fn normal_types() {
        is_full_eq::<State>();
        is_full_eq::<IbusDecoder>();
    }
}

#[cfg(test)]
mod tests {
    use crate::{RxFrame, RxLinkStatus};

    use super::*;

    /// Helper function to build a valid IBUS frame array
    fn create_valid_frame(channel_values: [u16; IbusDecoder::CHANNEL_COUNT]) -> [u8; 32] {
        let mut frame = [0u8; 32];
        frame[0] = 0x20; // Length
        frame[1] = 0x40; // Command type

        // Inject the channel values as Little Endian bytes
        for i in 0..IbusDecoder::CHANNEL_COUNT {
            let offset = 2 + (i * 2);
            let bytes = channel_values[i].to_le_bytes();
            frame[offset] = bytes[0];
            frame[offset + 1] = bytes[1];
        }

        // Calculate Checksum: 0xFFFF - sum(bytes 0..30)
        let mut checksum: u16 = 0xFFFF;
        for i in 0..30 {
            checksum -= frame[i] as u16;
        }

        let cs_bytes = checksum.to_le_bytes();
        frame[30] = cs_bytes[0];
        frame[31] = cs_bytes[1];

        frame
    }

    #[test]
    fn test_successful_decode() {
        let mut decoder = IbusDecoder::new();

        // 1. Establish an arbitrary set of healthy receiver channel mappings
        let input_channels = [
            1500, 1500, 1100, 1500, // Roll, Pitch, Throttle, Yaw
            1000, 2000, 1500, 1000, // AUX 1-4
            1200, 1300, 1400, 1600, 1700, 1800, // Extra channels
        ];

        let raw_stream = create_valid_frame(input_channels);

        // 2. Feed the stream into the state machine byte-by-byte
        let mut result = None;
        for &byte in raw_stream.iter() {
            result = decoder.on_byte_received(byte);
        }

        // 3. Assert the state machine accurately matched the complete packet
        assert!(result.is_some(), "Decoder failed to yield channels on final frame byte!");
        let ibus_frame = result.unwrap();

        assert_eq!(
            ibus_frame.channels[..IbusDecoder::CHANNEL_COUNT],
            input_channels,
            "Decoded values do not match original inputs"
        );
        let rx_frame = RxFrame::from(ibus_frame);
        assert!(
            rx_frame.link_status == RxLinkStatus::Ok,
            "Decoder incorrectly flagged a healthy signal as a failsafe!"
        );
    }

    #[test]
    fn test_corrupted_checksum_rejection() {
        let mut decoder = IbusDecoder::new();
        let input_channels = [1500; IbusDecoder::CHANNEL_COUNT];
        let mut raw_stream = create_valid_frame(input_channels);

        // Corrupt a middle payload byte (byte index 5)
        raw_stream[5] ^= 0xFF;

        let mut result = None;
        for &byte in raw_stream.iter() {
            result = decoder.on_byte_received(byte);
        }

        // The state machine should finish processing but return None due to a failed checksum match
        assert!(result.is_none(), "Decoder accepted a packet with a corrupted payload byte!");
    }

    #[test]
    fn test_invalid_header_recovery() {
        let mut decoder = IbusDecoder::new();
        let input_channels = [1500; IbusDecoder::CHANNEL_COUNT];
        let valid_stream = create_valid_frame(input_channels);

        // Define our fixed noise sequence (6 bytes)
        let noise = [0x00, 0xFF, 0x20, 0x20, 0x41, 0x00];

        // Combine them into a single compile-time fixed-size buffer (6 + 32 = 38 bytes)
        let mut noisy_stream = [0u8; 6 + 32];

        // Copy the slices into our fixed stack memory
        noisy_stream[..6].copy_from_slice(&noise);
        noisy_stream[6..].copy_from_slice(&valid_stream);

        let mut decoded_frame = None;

        for &byte in noisy_stream.iter() {
            // By dereferencing or cloning the value inside the if-let,
            // we release the borrow on `decoder` immediately.
            if let Some(ibus_frame) = decoder.on_byte_received(byte) {
                decoded_frame = Some(ibus_frame);
            }
        }

        assert!(decoded_frame.is_some(), "Decoder failed to sync and recover after receiving noise!");
        assert_eq!(
            decoded_frame.unwrap().channels[..IbusDecoder::CHANNEL_COUNT],
            [1500; IbusDecoder::CHANNEL_COUNT],
            "Recovered packet contained bad channel data"
        );
    }

    #[test]
    fn test_receiver_failsafe_detection() {
        let mut decoder = IbusDecoder::new();

        // Emulate typical radio-link failure values where throttle (Channel 3) drops below 950
        let mut failsafe_channels = [1500; IbusDecoder::CHANNEL_COUNT];
        failsafe_channels[2] = 900; // Drop channel 3 below 950 boundary

        let raw_stream = create_valid_frame(failsafe_channels);

        let mut result = None;
        for &byte in raw_stream.iter() {
            result = decoder.on_byte_received(byte);
        }

        assert!(result.is_some());
        let ibus_frame = result.unwrap();
        let rx_frame = RxFrame::from(ibus_frame);
        assert!(
            rx_frame.link_status == RxLinkStatus::Failsafe,
            "Decoder failed to identify internal receiver link failure!"
        );
    }
}
