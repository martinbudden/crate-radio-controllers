use super::CrsfDecoder;
use crate::{RxFrame, RxRadio};

/*pub struct CrsfReceiverXXXX<UART> {
    //shared: SerialReceiver<UART>,
    // CRSF specific data
}*/

/// Crossfire radio<br><br>
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CrsfRadio {
    decoder: CrsfDecoder,
}

impl Default for CrsfRadio {
    fn default() -> Self {
        Self::new()
    }
}

#[allow(missing_docs)]
impl CrsfRadio {
    // 8N1
    const _DATA_BITS: u8 = 8;
    //const PARITY:u8 = SerialPort::PARITY_NONE;
    const _STOP_BITS: u8 = 1;
    const _BAUD_RATE: u32 = 416_666;
    const _BAUD_RATE_UNOFFICIAL: u32 = 420_000;

    pub const TIME_NEEDED_PER_FRAME_US: u32 = 1750;

    pub const CRSF_SYNC_BYTE: u8 = 0xC8;
    pub const EDGE_TX_SYNC_BYTE: u8 = 0xEE;

    const _ADDRESS_BROADCAST: u8 = 0x00;
    const _ADDRESS_USB: u8 = 0x10;
    const _ADDRESS_TBS_CORE_PNP_PRO: u8 = 0x80;
    const _ADDRESS_RESERVED1: u8 = 0x8A;
    const _ADDRESS_CURRENT_SENSOR: u8 = 0xC0;
    const _ADDRESS_GPS: u8 = 0xC2;
    const _ADDRESS_TBS_BLACKBOX: u8 = 0xC4;
    const _ADDRESS_FLIGHT_CONTROLLER: u8 = 0xC8;
    const _ADDRESS_RESERVED2: u8 = 0xCA;
    const _ADDRESS_RACE_TAG: u8 = 0xCC;
    const _ADDRESS_RADIO_TRANSMITTER: u8 = 0xEA;
    const _ADDRESS_CRSF_RECEIVER: u8 = 0xEC;
    const _ADDRESS_CRSF_TRANSMITTER: u8 = 0xEE;

    const _COMMAND_SUBCMD_RX_BIND: u8 = 0x01;
    const _COMMAND_SUBCMD_RX: u8 = 0x10;
    const _COMMAND_SUBCMD_GENERAL: u8 = 0x0A;
    const _COMMAND_SUBCMD_GENERAL_CRSF_SPEED_PROPOSAL: u8 = 0x70;
    const _COMMAND_SUBCMD_GENERAL_CRSF_SPEED_RESPONSE: u8 = 0x71;

    /// Constructor.
    #[must_use]
    pub const fn new() -> Self {
        Self { decoder: CrsfDecoder::new() }
    }
}

impl RxRadio for CrsfRadio {
    fn on_byte_received(&mut self, byte: u8) -> Option<RxFrame> {
        self.decoder.on_byte_received(byte)
    }
}

#[cfg(test)]
mod test_traits {
    use super::*;

    fn is_full<T: Sized + Send + Sync + Unpin + Copy + Clone + Default + PartialEq>() {}

    #[test]
    fn normal_types() {
        is_full::<CrsfRadio>();
    }
}
