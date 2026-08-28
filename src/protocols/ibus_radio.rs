use crate::{
    RxFrame, RxRadio, RxRadioCommon,
    protocols::{RxProtocol, ibus::IbusFrame, serial_radio::RadioSerial},
};

/// Ibus radio<br><br>
#[allow(unused)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct IbusRadio {
    common: RxRadioCommon,
    serial: RadioSerial,
    frame: IbusFrame,
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
        Self { common: RxRadioCommon::new(), serial: RadioSerial::new(), frame: IbusFrame::new() }
    }
}

impl RxRadio for IbusRadio {
    fn rx_frame(&self) -> RxFrame {
        RxFrame::default()
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
