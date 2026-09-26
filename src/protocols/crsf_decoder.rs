#![allow(unused)]

use super::CrcDvbS2;

/// Maximum channels provided by CRSF.
pub const CRSF_MAX_CHANNELS: usize = 16;
/// Length of the raw bit-packed channels block (16 channels * 11 bits = 176 bits = 22 bytes).
pub const CRSF_PAYLOAD_LEN: usize = 22;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CrsfFrame {
    pub channels: [u16; CRSF_MAX_CHANNELS],
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum State {
    #[default]
    WaitForAddressByte,
    WaitLength,
    ReadPayload {
        index: usize,
        expected_len: usize,
    },
    ValidateChecksum,
}

impl State {
    const ADDRESS_BYTE: u8 = 0xC8;
    // Stores Type (1 byte) + Channel Payload (22 bytes)
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CrsfDecoder {
    state: State,
    frame: CrsfFrame,
    buffer: [u8; Self::BUFFER_SIZE],
}

impl CrsfDecoder {
    pub const CHANNEL_COUNT: usize = 16;
    const BUFFER_SIZE: usize = CRSF_PAYLOAD_LEN + 1;
    pub const fn new() -> Self {
        Self {
            state: State::WaitForAddressByte,
            frame: CrsfFrame { channels: [1000; Self::CHANNEL_COUNT] },
            buffer: [0u8; Self::BUFFER_SIZE],
        }
    }

    /// Feeds a single byte into the CRSF state machine.
    /// Returns `Some(&CrsfFrame)` on a successful CRC8 match.
    pub fn on_byte_received(&mut self, byte: u8) -> Option<&CrsfFrame> {
        let mut should_parse = false;

        self.state = match core::mem::take(&mut self.state) {
            State::WaitForAddressByte => {
                if byte == State::ADDRESS_BYTE {
                    State::WaitLength
                } else {
                    State::WaitForAddressByte
                }
            }
            State::WaitLength => {
                if byte >= 3 && (byte as usize) <= (CRSF_PAYLOAD_LEN + 2) {
                    State::ReadPayload { index: 0, expected_len: byte as usize }
                } else {
                    State::WaitForAddressByte
                }
            }
            State::ReadPayload { index, expected_len } => {
                self.buffer[index] = byte;
                let next_index = index + 1;

                // FIX: Let index fill all payload bytes completely (Type + Payload = expected_len - 1).
                // For a standard frame of length 24, this will fill buffer indices 0 to 22.
                if next_index >= (expected_len - 1) {
                    State::ValidateChecksum
                } else {
                    State::ReadPayload { index: next_index, expected_len }
                }
            }
            State::ValidateChecksum => {
                // This byte is now accurately the 24th byte (the raw CRC byte)
                let received_crc = byte;

                // CRSF payload for checksum is always (Length - 1).
                // Since this state is constant, we can slice the exact written payload length.
                let payload = &self.buffer[..Self::BUFFER_SIZE];

                if CrcDvbS2::calculate(payload) == received_crc && self.buffer[0] == 0x16 {
                    should_parse = true;
                }
                State::WaitForAddressByte
            }
        };

        if should_parse {
            // Safe compile-time slice extraction
            if let Ok(channel_data) = self.buffer[1..23].try_into() {
                Self::parse_channels(&mut self.frame.channels, channel_data);
                return Some(&self.frame);
            }
        }

        None
    }

    /// Fast 32-bit overlapping window channel extraction (leveraging your optimized SBUS pipeline).
    fn parse_channels(channels: &mut [u16; Self::CHANNEL_COUNT], payload: &[u8; CRSF_PAYLOAD_LEN]) {
        let chunks = payload.as_chunks::<11>().0;

        Self::parse_8_channels(&chunks[0], &mut channels[0..8]);
        Self::parse_8_channels(&chunks[1], &mut channels[8..16]);
    }

    #[inline]
    fn parse_8_channels(p: &[u8; 11], out: &mut [u16]) {
        let w0 = u32::from_le_bytes([p[0], p[1], p[2], p[3]]);
        out[0] = (w0 & 0x7FF) as u16;
        out[1] = ((w0 >> 11) & 0x7FF) as u16;

        let w1 = u32::from_le_bytes([p[2], p[3], p[4], p[5]]);
        out[2] = ((w1 >> 6) & 0x7FF) as u16;
        out[3] = ((w1 >> 17) & 0x7FF) as u16;

        let w2 = u32::from_le_bytes([p[5], p[6], p[7], p[8]]);
        out[4] = ((w2 >> 4) & 0x7FF) as u16;
        out[5] = ((w2 >> 15) & 0x7FF) as u16;

        let w3 = u32::from_le_bytes([p[8], p[9], p[10], 0]);
        out[6] = ((w3 >> 2) & 0x7FF) as u16;
        out[7] = ((w3 >> 13) & 0x7FF) as u16;
    }
}

#[cfg(test)]
mod test_traits {
    use super::*;

    fn is_full<T: Sized + Send + Sync + Unpin + Copy + Clone + Default + PartialEq>() {}

    #[test]
    fn normal_types() {
        is_full::<CrsfDecoder>();
    }
}

#[cfg(test)]
mod crsf_tests {
    use super::*; // Assumes CrsfDecoder, CrsfFrame, and CRSF_PAYLOAD_LEN are in scope

    /// Standard CRSF CRC8 implementation (DVB-S2 variant, polynomial 0xD5)
    /// Used here to cleanly generate valid testing packets.
    fn calculate_test_crc8(data: &[u8]) -> u8 {
        let mut crc = 0u8;
        for &byte in data {
            crc ^= byte;
            for _ in 0..8 {
                if (crc & 0x80) != 0 {
                    crc = (crc << 1) ^ 0xD5;
                } else {
                    crc <<= 1;
                }
            }
        }
        crc
    }

    /// Helper function to build a completely valid, binary-accurate CRSF packet.
    /// Packs 16 channels into 11-bit chunks, appends type headers, and stamps the CRC8 byte.
    fn create_valid_crsf_packet(input_channels: [u16; 16]) -> [u8; 26] {
        let mut packet = [0u8; 26];
        packet[0] = 0xC8; // Address byte
        packet[1] = 24; // Length (1 byte Type + 22 bytes Payload + 1 byte CRC)
        packet[2] = 0x16; // Type: RC Channels Packed Frame

        // Pack 16 channels (11 bits each) into the 22-byte payload allocation block
        let mut bit_bucket: u32 = 0;
        let mut bits_in_bucket: u32 = 0;
        let mut byte_offset = 3;

        for &channel_val in input_channels.iter() {
            let clean_val = channel_val & 0x07FF; // Enforce 11-bit boundary constraint
            bit_bucket |= (clean_val as u32) << bits_in_bucket;
            bits_in_bucket += 11;

            while bits_in_bucket >= 8 {
                packet[byte_offset] = (bit_bucket & 0xFF) as u8;
                byte_offset += 1;
                bit_bucket >>= 8;
                bits_in_bucket -= 8;
            }
        }
        if bits_in_bucket > 0 && byte_offset < 25 {
            packet[byte_offset] = (bit_bucket & 0xFF) as u8;
        }

        // Calculate and append the final CRC8 byte over indices 2..25 (Type + Channel Payload)
        let crc = calculate_test_crc8(&packet[2..25]);
        packet[25] = crc;

        packet
    }

    #[test]
    fn test_successful_crsf_decode() {
        let mut decoder = CrsfDecoder::new();

        // 1. Establish an arbitrary set of healthy channel mappings (within 0..2047 limits)
        let input_channels = [1000, 1500, 172, 2000, 111, 1890, 512, 1024, 1500, 992, 1234, 45, 2047, 0, 777, 1520];

        let raw_stream = create_valid_crsf_packet(input_channels);

        // 2. Feed the stream into the state machine byte-by-byte
        let mut result = None;
        for &byte in raw_stream.iter() {
            if let Some(frame) = decoder.on_byte_received(byte) {
                result = Some(*frame); // Copy the reference data out safely for evaluation
            }
        }

        // 3. Assert the state machine accurately matched the complete packet
        assert!(result.is_some(), "Decoder failed to yield channels on final frame byte!");
        let output_frame = result.unwrap();

        assert_eq!(
            output_frame.channels, input_channels,
            "Decoded values do not match original 11-bit input boundaries!"
        );
    }

    #[test]
    fn test_corrupted_crc_rejection() {
        let mut decoder = CrsfDecoder::new();
        let input_channels = [1000; 16];
        let mut raw_stream = create_valid_crsf_packet(input_channels);

        // Corrupt the final CRC byte intentionally
        let last_idx = raw_stream.len() - 1;
        raw_stream[last_idx] ^= 0x5A;

        let mut result = None;
        for &byte in raw_stream.iter() {
            if let Some(frame) = decoder.on_byte_received(byte) {
                result = Some(*frame);
            }
        }

        assert!(result.is_none(), "Decoder accepted a packet containing a corrupted CRC byte!");
    }

    #[test]
    fn test_noise_and_recovery() {
        let mut decoder = CrsfDecoder::new();
        let input_channels = [1500; 16];
        let valid_packet = create_valid_crsf_packet(input_channels);

        // Create a fixed stack noise prefix array (6 bytes)
        let noise = [0x00, 0xC7, 0xC8, 0x02, 0x16, 0xFF]; // Includes a false start 0xC8

        let mut noisy_stream = [0u8; 6 + 26];
        noisy_stream[..6].copy_from_slice(&noise);
        noisy_stream[6..].copy_from_slice(&valid_packet);

        let mut decoded_frame = None;
        for &byte in noisy_stream.iter() {
            if let Some(frame) = decoder.on_byte_received(byte) {
                decoded_frame = Some(*frame);
            }
        }

        assert!(decoded_frame.is_some(), "Decoder failed to re-sync and recover after stream noise!");
        assert_eq!(decoded_frame.unwrap().channels, [1500; 16]);
    }
}
