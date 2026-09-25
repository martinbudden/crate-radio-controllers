use super::SbusFrame;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
enum State {
    #[default]
    WaitingForHeader,
    CollectingPayload {
        index: usize,
    },
    ValidatingFooter,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SbusDecoder {
    state: State,
    buffer: [u8; Self::PACKET_LENGTH],
}

impl Default for SbusDecoder {
    fn default() -> Self {
        Self::new()
    }
}

impl SbusDecoder {
    pub const PACKET_LENGTH: usize = 25;
    pub const HEADER_LENGTH: usize = 1;
    pub const PAYLOAD_LENGTH: usize = 22;

    pub const fn new() -> Self {
        Self { state: State::WaitingForHeader, buffer: [0u8; Self::PACKET_LENGTH] }
    }
}

/// An SBUS frame is 25 bytes total:
/// Byte 0: Header (0x0F).
/// Bytes 1-22: The channel data (used in the function above)
/// Byte 23: Flags Byte (Contains digital channels 17/18 and the Failsafe/Frame Lost bits)
/// Byte 24: Footer (0x00).
///
/// The Flags Byte Structure:
/// Bit 0: Digital Channel 17 (Aux13) (0 = Off, 1 = On)
/// Bit 1: Digital Channel 18 (Aux14) (0 = Off, 1 = On)
/// Bit 2: Frame Lost (Signal was missed this frame)
/// Bit 3: Failsafe (Receiver has completely lost connection)
/// Bits 4-7: Reserved.
///
/// SBUS is inverted. For microcontrollers that don't support "Inverted UART",
/// a hardware inverter (a simple NPN transistor or a NOT gate) is required.
///
impl SbusDecoder {
    /// This is the core logic. It takes one byte and returns a Some(Frame) when a full, valid packet is completed.
    pub fn on_byte_received(&mut self, byte: u8) -> Option<SbusFrame> {
        let mut complete = false;

        self.state = match core::mem::take(&mut self.state) {
            State::WaitingForHeader => {
                if byte == 0x0F {
                    // We have a valid header byte, so start collecting the payload.
                    self.buffer[0] = byte;
                    State::CollectingPayload { index: 1 }
                } else {
                    State::WaitingForHeader
                }
            }
            // Collect the 22 bytes of payload.
            State::CollectingPayload { index } => {
                self.buffer[index] = byte;
                let index = index + 1;

                // When we have collected the payload, move onto the footer.
                if index > Self::HEADER_LENGTH + Self::PAYLOAD_LENGTH {
                    State::ValidatingFooter
                } else {
                    State::CollectingPayload { index }
                }
            }
            State::ValidatingFooter => {
                if byte == 0x00 {
                    complete = true;
                }
                State::WaitingForHeader
            }
        };

        if complete {
            let data: [u8; Self::PAYLOAD_LENGTH] = self.buffer[1..23].try_into().unwrap_or([0u8; Self::PAYLOAD_LENGTH]);
            let channels = Self::parse_sbus_channels(&data);
            let sbus_frame = SbusFrame { channels, flags: 0, rssi: 0 };
            Some(sbus_frame)
        } else {
            None
        }
    }

    #[allow(unused)]
    pub fn parse(&mut self, buffer: &[u8; Self::PACKET_LENGTH]) -> Option<SbusFrame> {
        for byte in buffer {
            if let Some(frame) = self.on_byte_received(*byte) {
                return Some(frame);
            }
        }
        None
    }

    /// Extracts 16 channels from a 22-byte SBUS payload.
    /// SBUS uses 11 bits per channel, Little-Endian bit-packing.
    /// Because 11 bits don't divide evenly into 8-bit bytes, the pattern repeats every 8 channels (every 11 bytes).
    ///
    /// Bitmasking: Every line ends with & 0x07FF. This ensures that even if bits "bleed" over from the next byte, only the 11 bits we care about are kept.
    /// Performance: On a typical 32-bit MCU , the compiler will optimize these into simple LDR, LSR/LSL, and AND instructions.
    pub fn parse_sbus_channels(p: &[u8; 22]) -> [u16; 16] {
        let mut channels = [0u16; 16];

        // .as_chunks::<11>().0 returns a slice of [u8; 11] arrays
        let chunks = p.as_chunks::<11>().0;

        // Process the first 11 bytes into channels 0..8
        Self::parse_8_channels(&chunks[0], &mut channels[0..8]);

        // Process the next 11 bytes into channels 8..16
        Self::parse_8_channels(&chunks[1], &mut channels[8..16]);

        channels
    }

