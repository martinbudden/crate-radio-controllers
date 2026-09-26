use crate::{RxChannel, RxFrame, RxLinkStatus};

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct SbusFrame {
    pub channels: [u16; Self::CHANNEL_COUNT],
    pub flags: u8,
    pub rssi: u8,
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
    pub const CHANNEL_COUNT: usize = 16;
    /// Deliberately no not support AUX13.
    const _AUX13: u8 = 0x01;
    /// Deliberately no not support AUX14.
    const _AUX14: u8 = 0x02;
    const FRAME_LOST: u8 = 0x04;
    const FAILSAFE: u8 = 0x08;

    /// Constructor.
    pub const fn new() -> Self {
        Self { channels: [0u16; Self::CHANNEL_COUNT], flags: 0, rssi: 0 }
    }
}

impl SbusFrame {
    /// SBUS values typically range from 172 to 1811 (representing 1000µs to 2000µs),
    /// so they need to be normalized to the standard PWM range `[1000,2000]`.
    #[allow(unused)]
    #[allow(clippy::cast_possible_truncation)]
    pub fn normalize_channels(input: &[u16; Self::CHANNEL_COUNT]) -> [u16; Self::CHANNEL_COUNT] {
        const PWM_LOW: u32 = 172;
        const PWM_HIGH: u32 = 1811;
        const PWM_RANGE: u32 = PWM_HIGH - PWM_LOW;

        let mut output = [0u16; Self::CHANNEL_COUNT];

        for (in_val, out_val) in input.iter().zip(output.iter_mut()) {
            let val = u32::from(*in_val).clamp(PWM_LOW, PWM_HIGH);
            *out_val = ((val - PWM_LOW) * u32::from(RxChannel::RANGE) / (PWM_RANGE) + u32::from(RxChannel::LOW)) as u16;
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
