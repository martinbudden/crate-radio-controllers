use crate::{RxChannels, RxLinkStatus};

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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct IbusDecoder {
    state: State,
    buffer: [u8; Self::PAYLOAD_LENGTH],
}

impl Default for IbusDecoder {
    fn default() -> Self {
        Self::new()
    }
}

impl IbusDecoder {
    #[allow(unused)]
    pub const CHANNEL_COUNT: usize = 14;
    pub const PACKET_LENGTH: usize = 32;
    const PAYLOAD_LENGTH: usize = 28;

    #[must_use]
    pub const fn new() -> Self {
        Self { state: State::WaitForSizeByte, buffer: [0u8; Self::PAYLOAD_LENGTH] }
    }
}

impl IbusDecoder {
    /// Processes a single byte incoming from the UART interface.
    /// Returns `Some(&[u16; 14])` only when a valid frame passes checksum validation.
    pub fn on_byte_received(&mut self, byte: u8) -> Option<(RxChannels, RxLinkStatus)> {
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

            let mut channels = RxChannels::default();
            // .as_chunks::<2>().0 gives a slice of [u8; 2] arrays
            for (ii, &chunk) in self.buffer.as_chunks::<2>().0.iter().enumerate() {
                channels[ii] = u16::from_le_bytes(chunk);
            }
            let link_status = if channels[THROTTLE_CHANNEL] < 950 { RxLinkStatus::Failsafe } else { RxLinkStatus::Ok };

            Some((channels, link_status))
        } else {
            None
        }
    }

    #[allow(unused)]
    pub fn parse_packet(&mut self, buffer: &[u8; Self::PACKET_LENGTH]) -> Option<(RxChannels, RxLinkStatus)> {
        for byte in buffer {
            if let Some(result) = self.on_byte_received(*byte) {
                return Some(result);
            }
        }
        None
    }

    /// Helper function to build an IBUS frame.
    #[must_use]
    pub fn create_ibus_frame(channels: [u16; IbusDecoder::CHANNEL_COUNT]) -> [u8; Self::PACKET_LENGTH] {
        let mut frame = [0u8; 32];
        frame[0] = State::SIZE_BYTE;
        frame[1] = State::COMMAND_BYTE;

        /*// Inject the channel values as Little Endian bytes
        for (ii, channel) in channels.iter().enumerate().take(IbusDecoder::CHANNEL_COUNT) {
            let bytes = channel.to_le_bytes();
            let offset = 2 + (ii * 2);
            frame[offset] = bytes[0];
            frame[offset + 1] = bytes[1];
        }*/
        // Split the frame into compile-time fixed [u8; 2] chunks
        let (frame_chunks, _remainder) = frame[2..].as_chunks_mut::<2>();

        // Zip and assign directly.
        for (dst_channel, channel) in frame_chunks.iter_mut().zip(channels.iter().take(IbusDecoder::CHANNEL_COUNT)) {
            *dst_channel = channel.to_le_bytes();
        }

        // Calculate Checksum: 0xFFFF - sum(bytes 0..30)
        let mut checksum: u16 = 0xFFFF;
        for frame in frame.iter().take(30) {
            checksum -= u16::from(*frame);
        }

        frame[30..32].copy_from_slice(&checksum.to_le_bytes());

        frame
    }
}

#[cfg(test)]
mod test_traits {
    use super::*;

    fn is_full<T: Sized + Send + Sync + Unpin + Copy + Clone + Default + PartialEq>() {}

    #[test]
    fn normal_types() {
        is_full::<State>();
        is_full::<IbusDecoder>();
    }
}

#[cfg(test)]
mod tests {
    use crate::RxLinkStatus;

    use super::*;