    #[inline]
    fn parse_8_channels(p: &[u8; 11], out: &mut [u16]) {
        // Slurp bytes in 32-bit windows using zero-overhead from_le_bytes
        // This allows the CPU to use 32-bit hardware registers and barrel shifters

        // Window 1: Bytes 0, 1, 2, 3 (Contains Ch 0, 1, and parts of 2)
        let w0 = u32::from_le_bytes([p[0], p[1], p[2], p[3]]);
        out[0] = (w0 & 0x7FF) as u16;
        out[1] = ((w0 >> 11) & 0x7FF) as u16;

        // Window 2: Bytes 2, 3, 4, 5 (Contains Ch 2, 3, and parts of 4)
        let w1 = u32::from_le_bytes([p[2], p[3], p[4], p[5]]);
        out[2] = ((w1 >> 6) & 0x7FF) as u16;
        out[3] = ((w1 >> 17) & 0x7FF) as u16;

        // Window 3: Bytes 5, 6, 7, 8 (Contains Ch 4, 5, and parts of 6)
        let w2 = u32::from_le_bytes([p[5], p[6], p[7], p[8]]);
        out[4] = ((w2 >> 4) & 0x7FF) as u16;
        out[5] = ((w2 >> 15) & 0x7FF) as u16;

        // Window 4: Bytes 8, 9, 10, plus a trailing 0 padding byte
        let w3 = u32::from_le_bytes([p[8], p[9], p[10], 0]);
        out[6] = ((w3 >> 2) & 0x7FF) as u16;
        out[7] = ((w3 >> 13) & 0x7FF) as u16;
    }
    /*pub fn parse_sbus_channels(p: &[u8; 22]) -> [u16; 16] {
        [
            ((u16::from(p[0]) | (u16::from(p[1])) << 8) & 0x07FF),
            ((u16::from(p[1]) >> 3 | (u16::from(p[2])) << 5) & 0x07FF),
            ((u16::from(p[2]) >> 6 | (u16::from(p[3])) << 2 | (u16::from(p[4])) << 10) & 0x07FF),
            ((u16::from(p[4]) >> 1 | (u16::from(p[5])) << 7) & 0x07FF),
            ((u16::from(p[5]) >> 4 | (u16::from(p[6])) << 4) & 0x07FF),
            ((u16::from(p[6]) >> 7 | (u16::from(p[7])) << 1 | (u16::from(p[8])) << 9) & 0x07FF),
            ((u16::from(p[8]) >> 2 | (u16::from(p[9])) << 6) & 0x07FF),
            ((u16::from(p[9]) >> 5 | (u16::from(p[10])) << 3) & 0x07FF),
            // the pattern repeats because we've exactly consumed 11 bytes
            ((u16::from(p[11]) | (u16::from(p[12])) << 8) & 0x07FF),
            ((u16::from(p[12]) >> 3 | (u16::from(p[13])) << 5) & 0x07FF),
            ((u16::from(p[13]) >> 6 | (u16::from(p[14])) << 2 | (u16::from(p[15])) << 10) & 0x07FF),
            ((u16::from(p[15]) >> 1 | (u16::from(p[16])) << 7) & 0x07FF),
            ((u16::from(p[16]) >> 4 | (u16::from(p[17])) << 4) & 0x07FF),
            ((u16::from(p[17]) >> 7 | (u16::from(p[18])) << 1 | (u16::from(p[19])) << 9) & 0x07FF),
            ((u16::from(p[19]) >> 2 | (u16::from(p[20])) << 6) & 0x07FF),
            ((u16::from(p[20]) >> 5 | (u16::from(p[21])) << 3) & 0x07FF),
        ]
    }*/
}

#[cfg(test)]
mod test_traits {
    use super::*;

    fn is_full<T: Sized + Send + Sync + Unpin + Copy + Clone + Default + PartialEq>() {}

