use crate::{RxFrame, RxRadio};

/// Mock radio<br><br>
#[allow(unused)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MockRadio {}

impl Default for MockRadio {
    fn default() -> Self {
        Self::new()
    }
}

impl MockRadio {
    /// Constructor.
    #[must_use]
    pub const fn new() -> Self {
        Self {}
    }
}

impl RxRadio for MockRadio {
    fn rx_frame(&self) -> RxFrame {
        RxFrame::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn _is_normal<T: Sized + Send + Sync + Unpin>() {}
    fn is_full<T: Sized + Send + Sync + Unpin + Copy + Clone + Default + PartialEq>() {}

    #[test]
    fn normal_types() {
        is_full::<MockRadio>();
    }
    #[test]
    fn new() {
        let _radio = MockRadio::new();
        //assert!(radio.is_data_available());
    }
}
