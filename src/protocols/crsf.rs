#![allow(unused)]
use crate::rx_receiver::RxChannels;

/// Precomputed CRC8 table for polynomial 0xD5 (DVB-S2).
const CRC8_TABLE: [u8; 256] = [
    0x00, 0xD5, 0x7F, 0xAA, 0xFE, 0x2B, 0x81, 0x54, 0x2D, 0xF8, 0x52, 0x87, 0xD3, 0x06, 0xAC, 0x79, 0x5A, 0x8F, 0x25,
    0xF0, 0xA4, 0x71, 0xDB, 0x0E, 0x77, 0xA2, 0x08, 0xDD, 0x89, 0x5C, 0xF6, 0x23, 0xB4, 0x61, 0xCB, 0x1E, 0x4A, 0x9F,
    0x35, 0xE0, 0x99, 0x4C, 0xE6, 0x33, 0x67, 0xB2, 0x18, 0xCD, 0xEE, 0x3B, 0x91, 0x44, 0x10, 0xC5, 0x6F, 0xBA, 0xC3,
    0x16, 0xBC, 0x69, 0x3D, 0xE8, 0x42, 0x97, 0x39, 0xEC, 0x46, 0x93, 0xC7, 0x12, 0xB8, 0x6D, 0x14, 0xC1, 0x6B, 0xBE,
    0xEA, 0x3F, 0x95, 0x40, 0x63, 0xB6, 0x1C, 0xC9, 0x9D, 0x48, 0xE2, 0x37, 0x4E, 0x9B, 0x31, 0xE4, 0xB0, 0x65, 0xCF,
    0x1A, 0x8D, 0x58, 0xF2, 0x27, 0x73, 0xA6, 0x0C, 0xD9, 0xA0, 0x75, 0xDF, 0x0A, 0x5E, 0x8B, 0x21, 0xF4, 0xD7, 0x02,
    0xA8, 0x7D, 0x29, 0xFC, 0x56, 0x83, 0xFA, 0x2F, 0x85, 0x50, 0x04, 0xD1, 0x7B, 0xAE, 0x72, 0xA7, 0x0D, 0xD8, 0x8C,
    0x59, 0xF3, 0x26, 0x5F, 0x8A, 0x20, 0xF5, 0xA1, 0x74, 0xDE, 0x0B, 0x28, 0xFD, 0x57, 0x82, 0xD6, 0x03, 0xA9, 0x7C,
    0x05, 0xD0, 0x7A, 0xAB, 0xFB, 0x2E, 0x84, 0x51, 0xC6, 0x13, 0xB9, 0x6C, 0x38, 0xED, 0x47, 0x92, 0xEB, 0x3E, 0x94,
    0x41, 0x15, 0xC0, 0x6A, 0xBF, 0x9C, 0x49, 0xE3, 0x36, 0x62, 0xB7, 0x1D, 0xC8, 0xB1, 0x64, 0xCE, 0x1B, 0x4F, 0x9A,
    0x30, 0xE5, 0x4B, 0x9E, 0x34, 0xE1, 0xB5, 0x60, 0xCA, 0x1F, 0x66, 0xB3, 0x19, 0xCC, 0x98, 0x4D, 0xE7, 0x32, 0x11,
    0xC4, 0x6E, 0xBB, 0xEF, 0x3A, 0x90, 0x45, 0x3C, 0xE9, 0x43, 0x96, 0xC2, 0x17, 0xBD, 0x68, 0xFF, 0x2A, 0x80, 0x55,
    0x01, 0xD4, 0x7E, 0xAB, 0xD2, 0x07, 0xAD, 0x78, 0x2C, 0xF9, 0x53, 0x86, 0xA5, 0x70, 0xDA, 0x0F, 0x5B, 0x8E, 0x24,
    0xF1, 0x88, 0x5D, 0xF7, 0x22, 0x76, 0xA3, 0x09, 0xDC,
];

/// `CrsfPacket` is represented as an enum, as per Rust idiom.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum CrsfPacket {
    Channels(RxChannels),
    LinkStatistics {
        rssi_dbm: u8,
        lq: u8,
        rf_mode: u8,
    },
    Battery {
        voltage: u16, // deci-volts
        current: u16, // deci-amps
    },
    Unknown(u8),
}

pub type CrsfPayload = [u8; CrsfParser::PACKET_LENGTH];

#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct CrsfParser {}

impl CrsfParser {
    pub const MAX_PACKET_SIZE: usize = 64;

    pub const PACKET_LENGTH: usize = 22;