    #[test]
    fn example() {
        let expected_channels = RxChannels::from_channels([
            1500, 1500, 1100, 1500, // Roll, Pitch, Throttle, Yaw
            1000, 2000, 1500, 1000, // AUX 1-4
            1200, 1300, 1400, 1600, // AUX 5-8
            1700, 1800, 1000, 1000, // AUX 9-12
        ]);

        // simulate byte stream from serial port using an array.
        let byte_stream = [
            0x20, 0x40, 0xDC, 0x05, 0xDC, 0x05, 0x4C, 0x04, 0xDC, 0x05, 0xE8, 0x03, 0xD0, 0x07, 0xDC, 0x05, 0xE8, 0x03,
            0xB0, 0x04, 0x14, 0x05, 0x78, 0x05, 0x40, 0x06, 0xA4, 0x06, 0x08, 0x07, 0xD5, 0xF6,
        ];

        // decode the byte stream
        let mut decoder = IbusDecoder::new();
        let mut result = None;
        for &byte in byte_stream.iter() {
            result = decoder.on_byte_received(byte);
        }

        // deconstruct result and check it is correct
        if let Some((channels, link_status)) = result {
            assert_eq!(channels, expected_channels);
            assert_eq!(link_status, RxLinkStatus::Ok);
        } else {
            panic!("decode failed");
        }
    }

    #[test]
    fn test_successful_decode() {
        let mut decoder = IbusDecoder::new();

        let input_channels = [
            1500, 1500, 1100, 1500, // Roll, Pitch, Throttle, Yaw
            1000, 2000, 1500, 1000, // AUX 1-4
            1200, 1300, 1400, 1600, 1700, 1800, // Extra channels
        ];

        let byte_stream = IbusDecoder::create_ibus_frame(input_channels);
        assert_eq!(
            byte_stream,
            [
                0x20, 0x40, 0xDC, 0x05, 0xDC, 0x05, 0x4C, 0x04, 0xDC, 0x05, 0xE8, 0x03, 0xD0, 0x07, 0xDC, 0x05, 0xE8,
                0x03, 0xB0, 0x04, 0x14, 0x05, 0x78, 0x05, 0x40, 0x06, 0xA4, 0x06, 0x08, 0x07, 0xD5, 0xF6
            ]
        );

        // Feed the stream into the state machine byte-by-byte
        let mut result = None;
        for &byte in byte_stream.iter() {
            result = decoder.on_byte_received(byte);
        }

        // Assert the state machine accurately matched the complete packet
        assert!(result.is_some(), "Decoder failed to yield channels on final frame byte!");
        let (channels, link_status) = result.unwrap();
        assert_eq!(
            channels.channels()[..IbusDecoder::CHANNEL_COUNT],
            input_channels,
            "Decoded values do not match original inputs"
        );
        assert!(link_status == RxLinkStatus::Ok, "Decoder incorrectly flagged a healthy signal as a failsafe!");
    }

    #[test]
    fn test_corrupted_checksum_rejection() {
        let mut decoder = IbusDecoder::new();
        let input_channels = [1500; IbusDecoder::CHANNEL_COUNT];
        let mut byte_stream = IbusDecoder::create_ibus_frame(input_channels);

        // Corrupt a middle payload byte (byte index 5)
        byte_stream[5] ^= 0xFF;

        let mut result = None;
        for &byte in byte_stream.iter() {
            result = decoder.on_byte_received(byte);
        }

        // The state machine should finish processing but return None due to a failed checksum match
        assert!(result.is_none(), "Decoder accepted a packet with a corrupted payload byte!");
    }

    #[test]
    fn test_invalid_header_recovery() {
        let mut decoder = IbusDecoder::new();
        let input_channels = [1500; IbusDecoder::CHANNEL_COUNT];
        let valid_stream = IbusDecoder::create_ibus_frame(input_channels);

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
        let (channels, link_status) = decoded_frame.unwrap();
        assert_eq!(
            channels.channels()[..IbusDecoder::CHANNEL_COUNT],
            [1500; IbusDecoder::CHANNEL_COUNT],
            "Recovered packet contained bad channel data"
        );
        assert_eq!(link_status, RxLinkStatus::Ok)
    }

    #[test]
    fn test_receiver_failsafe_detection() {
        let mut decoder = IbusDecoder::new();

        // Emulate typical radio-link failure values where throttle (Channel 3) drops below 950
        let mut failsafe_channels = [1500; IbusDecoder::CHANNEL_COUNT];
        failsafe_channels[2] = 900; // Drop channel 3 below 950 boundary

        let byte_stream = IbusDecoder::create_ibus_frame(failsafe_channels);

        let mut result = None;
        for &byte in byte_stream.iter() {
            result = decoder.on_byte_received(byte);
        }

        assert!(result.is_some());
        let (_channels, link_status) = result.unwrap();
        assert!(link_status == RxLinkStatus::Failsafe, "Decoder failed to identify internal receiver link failure!");
    }
}
