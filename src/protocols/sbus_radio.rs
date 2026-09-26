use super::SbusDecoder;
use crate::{RxFrame, RxRadio};

/// Ibus radio<br><br>
#[allow(unused)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SbusRadio {
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
        Self { decoder: SbusDecoder::new(), rx_frame: RxFrame::new() }
    }
}

impl RxRadio for SbusRadio {
    fn rx_frame(&self) -> RxFrame {
        self.rx_frame
    }
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
        is_full::<SbusRadio>();
    }
    #[test]
    fn new() {
        let _radio = SbusRadio::new();
        //assert!(radio.is_data_available());
    }
}