    #[test]
    fn normal_types() {
        is_full::<SbusDecoder>();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new() {
        let frame = SbusFrame::default();
        assert_eq!(0, frame.rssi);
    }
    #[test]
    fn parse_message() {
        #[rustfmt::skip]
        let stream: [u8; SbusDecoder::PACKET_LENGTH] = [
            0x0F, // header
            // 22 u8s
            0xE0, 0x03, 0x1F, 0x58, 0xC0, 0x07, 0x16, 0xB0, 0x80, 0x05, 0x2C, 
            0x60, 0x01, 0x0B, 0xF8, 0xC0, 0x07, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x03, // flags
            0x00, // footer
        ];

        let expected_channels: [u16; SbusFrame::CHANNEL_COUNT] =
            [992, 992, 352, 992, 352, 352, 352, 352, 352, 352, 992, 992, 0, 0, 0, 0];

        let mut sbus_parser = SbusDecoder::new();
        if let Some(frame) = sbus_parser.parse(&stream) {
            let channels = frame.channels;
            assert_eq!(expected_channels, channels);
        } else {
            unreachable!();
        }
    }
}
#[cfg(test)]
mod sbus_tests {
    // Bring the `parse_sbus_channels` visibility into the local test scope
    use super::*;

    /// Safe helper function to pack a raw channel into 11-bit chunks manually.
    /// This gives us a 100% accurate, known input frame buffer for the decoder test.
    fn generate_mock_sbus_buffer(input_channels: [u16; 16]) -> [u8; 22] {
        let mut buffer = [0u8; 22];
        let mut bit_bucket: u32 = 0;
        let mut bits_in_bucket: u32 = 0;
        let mut byte_index: usize = 0;

        for channel_val in input_channels.iter() {
            // Mask to ensure the value stays strictly constrained inside 11 bits (0..2047)
            let clean_val = channel_val & 0x07FF;

            // Push the 11 bits onto our active buffer accumulator
            bit_bucket |= (clean_val as u32) << bits_in_bucket;
            bits_in_bucket += 11;

            // Drain whole bytes out of our bit accumulator into the final buffer array
            while bits_in_bucket >= 8 {
                buffer[byte_index] = (bit_bucket & 0xFF) as u8;
                byte_index += 1;
                bit_bucket >>= 8;
                bits_in_bucket -= 8;
            }
        }

        // Flush any remaining trailing bits into the final payload slots
        if bits_in_bucket > 0 && byte_index < 22 {
            buffer[byte_index] = (bit_bucket & 0xFF) as u8;
        }

        buffer
    }

    #[test]
    fn test_sbus_32bit_window_decoder() {
        // Establish an arbitrary, diverse set of test channels (values 0..2047)
        let expected_channels = [1000, 1500, 172, 2000, 111, 1890, 512, 1024, 1500, 992, 1234, 45, 2047, 0, 777, 1520];

        // 1. Pack our known good values into a real 22-byte packed stream array
        let packed_sbus_stream = generate_mock_sbus_buffer(expected_channels);

        // 2. Pass the byte payload through your optimized chunks and 32-bit window algorithm
        // Note: Replace `MyStruct::` with the correct namespace/struct where your method lives
        let decoded_output = SbusDecoder::parse_sbus_channels(&packed_sbus_stream);

        // 3. Assert the output matches exactly with zero data degradation
        assert_eq!(
            decoded_output, expected_channels,
            "The optimized 32-bit overlapping window bit shift algorithm misaligned channel data!"
        );
    }

    #[test]
    fn test_sbus_decoder_extremes() {
        // Test edge cases: Every single bit turned entirely on (2047) or off (0)
        let max_channels = [2047u16; 16];
        let min_channels = [0u16; 16];

        let max_buffer = generate_mock_sbus_buffer(max_channels);
        let min_buffer = generate_mock_sbus_buffer(min_channels);

        assert_eq!(
            SbusDecoder::parse_sbus_channels(&max_buffer),
            max_channels,
            "Failed to decode max 11-bit boundaries"
        );
        assert_eq!(
            SbusDecoder::parse_sbus_channels(&min_buffer),
            min_channels,
            "Failed to decode min 11-bit boundaries"
        );
    }
}
