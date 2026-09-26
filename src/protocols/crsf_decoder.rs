#![allow(unused)]

use crate::{RxChannels, RxChannelsLinkStatus, RxFrame, RxFrameType, RxLinkStatus};

use super::CrcDvbS2;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum State {
    #[default]
    WaitForSyncByte,
    WaitForLengthByte,
    ReadPayload {
        index: usize,
        length: usize,
    },
    ValidateChecksum {
        length: usize,
    },
}

impl State {
    const SYNC_BYTE: u8 = 0xC8;
}
/// packet is composed as follows
/// byte 0: sync;
/// byte 1: length; // length is length of type, payload, and CRC
/// byte 2: type;
/// 22 bytes of payload 176 bits of data (11 bits per channel * 16 channels) = 22 bytes.
/// CRC byte after payload. CRC is calculated on all bytes from type to end of payload.
/// ie `[Sync] [Length] [Type] [Payload...] [CRC]`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CrsfDecoder {
    state: State,
    // Stores Type (1 byte) + Channel Payload (22 bytes)
    buffer: [u8; Self::MAX_PACKET_SIZE],
}

impl Default for CrsfDecoder {
    fn default() -> Self {
        Self::new()
    }
}

impl CrsfDecoder {
    pub const MAX_PACKET_SIZE: usize = 64;

    /// Length of the bit-packed channels block (16 channels * 11 bits = 176 bits = 22 bytes).
    pub const RC_PACKET_LENGTH: usize = 22;
    pub(crate) const HALF_RC_PACKET_LENGTH: usize = 11;

    pub const CHANNEL_COUNT: usize = 16;

    pub const fn new() -> Self {
        Self { state: State::WaitForSyncByte, buffer: [0u8; Self::MAX_PACKET_SIZE] }
    }
}

impl CrsfDecoder {
    /// Feeds a single byte into the CRSF state machine.
    /// Returns `Some(RxFrame)` on successfully parsing a CRSF frame.
    /// Currently only handles `FRAMETYPE_RC_CHANNELS_PACKED`.
    pub fn on_byte_received(&mut self, byte: u8) -> Option<RxFrame> {
        let mut complete = false;

        self.state = match core::mem::take(&mut self.state) {
            State::WaitForSyncByte => {
                if byte == State::SYNC_BYTE {
                    State::WaitForLengthByte
                } else {
                    State::WaitForSyncByte
                }
            }
            State::WaitForLengthByte => {
                let payload_len = byte as usize;
                if (3..=(Self::MAX_PACKET_SIZE)).contains(&payload_len) {
                    State::ReadPayload { index: 0, length: payload_len }
                } else {
                    State::WaitForSyncByte
                }
            }
            State::ReadPayload { index, length } => {
                // Let index fill all payload bytes completely (Type + Payload = expected_len - 1).
                // For a standard frame of length 24, this will fill buffer indices 0 to 22.
                self.buffer[index] = byte;

                if index >= length - 2 {
                    State::ValidateChecksum { length }
                } else {
                    let index = index + 1;
                    State::ReadPayload { index, length }
                }
            }
            State::ValidateChecksum { length } => {
                const BUFFER_SIZE: usize = 23;
                // This byte is now the CRC byte
                let received_crc = byte;

                // length is length of type, payload, and CRC
                // CRC is calculated on all bytes from type to end of payload
                // CRSF payload for checksum is always (Length - 1).
                //let payload = &self.buffer[..BUFFER_SIZE];
                let payload = &self.buffer[..length - 1];

                if CrcDvbS2::calculate(payload) == received_crc {
                    complete = true;
                }
                State::WaitForSyncByte
            }
        };

        if complete {
            let mut channels = [0; RxChannelsLinkStatus::CHANNEL_COUNT];
            let frame_type = self.buffer[0];
            if frame_type == RxFrameType::RcChannels as u8 {
                if let Ok(channel_data) = self.buffer[1..23].try_into() {
                    Self::parse_rc_channels(&mut channels, channel_data);
                    let channels = RxChannels::from_channels(channels);
                    let link_status = RxLinkStatus::Ok;
                    let channels_link = RxChannelsLinkStatus { channels, link_status };

                    return Some(RxFrame::ChannelsLink { channels_link });
                }
                return None;
            }
            let frame_type = RxFrameType::from_u8(frame_type);
            let rx_frame = match frame_type {
                RxFrameType::Heartbeat => Some(RxFrame::Heartbeat()),
                RxFrameType::LinkStatisticsTx => Some(RxFrame::LinkStatisticsTx {
                    rssi_dbm: self.buffer[1],
                    rssi_percent: self.buffer[2],
                    link_quality: self.buffer[3],
                    snr: self.buffer[4].cast_signed(),
                }),
                RxFrameType::BatterySensor => {
                    // Battery (Type 0x08)
                    // Big-Endian packing: [Volt High] [Volt Low] [Curr High] [Curr Low] ...
                    let voltage = u16::from_be_bytes([self.buffer[1], self.buffer[2]]);
                    let current = u16::from_be_bytes([self.buffer[3], self.buffer[4]]);
                    Some(RxFrame::Battery { voltage, current })
                }

                _ => Some(RxFrame::Unknown { frame_type: self.buffer[0] }),
            };
        }

        None
    }