    const _PACKET_SYNC_OFFSET: usize = 0;
    const PACKET_LENGTH_OFFSET: usize = 1;
    const PACKET_TYPE_OFFSET: usize = 2;
    const PACKET_DATA_OFFSET: usize = 3;

    // see https://github.com/crsf-wg/crsf/wiki/Packet-Types
    const _FRAMETYPE_GPS: u8 = 0x02;
    const _FRAMETYPE_VARIO_SENSOR: u8 = 0x07;
    const FRAMETYPE_BATTERY_SENSOR: u8 = 0x08;
    const _FRAMETYPE_BARO_ALTITUDE: u8 = 0x09;
    const _FRAMETYPE_HEARTBEAT: u8 = 0x0B;
    const _FRAMETYPE_LINK_STATISTICS: u8 = 0x14;
    const FRAMETYPE_RC_CHANNELS_PACKED: u8 = 0x16;
    const _FRAMETYPE_SUBSET_RC_CHANNELS_PACKED: u8 = 0x17;
    const _FRAMETYPE_LINK_STATISTICS_RX: u8 = 0x1C;
    const _FRAMETYPE_LINK_STATISTICS_TX: u8 = 0x1D;
    const _FRAMETYPE_ATTITUDE: u8 = 0x1E;
    const _FRAMETYPE_FLIGHT_MODE: u8 = 0x21;
    // Extended Header Frames; range: 0x28 to 0x96
    const _FRAMETYPE_DEVICE_PING: u8 = 0x28;
    const _FRAMETYPE_DEVICE_INFO: u8 = 0x29;
    const _FRAMETYPE_PARAMETER_SETTINGS_ENTRY: u8 = 0x2B;
    const _FRAMETYPE_PARAMETER_READ: u8 = 0x2C;
    const _FRAMETYPE_PARAMETER_WRITE: u8 = 0x2D;
    const _FRAMETYPE_COMMAND: u8 = 0x32;
    // MSP commands
    const _FRAMETYPE_MSP_REQ: u8 = 0x7A;
    const _FRAMETYPE_MSP_RESP: u8 = 0x7B;
    const _FRAMETYPE_MSP_WRITE: u8 = 0x7C;
    const _FRAMETYPE_DISPLAYPORT_CMD: u8 = 0x7D;
    const _FRAMETYPE_ARDUPILOT_RESP: u8 = 0x80;

    pub fn parse_payload(packet_type: u8, payload: &[u8]) -> CrsfPacket {
        match packet_type {
            Self::FRAMETYPE_RC_CHANNELS_PACKED => {
                // Use the 11-bit extraction logic we discussed for SBUS
                let channels = Self::parse_crsf_channels(payload);
                CrsfPacket::Channels(channels)
            }
            0x21 => {
                // Link Statistics (Type 0x21)
                CrsfPacket::LinkStatistics { rssi_dbm: payload[0], lq: payload[1], rf_mode: payload[3] }
            }
            Self::FRAMETYPE_BATTERY_SENSOR => {
                // Battery (Type 0x08)
                // Big-Endian packing: [Volt High] [Volt Low] [Curr High] [Curr Low] ...
                let voltage = u16::from_be_bytes([payload[0], payload[1]]);
                let current = u16::from_be_bytes([payload[2], payload[3]]);
                CrsfPacket::Battery { voltage, current }
            }
            other => CrsfPacket::Unknown(other),
        }
    }
    pub fn parse_crsf_channels(_payload: &[u8]) -> RxChannels {
        RxChannels::default()
    }

    pub fn crsf_crc8(data: &[u8]) -> u8 {
        let mut crc = 0u8;
        for &byte in data {
            crc = CRC8_TABLE[(crc ^ byte) as usize];
        }
        crc
    }

    pub fn calculate_crc(crc: u8, value: u8) -> u8 {
        const POLYNOMIAL: u8 = 0xD5;

        let mut crc = crc;
        crc ^= value;
        for _ in 0..8 {
            let top_bit = crc & 0x80;
            crc <<= 1;
            if top_bit != 0 {
                crc ^= POLYNOMIAL;
            }
        }
        crc
    }

    fn _calculate_packet_crc(packet: [u8; Self::MAX_PACKET_SIZE]) -> u8 {
        let packet_length = packet[Self::PACKET_LENGTH_OFFSET] as usize;
        let mut ii = Self::PACKET_TYPE_OFFSET;
        let mut crc = Self::calculate_crc(0, packet[ii]);
        while ii < packet_length {
            // length is length of type, payload, and CRC
            crc = Self::calculate_crc(crc, packet[ii]);
            ii += 1;
        }
        crc
    }

