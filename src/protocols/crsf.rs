#![allow(unused)]
use crate::{protocols::CrcDvbS2, rx_radio::RxChannels};

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

    /*fn calculate_packet_crc(packet: [u8; Self::MAX_PACKET_SIZE]) -> u8 {
        let packet_length = packet[Self::PACKET_LENGTH_OFFSET] as usize;
        let mut ii = Self::PACKET_TYPE_OFFSET;
        let mut crc = Self::calculate_crc(0, packet[ii]);
        while ii < packet_length {
            // length is length of type, payload, and CRC
            crc = Self::calculate_crc(crc, packet[ii]);
            ii += 1;
        }
        crc
    }*/

    fn received_crc(packet: [u8; Self::MAX_PACKET_SIZE]) -> u8 {
        let packet_length = packet[Self::PACKET_LENGTH_OFFSET] as usize;
        packet[packet_length - 2]
    }

    #[cfg(test)]
    fn _pack_crsf_payload(channels: RxChannels) -> CrsfPayload {
        let mut bits: u32 = 0;
        let mut bit_count: u32 = 0;
        let mut bytes = CrsfPayload::default();
        let mut byte_idx = 0;

        for &ch in &channels {
            bits |= u32::from(ch) << bit_count;
            bit_count += 11;
            #[allow(clippy::cast_possible_truncation)]
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
        if CrcDvbS2::calculate(&packet) != Self::received_crc(packet) {
            //self.radio_serial.packet_is_empty = true;
            return CrsfPayload::default();
        }
        let packet_type = packet[Self::PACKET_TYPE_OFFSET];
        if packet_type == Self::FRAMETYPE_RC_CHANNELS_PACKED {
            //self.radio_serial.packet_is_empty = false;
            // unpack packet_length bytes starting at PACKET_DATA_OFFSET
            //let packet_length = packet[Self::PACKET_LENGTH_OFFSET] as usize;

            let data: CrsfPayload = packet[Self::PACKET_DATA_OFFSET..Self::PACKET_DATA_OFFSET + Self::PACKET_LENGTH]
                .try_into()
                .unwrap_or_default();
            //let count;
            //(self.channels, count) = CrsfReceiver::unpack_11bit_channels(&data);
            //return count;
            return data;
        }
        CrsfPayload::default()
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct CrsfLinkStatistics {
    pub uplink_rssi_dbm: i8, // Usually -30 to -120
    pub uplink_lq: u8,       // 0 - 100
    pub uplink_snr: i8,      // Signal-to-Noise Ratio
    pub rf_mode: u8,         // 0=4Hz, 1=50Hz, 2=150Hz, etc.
    pub tx_power_mw: u16,    // Mapped from the power enum
    pub downlink_rssi_dbm: i8,
    pub downlink_lq: u8,
}

impl Default for CrsfLinkStatistics {
    fn default() -> Self {
        Self::new()
    }
}

impl CrsfLinkStatistics {
    pub const fn new() -> Self {
        Self {
            uplink_rssi_dbm: 0,
            uplink_lq: 0,
            uplink_snr: 0,
            rf_mode: 0,
            tx_power_mw: 0,
            downlink_rssi_dbm: 0,
            downlink_lq: 0,
        }
    }
}

impl CrsfLinkStatistics {
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

    fn is_normal<T: Sized + Send + Sync + Unpin>() {}
    fn is_full<T: Sized + Send + Sync + Unpin + Copy + Clone + Default + PartialEq>() {}

    #[test]
    fn normal_types() {
        is_normal::<CrsfPacket>();
        is_full::<CrsfParser>();
        is_full::<CrsfLinkStatistics>();
    }
    #[test]
    fn link_statistics() {
        let link_statistics = CrsfLinkStatistics::default();
        assert_eq!(0, link_statistics.uplink_rssi_dbm);
    }
}
