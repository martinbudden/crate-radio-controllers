use super::{RxProtocol,CrsfFrame};
use crate::{CrsfRadio, RxChannel};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CrsfRadio {
    pub(crate) serial: RadioSerial,
    decoder: CrsfDecoder,
    pub(crate) frame: CrsfFrame,
    pub(crate) packet_size: usize,
    pub(crate) packet_type: u8,
    // packet is composed as follows
    // byte 0: sync;
    // byte 1: length; // length is length of type, payload, and CRC
    // byte 2: type;
    // 22 bytes of payload 176 bits of data (11 bits per channel * 16 channels) = 22 bytes.
    // CRC byte after payload. CRC is calculated on all bytes from type to end of payload
    pub(crate) packet_isr: [u8; Self::MAX_PACKET_SIZE], // packet date written here in ISR
    pub(crate) packet: [u8; Self::MAX_PACKET_SIZE],     // copy of packet used outside of ISR
}

impl RxProtocol for CrsfRadio {
    fn is_data_available(&self) -> bool {
        false
    }

    fn read_byte(&mut self) -> u8 {
        0
    }
    //fn update(&mut self) -> Result<Option<Self::Frame>, Error> {}

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
        let pwm = CHANNEL_SCALE * f32::from(self.frame.channels[channel_index as usize]) + CHANNEL_OFFSET;
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        {
            pwm as u16
        }
    }

    fn on_data_received_from_isr(&mut self, data: u8) -> bool {
        let time_now_us: u32 = 0; //time_us();
        if time_now_us > self.serial.start_time + CrsfRadio::TIME_NEEDED_PER_FRAME_US {
            self.serial.packet_index = 0;
        }

        match self.serial.packet_index {
            0 => {
                if data != CrsfRadio::CRSF_SYNC_BYTE && data != CrsfRadio::EDGE_TX_SYNC_BYTE {
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