    fn received_crc(packet: [u8; Self::MAX_PACKET_SIZE]) -> u8 {
        let packet_length = packet[Self::PACKET_LENGTH_OFFSET] as usize;
        packet[packet_length - 2]
    }

    #[cfg(test)]
    #[allow(clippy::cast_possible_truncation)]
    fn _pack_crsf_payload(channels: RxChannels) -> CrsfPayload {
        let mut bits: u32 = 0;
        let mut bit_count: u32 = 0;
        let mut bytes = CrsfPayload::default();
        let mut byte_idx = 0;

        for &ch in &channels {
            bits |= u32::from(ch) << bit_count;
            bit_count += 11;
            while bit_count >= 8 && byte_idx < 22 {
                bytes[byte_idx] = bits as u8;
                bits >>= 8;
                bit_count -= 8;
                byte_idx += 1;
            }
        }
        bytes
    }

    /// Convert packed payload into channels.
    pub fn unpack_crsf_channels(data: &[u8]) -> (RxChannels, usize) {
        let mut result = RxChannels::default();
        let mut bit_offset = 0;
        let mut count = 0;

        for value in &mut result {
            if bit_offset + 11 > data.len() * 8 {
                break;
            }
            let byte_idx = bit_offset / 8;
            let bit_idx = bit_offset % 8;

            let mut bits = u16::from(data[byte_idx]) << 8;
            if byte_idx + 1 < data.len() {
                bits |= u16::from(data[byte_idx + 1]);
            }

            *value = (bits >> (16 - 11 - bit_idx)) & 0x7FF; // Extract 11 bits
            bit_offset += 11;
            count += 1;
        }
        (result, count)
    }

    /// A CRSF packet always follows this pattern:
    /// `[Sync] [Length] [Type] [Payload...] [CRC]`
    /// Note: Length includes everything from Type to CRC.
    pub fn _unpack_packet(packet: [u8; Self::MAX_PACKET_SIZE]) -> CrsfPayload {
        if Self::crsf_crc8(&packet) != Self::received_crc(packet) {
            //self.receiver_serial.packet_is_empty = true;
            return CrsfPayload::default();
        }
        let packet_type = packet[Self::PACKET_TYPE_OFFSET];
        if packet_type == Self::FRAMETYPE_RC_CHANNELS_PACKED {
            //self.receiver_serial.packet_is_empty = false;
            // unpack packet_length bytes starting at PACKET_DATA_OFFSET
            //let packet_length = packet[Self::PACKET_LENGTH_OFFSET] as usize;

            let data: CrsfPayload =
                match packet[Self::PACKET_DATA_OFFSET..Self::PACKET_DATA_OFFSET + Self::PACKET_LENGTH].try_into() {
                    Ok(arr) => arr,
                    Err(_) => CrsfPayload::default(), // just return an empty payload
                };
            //let count;
            //(self.channels, count) = CrsfReceiver::unpack_11bit_channels(&data);
            //return count;
            return data;
        }
        CrsfPayload::default()
    }
}

#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct LinkStatistics {
    pub uplink_rssi_dbm: i8, // Usually -30 to -120
    pub uplink_lq: u8,       // 0 - 100
    pub uplink_snr: i8,      // Signal-to-Noise Ratio
    pub rf_mode: u8,         // 0=4Hz, 1=50Hz, 2=150Hz, etc.
    pub tx_power_mw: u16,    // Mapped from the power enum
    pub downlink_rssi_dbm: i8,
    pub downlink_lq: u8,
}

impl LinkStatistics {
    pub fn parse(payload: &[u8]) -> Self {
        // Basic bounds check to prevent panics in no_std
        if payload.len() < 10 {
            return Self::default();
        }

        Self {
            // CRSF sends RSSI as positive (e.g. 60), actual is -60dBm
            uplink_rssi_dbm: -payload[0].cast_signed(),
            uplink_lq: payload[2],
            uplink_snr: payload[3].cast_signed(),
            rf_mode: payload[5],
            tx_power_mw: Self::map_tx_power(payload[6]),
            downlink_rssi_dbm: -payload[7].cast_signed(),
            downlink_lq: payload[8],
        }
    }

    fn map_tx_power(val: u8) -> u16 {
        match val {
            1 => 10,
            2 => 25,
            3 => 100,
            4 => 500,
            5 => 1000,
            6 => 2000,
            _ => 0,
        }
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
        is_normal::<CrsfPacket>();
        is_full::<CrsfParser>();
        is_full::<LinkStatistics>();
    }
    #[test]
    fn link_statistics() {
        let link_statistics = LinkStatistics::default();
        assert_eq!(0, link_statistics.uplink_rssi_dbm);
    }
}
