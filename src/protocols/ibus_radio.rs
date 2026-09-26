use super::IbusDecoder;
use crate::{RxFrame, RxRadio};

/// Ibus radio<br><br>
#[allow(unused)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct IbusRadio {
    decoder: IbusDecoder,
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
        Self { decoder: IbusDecoder::new() }
    }
}

impl RxRadio for IbusRadio {
    fn on_byte_received(&mut self, byte: u8) -> Option<RxFrame> {
        self.decoder.on_byte_received(byte)
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
