use super::{CrsfRadio, IbusRadio, MockRadio, RadioType, RxFrame};

/// 48-bit extended unique identifier (often synonymous with MAC address).<br><br>
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Eui48 {
    pub octets: [u8; 6],
}

impl Eui48 {
    /// Constructor.
    #[must_use]
    pub const fn new() -> Self {
        Self { octets: [0u8; 6] }
    }
}

/// Properties common to all RX radios.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RxRadioCommon {
    pub packet_received: bool, // may be invalid packet
    pub new_packet_available: bool,
    pub positive_half_throttle: bool,
    pub packet_count: i32,
    pub dropped_packet_count_delta: i32,
    pub dropped_packet_count: i32,
    pub dropped_packet_count_previous: i32,
    pub tick_count_delta: i32,
}

impl Default for RxRadioCommon {
    fn default() -> Self {
        Self::new()
    }
}

impl RxRadioCommon {
    // standardize radios to use AETR (Ailerons, Elevator, Throttle, Rudder), ie ROLL, PITCH, THROTTLE, YAW
    /// Constructor.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            packet_received: false,
            new_packet_available: false,
            positive_half_throttle: false,
            packet_count: 0,
            dropped_packet_count_delta: 0,
            dropped_packet_count: 0,
            dropped_packet_count_previous: 0,
            tick_count_delta: 0,
        }
    }
}

/// The common interface for all RC radios.
/// Note: this is not called (say) `RxReceiver` to avoid possible confusion with Embassy `Watch` `Receiver`.
pub trait RxRadio {
    fn rx_frame(&self) -> RxFrame;
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Radio {
    Mock(MockRadio),
    Crsf(CrsfRadio),
    Ibus(IbusRadio),
}

impl Radio {
    #[must_use]
    pub const fn new(radio_type: RadioType) -> Radio {
        match radio_type {
            RadioType::Mock => Self::Mock(MockRadio::new()),
            RadioType::Crsf => Self::Crsf(CrsfRadio::new()),
            RadioType::Ibus => Self::Ibus(IbusRadio::new()),
        }
    }
}
impl RxRadio for Radio {
    fn rx_frame(&self) -> RxFrame {
        match self {
            Self::Mock(radio) => radio.rx_frame(),
            Self::Crsf(radio) => radio.rx_frame(),
            Self::Ibus(radio) => radio.rx_frame(),
        }
    }
}

#[cfg(test)]
mod test_traits {
    use super::*;

    fn _is_normal<T: Sized + Send + Sync + Unpin>() {}
    fn is_full<T: Sized + Send + Sync + Unpin + Copy + Clone + Default + PartialEq>() {}

    #[test]
    fn normal_types() {
        is_full::<Eui48>();
        is_full::<RxRadioCommon>();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let radio = RxRadioCommon::new();
        assert!(!radio.packet_received);
    }
}
