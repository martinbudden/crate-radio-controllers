use super::{RadioSerial, RxProtocol, SbusDecoder, SbusFrame};
use crate::{RxFrame, RxRadio, RxRadioCommon};

/// Ibus radio<br><br>
#[allow(unused)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SbusRadio {
    common: RxRadioCommon,
    serial: RadioSerial,
    frame: SbusFrame,
    decoder: SbusDecoder,
    rx_frame: RxFrame,
}

impl Default for SbusRadio {
    fn default() -> Self {
        Self::new()
    }
}

impl SbusRadio {
    /// Constructor.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            common: RxRadioCommon::new(),
            serial: RadioSerial::new(),
            frame: SbusFrame::new(),
            decoder: SbusDecoder::new(),
            rx_frame: RxFrame::new(),
        }
    }
}

impl RxRadio for SbusRadio {
    fn rx_frame(&self) -> RxFrame {
        self.rx_frame
    }
    fn on_byte_received(&mut self, byte: u8) -> bool {
        let result = self.decoder.on_byte_received(byte);
        if let Some(sbus_frame) = result {
            self.rx_frame = RxFrame::from(sbus_frame);
            true
        } else {
            false
        }
    }
}

impl RxProtocol for SbusRadio {
    fn is_data_available(&self) -> bool {
        false
    }

    fn read_byte(&mut self) -> u8 {
        0
    }
    //fn update(&mut self) -> Result<Option<Self::Frame>, Error> {}

    fn channel_pwm(&self, _channel_index: u8) -> u16 {
        0
    }

    fn on_data_received_from_isr(&mut self, _data: u8) -> bool {
        _ = self;
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn _is_normal<T: Sized + Send + Sync + Unpin>() {}
    fn is_full<T: Sized + Send + Sync + Unpin + Copy + Clone + Default + PartialEq>() {}

    #[test]
    fn normal_types() {
        is_full::<SbusRadio>();
    }
    #[test]
    fn new() {
        let _radio = SbusRadio::new();
        //assert!(radio.is_data_available());
    }
}
