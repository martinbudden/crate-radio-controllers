/// Packet Structure (32 Bytes)
/// Each packet contains 14 channels, each represented by a 16-bit value (2 bytes) in Little-Endian format.
///    Byte 0: Header 0x20 (Size)
///    Byte 1: Command/Type 0x40 (for RC channels)
///    Bytes 2–29: 14 Channels (2 bytes each, Little-Endian)
///    Bytes 30–31: Checksum (2 bytes, Little-Endian).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum IBusDecoderState {
    #[default]
    WaitForSizeByte,
    WaitForCommandByte,
    ReadPayload {
        buffer: [u8; Self::BUFFER_SIZE],
        idx: usize,
        checksum: u16,
    },
    ValidateChecksum {
        buffer: [u8; Self::BUFFER_SIZE],
        checksum: u16,
        is_high_byte: bool, // false = waiting for low byte, true = waiting for high byte
        checksum_low: u8,
    },
}

impl IBusDecoderState {
    const SIZE_BYTE: u8 = 0x20;
    const COMMAND_BYTE: u8 = 0x40;
    const BUFFER_SIZE: usize = IBusDecoder::CHANNEL_COUNT * 2;
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct IBusDecoder {
    state: IBusDecoderState,
    channels: [u16; Self::CHANNEL_COUNT],
}

impl IBusDecoder {
    pub const CHANNEL_COUNT: usize = 14;
    pub const THROTTLE_CHANNEL: usize = 2;

    #[must_use]
    pub const fn new() -> Self {
        Self {
            state: IBusDecoderState::WaitForSizeByte,
            channels: [1000; Self::CHANNEL_COUNT], // Default fallback value
        }
    }
}

impl IBusDecoder {
    /// Processes a single byte incoming from the UART interface.
    /// Returns `Some(&[u16; 14])` only when a valid frame passes checksum validation.
    pub fn on_byte_received(&mut self, byte: u8) -> Option<&[u16; Self::CHANNEL_COUNT]> {
        let current_state = core::mem::replace(&mut self.state, IBusDecoderState::WaitForSizeByte);

        match current_state {
            IBusDecoderState::WaitForSizeByte => {
                if byte == IBusDecoderState::SIZE_BYTE {
                    self.state = IBusDecoderState::WaitForCommandByte;
                } else {
                    self.state = IBusDecoderState::WaitForSizeByte;
                }
            }
            IBusDecoderState::WaitForCommandByte => {
                if byte == IBusDecoderState::COMMAND_BYTE {
                    // Start computing the checksum: 0xFFFF - 0x20 - 0x40 = 0xFF9F
                    let checksum =
                        0xFFFF - u16::from(IBusDecoderState::SIZE_BYTE) - u16::from(IBusDecoderState::COMMAND_BYTE);
                    self.state =
                        IBusDecoderState::ReadPayload { buffer: [0; IBusDecoderState::BUFFER_SIZE], idx: 0, checksum };
                } else {
                    self.state = IBusDecoderState::WaitForSizeByte;
                }
            }
            IBusDecoderState::ReadPayload { mut buffer, idx, mut checksum } => {
                checksum -= u16::from(byte);
                buffer[idx] = byte;
                let next_idx = idx + 1;

                if next_idx >= buffer.len() {
                    self.state =
                        IBusDecoderState::ValidateChecksum { buffer, checksum, is_high_byte: false, checksum_low: 0 };
                } else {
                    self.state = IBusDecoderState::ReadPayload { buffer, idx: next_idx, checksum };
                }
            }
            IBusDecoderState::ValidateChecksum { buffer, checksum, is_high_byte, checksum_low } => {
                if is_high_byte {
                    // Captured the final high byte.
                    let checksum_received = u16::from_le_bytes([checksum_low, byte]);

                    // Reset state to WaitLength for the next packet.
                    // This happens regardless of whether the checksum succeeds or fails.
                    self.state = IBusDecoderState::WaitForSizeByte;

                    if checksum == checksum_received {
                        self.parse_channels_from_buffer(&buffer);
                        return Some(&self.channels);
                    }
                } else {
                    // Captured the low byte, now shift to wait for the high byte
                    self.state =
                        IBusDecoderState::ValidateChecksum { buffer, checksum, is_high_byte: true, checksum_low: byte };
                }
            }
        }
        None
    }

