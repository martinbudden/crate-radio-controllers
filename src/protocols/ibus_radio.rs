use super::{IbusDecoder, IbusFrame, RadioSerial, RxProtocol};
use crate::{RxFrame, RxRadio};

/// Ibus radio<br><br>
#[allow(unused)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct IbusRadio {
    serial: RadioSerial,
    frame: IbusFrame,
    decoder: IbusDecoder,
    rx_frame: RxFrame,
}

impl Default for IbusRadio {
    fn default() -> Self {
        Self::new()
    }
}

impl IbusRadio {
    /// Constructor.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            serial: RadioSerial::new(),
            frame: IbusFrame::new(),
            decoder: IbusDecoder::new(),
            rx_frame: RxFrame::new(),
        }
    }
}

impl RxRadio for IbusRadio {
    fn rx_frame(&self) -> RxFrame {
        self.rx_frame
    }
    fn on_byte_received(&mut self, byte: u8) -> bool {
        let result = self.decoder.on_byte_received(byte);
        if let Some(ibus_frame) = result {
            self.rx_frame = RxFrame::from(ibus_frame);
            true
        } else {
            false
        }
    }
}

impl RxProtocol for IbusRadio {
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
        is_full::<IbusRadio>();
    }
    #[test]
    fn new() {
        let _radio = IbusRadio::new();
        //assert!(radio.is_data_available());
    }
}
