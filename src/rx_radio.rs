use super::{CrsfRadio, IbusRadio, RadioType, RxFrame, SbusRadio};

/// The common interface for all RC radios.
/// Note: this is not called (say) `RxReceiver` to avoid possible confusion with Embassy `Watch` `Receiver`.
pub trait RxRadio {
    fn on_byte_received(&mut self, byte: u8) -> bool;
    fn rx_frame(&self) -> RxFrame;
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Radio {
    Crsf(CrsfRadio),
    Ibus(IbusRadio),
    Sbus(SbusRadio),
}

impl Radio {
    #[must_use]
    pub const fn new(radio_type: RadioType) -> Radio {
        match radio_type {
            RadioType::Crsf => Self::Crsf(CrsfRadio::new()),
            RadioType::Ibus => Self::Ibus(IbusRadio::new()),
            RadioType::Sbus => Self::Sbus(SbusRadio::new()),
        }
    }
}

impl RxRadio for Radio {
    fn on_byte_received(&mut self, byte: u8) -> bool {
        match self {
            Self::Crsf(radio) => radio.on_byte_received(byte),
            Self::Ibus(radio) => radio.on_byte_received(byte),
            Self::Sbus(radio) => radio.on_byte_received(byte),
        }
    }
    fn rx_frame(&self) -> RxFrame {
        match self {
            Self::Crsf(radio) => radio.rx_frame(),
            Self::Ibus(radio) => radio.rx_frame(),
            Self::Sbus(radio) => radio.rx_frame(),
        }
    }
}

#[cfg(test)]
mod test_traits {
    use super::*;

    fn is_full_no_default<T: Sized + Send + Sync + Unpin + Copy + Clone + PartialEq>() {}

    #[test]
    fn normal_types() {
        is_full_no_default::<Radio>();
    }
}