    /// Fast 32-bit overlapping window channel extraction.
    pub fn parse_rc_channels(channels: &mut [u16; Self::CHANNEL_COUNT], payload: &[u8; Self::RC_PACKET_LENGTH]) {
        let chunks = payload.as_chunks::<11>().0;

        Self::parse_8_rc_channels(&mut channels[0..8], &chunks[0]);
        Self::parse_8_rc_channels(&mut channels[8..16], &chunks[1]);
    }

    #[inline]
    fn parse_8_rc_channels(out: &mut [u16], s: &[u8; Self::HALF_RC_PACKET_LENGTH]) {
        let w0 = u32::from_le_bytes([s[0], s[1], s[2], s[3]]);
        out[0] = (w0 & 0x7FF) as u16;
        out[1] = ((w0 >> 11) & 0x7FF) as u16;

        let w1 = u32::from_le_bytes([s[2], s[3], s[4], s[5]]);
        out[2] = ((w1 >> 6) & 0x7FF) as u16;
        out[3] = ((w1 >> 17) & 0x7FF) as u16;

        let w2 = u32::from_le_bytes([s[5], s[6], s[7], s[8]]);
        out[4] = ((w2 >> 4) & 0x7FF) as u16;
        out[5] = ((w2 >> 15) & 0x7FF) as u16;

        let w3 = u32::from_le_bytes([s[8], s[9], s[10], 0]);
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
    use crate::rx_frame;

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
    fn create_crsf_packet(input_channels: [u16; 16]) -> [u8; 26] {
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

        let raw_stream = create_crsf_packet(input_channels);

        // Feed the stream into the state machine byte-by-byte
        let mut result = None;
        for &byte in raw_stream.iter() {
            if let Some(frame) = decoder.on_byte_received(byte) {
                result = Some(frame); // Copy the reference data out safely for evaluation
            }
        }

        // Assert the state machine accurately matched the complete packet
        assert!(result.is_some(), "Decoder failed to yield channels on final frame byte!");
        let output_frame = result.unwrap();

        match output_frame {
            RxFrame::ChannelsLink { channels_link } => {
                assert_eq!(
                    channels_link.channels.channels(),
                    input_channels,
                    "Decoded values do not match original 11-bit input boundaries!"
                );
            }
            _ => {
                panic!("decoded to wrong frame type")
            }
        }
    }

    #[test]
    fn test_corrupted_crc_rejection() {
        let mut decoder = CrsfDecoder::new();
        let input_channels = [1000; 16];
        let mut raw_stream = create_crsf_packet(input_channels);

        // Corrupt the final CRC byte intentionally
        let last_idx = raw_stream.len() - 1;
        raw_stream[last_idx] ^= 0x5A;

        let mut result = None;
        for &byte in raw_stream.iter() {
            if let Some(frame) = decoder.on_byte_received(byte) {
                result = Some(frame);
            }
        }

        assert!(result.is_none(), "Decoder accepted a packet containing a corrupted CRC byte!");
    }

    #[test]
    fn test_noise_and_recovery() {
        let mut decoder = CrsfDecoder::new();
        let input_channels = [1500; 16];
        let valid_packet = create_crsf_packet(input_channels);

        // Create a fixed stack noise prefix array (6 bytes)
        let noise = [0x00, 0xC7, 0xC8, 0x02, 0x16, 0xFF]; // Includes a false start 0xC8
        let mut noisy_stream = [0u8; 6 + 26];
        noisy_stream[..6].copy_from_slice(&noise);
        noisy_stream[6..].copy_from_slice(&valid_packet);

        let mut decoded_frame = None;
        for &byte in noisy_stream.iter() {
            if let Some(frame) = decoder.on_byte_received(byte) {
                decoded_frame = Some(frame);
            }
        }

        assert!(decoded_frame.is_some(), "Decoder failed to re-sync and recover after stream noise!");
        let rx_frame = decoded_frame.unwrap();
        match rx_frame {
            RxFrame::ChannelsLink { channels_link } => {
                assert_eq!(channels_link.channels.channels(), [1500; 16]);
            }
            _ => {
                panic!("decoded to wrong frame type")
            }
        }
    }
}
