#![allow(unused)]
//use embedded_hal_nb::serial::{Error};
use crate::{RxChannel, RxFrame, RxLinkStatus};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SbusFrame {
    pub channels: [u16; Self::CHANNEL_COUNT],
    pub flags: u8,
    pub rssi: u8,
}

impl SbusFrame {
    const AUX13: u8 = 0x01;
    const AUX14: u8 = 0x02;
    const FRAME_LOST: u8 = 0x04;
    const FAILSAFE: u8 = 0x08;

    pub const fn new() -> Self {
        Self { channels: [0u16; Self::CHANNEL_COUNT], flags: 0, rssi: 0 }
    }
}

impl Default for SbusFrame {
    fn default() -> Self {
        Self::new()
    }
}

impl SbusFrame {
    // Forego channels AUX13 and AUX14, since we've limited CHANNEL_COUNT to 16,
    // since no other protocols use more than 16 channels.
    // channels[RxChannel::AUX13] = if frame.flags.aux13 { RxChannel::HIGH } else { RxChannel::LOW };
    // channels[RxChannel::AUX14] = if frame.flags.aux14 { RxChannel::HIGH } else { RxChannel::LOW };
    const CHANNEL_COUNT: usize = 16;

    const PWM_LOW: u32 = 172;
    const PWM_HIGH: u32 = 1811;
    const PWM_RANGE: u32 = Self::PWM_HIGH - Self::PWM_LOW;

    /// SBUS values typically range from 172 to 1811 (representing 1000µs to 2000µs),
    /// so they need to be normalized to the standard PWM range `[1000,2000]`.
    #[allow(clippy::cast_possible_truncation)]
    pub fn normalize_channels(input: &[u16; Self::CHANNEL_COUNT]) -> [u16; Self::CHANNEL_COUNT] {
        let mut output = [0u16; Self::CHANNEL_COUNT];

        for (in_val, out_val) in input.iter().zip(output.iter_mut()) {
            let val = u32::from(*in_val).clamp(Self::PWM_LOW, Self::PWM_HIGH);
            *out_val = ((val - Self::PWM_LOW) * u32::from(RxChannel::RANGE) / (Self::PWM_RANGE)
                + u32::from(RxChannel::LOW)) as u16;
        }
        output
    }
}

impl From<SbusFrame> for RxFrame {
    fn from(frame: SbusFrame) -> Self {
        //let flags = SbusFlags::from_byte(raw_buffer[23]);
        let status = if frame.flags & SbusFrame::FAILSAFE != 0 {
            RxLinkStatus::Failsafe
        } else if frame.flags & SbusFrame::FRAME_LOST != 0 {
            RxLinkStatus::NoSignal
        } else {
            RxLinkStatus::Ok
        };

        let mut channels = [Self::DEFAULT_CHANNEL_VALUE; Self::MAX_CHANNEL_COUNT];
        channels[..frame.channels.len()].copy_from_slice(&frame.channels);

        Self { channels, status, rssi: frame.rssi }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
enum ParserState {
    #[default]
    WaitingForHeader,
    CollectingPayload {
        index: usize,
    },
    ValidatingFooter,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct SbusParser {
    state: ParserState,
    buffer: [u8; Self::PACKET_LENGTH],
}

impl SbusParser {
    pub const HEADER_LENGTH: usize = 1;
    pub const PAYLOAD_LENGTH: usize = 22;
    pub const PACKET_LENGTH: usize = 25;

    pub const fn new() -> Self {
        Self { state: ParserState::WaitingForHeader, buffer: [0u8; Self::PACKET_LENGTH] }
    }
}

impl Default for SbusParser {
    fn default() -> Self {
        Self::new()
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
impl SbusParser {
    /// The feed Method
    /// This is the core logic. It takes one byte and returns a Some(Frame) when a full, valid packet is completed.
    pub fn feed(&mut self, byte: u8) -> Option<SbusFrame> {
        match self.state {
            ParserState::WaitingForHeader => {
                if byte == 0x0F {
                    // We have a valid header byte, so start collecting the payload.
                    self.buffer[0] = byte;
                    self.state = ParserState::CollectingPayload { index: 1 };
                }
            }
            // Collect the 22 bytes of payload.
            ParserState::CollectingPayload { ref mut index } => {
                self.buffer[*index] = byte;
                *index += 1;

                // When we have collected the payload, move onto the footer.
                if *index > Self::HEADER_LENGTH + Self::PAYLOAD_LENGTH {
                    self.state = ParserState::ValidatingFooter;
                }
            }
            ParserState::ValidatingFooter => {
                self.state = ParserState::WaitingForHeader;
                if byte == 0x00 {
                    // SBUS Footer
                    let data: [u8; Self::PAYLOAD_LENGTH] = match self.buffer[1..23].try_into() {
                        Ok(arr) => arr,
                        Err(_) => [0u8; Self::PAYLOAD_LENGTH], // just return an empty array
                    };
                    let channels = Self::parse_sbus_channels(&data);
                    let sbus_frame = SbusFrame { channels, flags: 0, rssi: 0 };

                    return Some(sbus_frame);
                }
            }
        }
        None
    }

    pub fn parse(&mut self, buffer: &[u8; Self::PACKET_LENGTH]) -> Option<SbusFrame> {
        for byte in buffer {
            if let Some(frame) = self.feed(*byte) {
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
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn is_normal<T: Sized + Send + Sync + Unpin>() {}
    fn is_full<T: Sized + Send + Sync + Unpin + Copy + Clone + Default + PartialEq>() {}

    #[test]
    fn normal_types() {
        is_full::<SbusFrame>();
        is_full::<SbusParser>();
        is_full::<ParserState>();
    }
    #[test]
    fn new() {
        let frame = SbusFrame::default();
        assert_eq!(0, frame.rssi);
    }
    #[test]
    fn parse_message() {
        #[rustfmt::skip]
        let stream: [u8; SbusParser::PACKET_LENGTH] = [
            0x0F, // header
            // 22 u8s
            0xE0, 0x03, 0x1F, 0x58, 0xC0, 0x07, 0x16, 0xB0, 0x80, 0x05, 0x2C, 
            0x60, 0x01, 0x0B, 0xF8, 0xC0, 0x07, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x03, // flags
            0x00, // footer
        ];

        let expected_channels: [u16; SbusFrame::CHANNEL_COUNT] =
            [992, 992, 352, 992, 352, 352, 352, 352, 352, 352, 992, 992, 0, 0, 0, 0];

        let mut sbus_parser = SbusParser::new();
        if let Some(frame) = sbus_parser.parse(&stream) {
            let channels = frame.channels;
            assert_eq!(expected_channels, channels);
        } else {
            unreachable!();
        }
    }
}