    /// Internal helper to parse the raw 28 bytes into 14 channels.
    fn parse_channels_from_buffer(&mut self, buffer: &[u8; IBusDecoderState::BUFFER_SIZE]) {
        // .as_chunks::<2>().0 gives a slice of [u8; 2] arrays
        for (ii, &chunk) in buffer.as_chunks::<2>().0.iter().enumerate() {
            self.channels[ii] = u16::from_le_bytes(chunk);
        }
    }

    /// Failsafe check (checks if throttle channel drops below 950).
    #[must_use]
    pub fn is_receiver_failsafe(&self) -> bool {
        self.channels[Self::THROTTLE_CHANNEL] < 950
    }
}

#[cfg(test)]
mod test_traits {
    use super::*;

    fn is_full_eq<T: Sized + Send + Sync + Unpin + Copy + Clone + Default + PartialEq>() {}

    #[test]
    fn normal_types() {
        is_full_eq::<IBusDecoderState>();
        is_full_eq::<IBusDecoder>();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper function to build a valid IBUS frame array
    fn create_valid_frame(channel_values: [u16; IBusDecoder::CHANNEL_COUNT]) -> [u8; 32] {
        let mut frame = [0u8; 32];
        frame[0] = 0x20; // Length
        frame[1] = 0x40; // Command type

        // Inject the channel values as Little Endian bytes
        for i in 0..IBusDecoder::CHANNEL_COUNT {
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
        let mut decoder = IBusDecoder::new();

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
        let output_channels = result.unwrap();

        assert_eq!(output_channels, &input_channels, "Decoded values do not match original inputs");
        assert!(!decoder.is_receiver_failsafe(), "Decoder incorrectly flagged a healthy signal as a failsafe!");
    }

    #[test]
    fn test_corrupted_checksum_rejection() {
        let mut decoder = IBusDecoder::new();
        let input_channels = [1500; IBusDecoder::CHANNEL_COUNT];
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
        let mut decoder = IBusDecoder::new();
        let input_channels = [1500; IBusDecoder::CHANNEL_COUNT];
        let valid_stream = create_valid_frame(input_channels);

        // Define our fixed noise sequence (6 bytes)
        let noise = [0x00, 0xFF, 0x20, 0x20, 0x41, 0x00];

        // Combine them into a single compile-time fixed-size buffer (6 + 32 = 38 bytes)
        let mut noisy_stream = [0u8; 6 + 32];

        // Copy the slices into our fixed stack memory
        noisy_stream[..6].copy_from_slice(&noise);
        noisy_stream[6..].copy_from_slice(&valid_stream);

        let mut decoded_channels = None;

        for &byte in noisy_stream.iter() {
            // By dereferencing or cloning the value inside the if-let,
            // we release the borrow on `decoder` immediately.
            if let Some(&channels) = decoder.on_byte_received(byte) {
                decoded_channels = Some(channels);
            }
        }

        assert!(decoded_channels.is_some(), "Decoder failed to sync and recover after receiving noise!");
        assert_eq!(
            decoded_channels.unwrap(),
            [1500; IBusDecoder::CHANNEL_COUNT],
            "Recovered packet contained bad channel data"
        );
    }

    #[test]
    fn test_receiver_failsafe_detection() {
        let mut decoder = IBusDecoder::new();

        // Emulate typical radio-link failure values where throttle (Channel 3) drops below 950
        let mut failsafe_channels = [1500; IBusDecoder::CHANNEL_COUNT];
        failsafe_channels[2] = 900; // Drop channel 3 below 950 boundary

        let raw_stream = create_valid_frame(failsafe_channels);

        let mut result = None;
        for &byte in raw_stream.iter() {
            result = decoder.on_byte_received(byte);
        }

        assert!(result.is_some());
        assert!(decoder.is_receiver_failsafe(), "Decoder failed to identify internal receiver link failure!");
    }
}
