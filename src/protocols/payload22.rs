pub struct Payload22 {}

impl Payload22 {
    /// SBUS supports 18 channels, but we only use 16 of them.
    pub const CHANNEL_COUNT: usize = 16;

    pub const PAYLOAD_LENGTH: usize = 22;
    pub const HALF_PAYLOAD_LENGTH: usize = 11;

    /// Extracts 16 channels from a 22-byte SBUS/CRSF payload.
    /// SBUS/CRSF uses 11 bits per channel, Little-Endian bit-packing.
    /// Because 11 bits don't divide evenly into 8-bit bytes, the pattern repeats after 8 channels (ie after 11 bytes).
    #[inline]
    pub fn parse_payload(payload: &[u8; Self::PAYLOAD_LENGTH]) -> [u16; Self::CHANNEL_COUNT] {
        let mut channels = [0u16; Self::CHANNEL_COUNT];

        // .as_chunks::<11>().0 returns a slice of [u8; 11] arrays
        let chunks = payload.as_chunks::<11>().0;

        // Process the first 11 bytes into channels 0..8
        Self::parse_payload_chunk(&mut channels[0..8], &chunks[0]);

        // Process the next 11 bytes into channels 8..16
        Self::parse_payload_chunk(&mut channels[8..16], &chunks[1]);

        channels
    }

    /// Fast 32-bit overlapping window channel extraction.
    #[inline]
    fn parse_payload_chunk(out: &mut [u16], chunk: &[u8; Self::HALF_PAYLOAD_LENGTH]) {
        // Slurp bytes in 32-bit windows using zero-overhead from_le_bytes
        // This allows the CPU to use 32-bit hardware registers and barrel shifters

        // Window 1: Bytes 0, 1, 2, 3 (Contains Ch 0, 1, and parts of 2)
        let w0 = u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
        out[0] = (w0 & 0x7FF) as u16;
        out[1] = ((w0 >> 11) & 0x7FF) as u16;

        // Window 2: Bytes 2, 3, 4, 5 (Contains Ch 2, 3, and parts of 4)
        let w1 = u32::from_le_bytes([chunk[2], chunk[3], chunk[4], chunk[5]]);
        out[2] = ((w1 >> 6) & 0x7FF) as u16;
        out[3] = ((w1 >> 17) & 0x7FF) as u16;

        // Window 3: Bytes 5, 6, 7, 8 (Contains Ch 4, 5, and parts of 6)
        let w2 = u32::from_le_bytes([chunk[5], chunk[6], chunk[7], chunk[8]]);
        out[4] = ((w2 >> 4) & 0x7FF) as u16;
        out[5] = ((w2 >> 15) & 0x7FF) as u16;

        // Window 4: Bytes 8, 9, 10, plus a trailing 0 padding byte
        let w3 = u32::from_le_bytes([chunk[8], chunk[9], chunk[10], 0]);
        out[6] = ((w3 >> 2) & 0x7FF) as u16;
        out[7] = ((w3 >> 13) & 0x7FF) as u16;
    }
    /*
    /// Bitmasking: Every line ends with & 0x07FF. This ensures that even if bits "bleed" over from the next byte, only the 11 bits we care about are kept.
    /// Performance: On a typical 32-bit MCU , the compiler will optimize these into simple LDR, LSR/LSL, and AND instructions.
    pub fn parse_sbus_channels(p: &[u8; 22]) -> [u16; 16] {
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

    /// Helper function to pack a channels into payload of 11-bit values.
    #[allow(unused)]
    pub fn generate_payload(channels: [u16; Self::CHANNEL_COUNT]) -> [u8; Self::PAYLOAD_LENGTH] {
        let mut payload = [0u8; 22];
        let mut bit_bucket: u32 = 0;
        let mut bits_in_bucket: u32 = 0;
        let mut byte_index: usize = 0;

        for channel in &channels {
            // Mask to ensure the value stays strictly constrained inside 11 bits (0..2047)
            let channel = channel & 0x07FF;

            // Push the 11 bits onto our active buffer accumulator
            bit_bucket |= u32::from(channel) << bits_in_bucket;
            bits_in_bucket += 11;

            // Drain whole bytes out of our bit accumulator into the final buffer array
            while bits_in_bucket >= 8 {
                payload[byte_index] = (bit_bucket & 0xFF) as u8;
                byte_index += 1;
                bit_bucket >>= 8;
                bits_in_bucket -= 8;
            }
        }

        // Flush any remaining trailing bits into the final payload slots
        if bits_in_bucket > 0 && byte_index < 22 {
            payload[byte_index] = (bit_bucket & 0xFF) as u8;
        }

        payload
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sbus_32bit_window_decoder() {
        // Establish an arbitrary, diverse set of test channels (values 0..2047)
        let channels = [1000, 1500, 172, 2000, 111, 1890, 512, 1024, 1500, 992, 1234, 45, 2047, 0, 777, 1520];

        let sbus_payload = Payload22::generate_payload(channels);
        assert_eq!(
            sbus_payload,
            [
                0xe8, 0xe3, 0x2e, 0x2b, 0xa0, 0xff, 0x06, 0xb1, 0x03, 0x08, 0x80, 0xdc, 0x05, 0x9f, 0x34, 0x5b, 0xf0,
                0x7f, 0x00, 0x24, 0x0c, 0xbe
            ]
        );

        let decoded_channels = Payload22::parse_payload(&sbus_payload);

        assert_eq!(
            decoded_channels, channels,
            "The optimized 32-bit overlapping window bit shift algorithm misaligned channel data!"
        );
    }

    #[test]
    fn test_sbus_decoder_extremes() {
        // Test edge cases: Every single bit turned entirely on (2047) or off (0)
        let max_channels = [2047u16; 16];
        let min_channels = [0u16; 16];

        let max_payload = Payload22::generate_payload(max_channels);
        let min_payload = Payload22::generate_payload(min_channels);

        assert_eq!(Payload22::parse_payload(&max_payload), max_channels, "Failed to decode max 11-bit boundaries");
        assert_eq!(Payload22::parse_payload(&min_payload), min_channels, "Failed to decode min 11-bit boundaries");
    }
}
