use super::Payload22;
use crate::{RxChannels, RxLinkStatus};

#[derive(Clone, Copy, Debug, Default, PartialEq)]
enum State {
    #[default]
    WaitForHeaderByte,
    ReadPayload {
        index: usize,
    },
    ValidateFooter,
}

impl State {
    const HEADER_BYTE: u8 = 0x0F;
    const FOOTER_BYTE: u8 = 0x00;
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
    /// SBUS supports 18 channels, but we only use 16 of them.
    pub const PACKET_LENGTH: usize = 25;
    pub const HEADER_LENGTH: usize = 1;
    pub const PAYLOAD_LENGTH: usize = 22;
    pub const FLAGS_BYTE: usize = 23;
    const FRAME_LOST: u8 = 0x04;
    const FAILSAFE: u8 = 0x08;

    #[must_use]
    pub const fn new() -> Self {
        Self { state: State::WaitForHeaderByte, buffer: [0u8; Self::PACKET_LENGTH] }
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
    pub fn on_byte_received(&mut self, byte: u8) -> Option<(RxChannels, RxLinkStatus)> {
        let mut complete = false;

        self.state = match core::mem::take(&mut self.state) {
            State::WaitForHeaderByte => {
                if byte == State::HEADER_BYTE {
                    // We have a valid header byte, so start collecting the payload.
                    self.buffer[0] = byte;
                    State::ReadPayload { index: 1 }
                } else {
                    State::WaitForHeaderByte
                }
            }
            // Collect the 22 bytes of payload.
            State::ReadPayload { index } => {
                self.buffer[index] = byte;
                let index = index + 1;

                // When we have collected the payload, move onto the footer.
                if index > Self::HEADER_LENGTH + Self::PAYLOAD_LENGTH {
                    State::ValidateFooter
                } else {
                    State::ReadPayload { index }
                }
            }
            State::ValidateFooter => {
                if byte == State::FOOTER_BYTE {
                    complete = true;
                }
                State::WaitForHeaderByte
            }
        };

        if complete && let Ok(channel_data) = self.buffer[1..23].try_into() {
            let channels = Payload22::parse_payload(&channel_data);
            let channels = RxChannels::from_channels(channels);

            let flags = self.buffer[Self::FLAGS_BYTE];
            // Check FAILSAFE flag first, since this indicates multiple lost frames
            let link_status = if flags & Self::FAILSAFE != 0 {
                RxLinkStatus::Failsafe
            } else if flags & Self::FRAME_LOST != 0 {
                RxLinkStatus::NoSignal
            } else {
                RxLinkStatus::Ok
            };
            Some((channels, link_status))
        } else {
            None
        }
    }

    pub fn parse_packet(&mut self, buffer: &[u8; Self::PACKET_LENGTH]) -> Option<(RxChannels, RxLinkStatus)> {
        for byte in buffer {
            if let Some(result) = self.on_byte_received(*byte) {
                return Some(result);
            }
        }
        None
    }
}

#[cfg(test)]
mod test_traits {
    use super::*;

    fn is_full<T: Sized + Send + Sync + Unpin + Copy + Clone + Default + PartialEq>() {}

    #[test]
    fn normal_types() {
        is_full::<State>();
        is_full::<SbusDecoder>();
    }
}
