use crate::protocols::crsf::CrsfParser;
use crate::protocols::receiver_serial::ReceiverSerial;
use crate::{RxChannel, RxFrame, RxLinkStatus, RxReceiver, RxReceiverCommon};

/*pub struct CrsfReceiverXXXX<UART> {
    //shared: SerialReceiver<UART>,
    // CRSF specific data
}*/

/// Crossfire receiver<br><br>
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CrsfReceiver {
    common: RxReceiverCommon,
    serial: ReceiverSerial,
    channels: [u16; CrsfFrame::CHANNEL_COUNT],
    packet_size: usize,
    packet_type: u8,
    // packet is composed as follows
    // byte 0: sync;
    // byte 1: length; // length is length of type, payload, and CRC
    // byte 2: type;
    // 22 bytes of payload 176 bits of data (11 bits per channel * 16 channels) = 22 bytes.
    // CRC byte after payload. CRC is calculated on all bytes from type to end of payload
    packet_isr: [u8; Self::MAX_PACKET_SIZE], // packet date written hear in ISR
    packet: [u8; Self::MAX_PACKET_SIZE],     // copy of packet used outside of ISR
}

impl Default for CrsfReceiver {
    fn default() -> Self {
        Self::new()
    }
}

impl CrsfReceiver {
    // 8N1
    const _DATA_BITS: u8 = 8;
    //const PARITY:u8 = SerialPort::PARITY_NONE;
    const _STOP_BITS: u8 = 1;
    const _BAUD_RATE: u32 = 416_666;
    const _BAUD_RATE_UNOFFICIAL: u32 = 420_000;

    const TIME_NEEDED_PER_FRAME_US: u32 = 1750;

    const CRSF_SYNC_BYTE: u8 = 0xC8;
    const EDGE_TX_SYNC_BYTE: u8 = 0xEE;

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
    const MAX_PACKET_SIZE: usize = CrsfParser::MAX_PACKET_SIZE;

    pub fn new() -> Self {
        Self {
            common: RxReceiverCommon::default(),
            serial: ReceiverSerial::default(),
            channels: <[u16; CrsfFrame::CHANNEL_COUNT]>::default(),
            packet_size: 0,
            packet_type: 0,
            packet_isr: [0u8; Self::MAX_PACKET_SIZE],
            packet: [0u8; Self::MAX_PACKET_SIZE],
        }
    }
}

pub struct CrsfFrame {
    pub channels: [u16; Self::CHANNEL_COUNT],
    pub failsafe: bool,
    pub frame_lost: bool,
    pub rssi: u8,
}

impl CrsfFrame {
    const CHANNEL_COUNT: usize = 16;
}

impl From<CrsfFrame> for RxFrame {
    fn from(frame: CrsfFrame) -> Self {
        let status = if frame.failsafe {
            RxLinkStatus::Failsafe
        } else if frame.frame_lost {
            RxLinkStatus::NoSignal
        } else {
            RxLinkStatus::Ok
        };

        let mut channels = [Self::DEFAULT_CHANNEL_VALUE; Self::MAX_CHANNEL_COUNT];
        channels[..frame.channels.len()].copy_from_slice(&frame.channels);

        Self { channels, status, rssi: frame.rssi }
    }
}

impl RxReceiver for CrsfReceiver {
    type Frame = CrsfFrame;

    fn is_data_available(&self) -> bool {
        false
    }
    fn read_byte(&mut self) -> u8 {
        0
    }
    //fn update(&mut self) -> Result<Option<Self::Frame>, Error> {}

    #[allow(clippy::cast_possible_truncation)]
    #[allow(clippy::cast_sign_loss)]
    fn channel_pwm(&self, channel_index: u8) -> u16 {
        // conversion from RC value to PWM
        // for FRAMETYPE_RC_CHANNELS_PACKED(0x16)
        //       RC     PWM
        // min   172 ->  988us
        // mid   992 -> 1500us
        // max  1811 -> 2012us
        // scale factor = (2012-988) / (1811-172) = 0.62477120195241
        // offset = 988 - 172 * 0.62477120195241 = 880.53935326418548
        const CHANNEL_SCALE: f32 = 0.624_771_2;
        const CHANNEL_OFFSET: f32 = 880.539_36;

        if channel_index as usize >= CrsfFrame::CHANNEL_COUNT {
            return RxChannel::LOW;
        }
        let pwm = CHANNEL_SCALE * f32::from(self.channels[channel_index as usize]) + CHANNEL_OFFSET;
        pwm as u16
    }

    fn on_data_received_from_isr(&mut self, data: u8) -> bool {
        let time_now_us: u32 = 0; //time_us();
        if time_now_us > self.serial.start_time + Self::TIME_NEEDED_PER_FRAME_US {
            self.serial.packet_index = 0;
            self.common.dropped_packet_count += 1;
        }

        match self.serial.packet_index {
            0 => {
                if data != CrsfReceiver::CRSF_SYNC_BYTE && data != CrsfReceiver::EDGE_TX_SYNC_BYTE {
                    self.serial.packet_is_empty = true;
                    return false;
                }
                self.serial.start_time = time_now_us;
            }
            1 => {
                self.packet_size = data as usize + 2;
            }
            2 => {
                self.packet_type = data;
            }
            _ => {}
        }

        self.packet_isr[self.serial.packet_index] = data;
        self.serial.packet_index += 1;

        if self.packet_size != 0 && self.serial.packet_index == self.packet_size {
            self.serial.packet_index = 0;
            self.packet_size = 0;
            // copy packet_isr into packet, so packet_isr is available for the next interrupt
            // TODO: make this atomic
            self.packet = self.packet_isr;
            return true;
        }

        false
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
        is_full::<CrsfReceiver>();
    }
    #[test]
    fn new() {
        let _receiver = CrsfReceiver::new();
        //assert!(receiver.is_data_available());
    }
}
