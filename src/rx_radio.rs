use crate::{CrsfRadio, IbusRadio, MockRadio, RadioType};

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

/// Status of radio link.
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum RxLinkStatus {
    #[default]
    Ok,
    Failsafe,
    NoSignal,
}

impl RxLinkStatus {
    /// Constructor.
    #[must_use]
    pub const fn new() -> Self {
        Self::Ok
    }
}

/// RX channel constants.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RxChannel {}

#[allow(missing_docs)]
impl RxChannel {
    // AETR (ailerons, elevators, throttle, rudder) ordering.
    pub const ROLL: usize = 0;
    pub const PITCH: usize = 1;
    pub const THROTTLE: usize = 2;
    pub const YAW: usize = 3;
    pub const AUX1: usize = 4;
    pub const AUX2: usize = 5;
    pub const AUX3: usize = 6;
    pub const AUX4: usize = 7;
    pub const AUX5: usize = 8;
    pub const AUX6: usize = 9;
    pub const AUX7: usize = 10;
    pub const AUX8: usize = 11;
    pub const AUX9: usize = 12;
    pub const AUX10: usize = 13;
    pub const AUX11: usize = 14;
    pub const AUX12: usize = 15;
    pub const AUX13: usize = 16;
    pub const AUX14: usize = 17;
    pub const AUX15: usize = 18;
    pub const AUX16: usize = 19;

    pub const ROLL_U8: u8 = 0;
    pub const PITCH_U8: u8 = 1;
    pub const THROTTLE_U8: u8 = 2;
    pub const YAW_U8: u8 = 3;
    pub const AUX1_U8: u8 = 4;
    pub const AUX2_U8: u8 = 5;
    pub const AUX3_U8: u8 = 6;
    pub const AUX4_U8: u8 = 7;
    pub const AUX5_U8: u8 = 8;
    pub const AUX6_U8: u8 = 9;
    pub const AUX7_U8: u8 = 10;
    pub const AUX8_U8: u8 = 11;
    pub const AUX9_U8: u8 = 12;
    pub const AUX10_U8: u8 = 13;
    pub const AUX11_U8: u8 = 14;
    pub const AUX12_U8: u8 = 15;
    pub const AUX13_U8: u8 = 16;
    pub const AUX14_U8: u8 = 17;
    pub const AUX15_U8: u8 = 18;
    pub const AUX16_U8: u8 = 19;

    // PWM values
    // Normal range is [1000, 2000]
    pub const MIN: u16 = 900;
    pub const LOW: u16 = 1000;
    pub const MID_LOW: u16 = 1250;
    pub const MID: u16 = 1500;
    pub const MID_HIGH: u16 = 1750;
    pub const HIGH: u16 = 2000;
    pub const MAX: u16 = 2100;
    pub const RANGE: u16 = Self::HIGH - Self::LOW;

    pub const MIN_F32: f32 = 900.0;
    pub const LOW_F32: f32 = 1000.0;
    pub const MID_LOW_F32: f32 = 1250.0;
    pub const MID_F32: f32 = 1500.0;
    pub const MID_HIGH_F32: f32 = 1750.0;
    pub const HIGH_F32: f32 = 2000.0;
    pub const MAX_F32: f32 = 2100.0;
    pub const RANGE_F32: f32 = 1000.0;
    pub const HALF_RANGE_F32: f32 = 500.0;
}

impl RxChannel {
    /// Maps [1000, 2000] to [-1000, 1000].
    #[must_use]
    pub fn map_rpy_pwm_to_plus_minus_1000(pwm: u16) -> i32 {
        i32::from(pwm) * 2 - 3000
    }
    /// Maps [1000, 2000] to [0, 1000].
    #[must_use]
    pub fn map_throttle_pwm_to_0_to_1000(pwm: u16) -> i32 {
        i32::from(pwm) - 1000
    }
}

/// Array of RX channels.
pub(crate) type RxChannels = [u16; RxFrame::MAX_CHANNEL_COUNT];

/// Receiver frame containing array of rx channel values, link status and RSSI.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RxFrame {
    /// The channels in PWM range, nominally `[1000,2000]`.
    pub channels: [u16; RxFrame::MAX_CHANNEL_COUNT],
    pub status: RxLinkStatus,
    pub rssi: u8,
}

impl Default for RxFrame {
    fn default() -> Self {
        Self::new()
    }
}

impl RxFrame {
    // SBUS has 18 channels (the last two are digital channels with the two values 1000 or 2000), but we only use 16.
    // IBUS has 14 channels
    // CRSF has 16 channels
    pub const MAX_CHANNEL_COUNT: usize = 16;
    pub const DEFAULT_CHANNEL_VALUE: u16 = RxChannel::LOW;

    /// Constructor.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            channels: [
                RxChannel::MID, // Sticks default to MID.
                RxChannel::MID,
                RxChannel::LOW, // Throttle defaults to LOW.
                RxChannel::MID,
                RxChannel::LOW,
                RxChannel::LOW,
                RxChannel::LOW,
                RxChannel::LOW,
                RxChannel::LOW,
                RxChannel::LOW,
                RxChannel::LOW,
                RxChannel::LOW,
                RxChannel::LOW,
                RxChannel::LOW,
                RxChannel::LOW,
                RxChannel::LOW,
            ],
            status: RxLinkStatus::new(),
            rssi: 0,
        }
    }
}

impl RxFrame {
    /// Returns true if the frame is safe to use for flight control.
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.status == RxLinkStatus::Ok
    }
    /// Returns value of auxiliary channel, or `RxChannel::LOW` if channel index invalid.
    #[must_use]
    pub fn channel(&self, channel_index: u8) -> u16 {
        let index = usize::from(channel_index);
        if index < Self::MAX_CHANNEL_COUNT {
            return self.channels[channel_index as usize];
        }
        RxChannel::LOW
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
    #[test]
    fn map_rpy_pwm_to_plus_minus_1000() {
        assert_eq!(-1000, RxChannel::map_rpy_pwm_to_plus_minus_1000(1000));
        assert_eq!(-500, RxChannel::map_rpy_pwm_to_plus_minus_1000(1250));
        assert_eq!(0, RxChannel::map_rpy_pwm_to_plus_minus_1000(1500));
        assert_eq!(500, RxChannel::map_rpy_pwm_to_plus_minus_1000(1750));
        assert_eq!(1000, RxChannel::map_rpy_pwm_to_plus_minus_1000(2000));
    }
    #[test]
    fn map_throttle_pwm_to_0_to_1000() {
        assert_eq!(0, RxChannel::map_throttle_pwm_to_0_to_1000(1000));
        assert_eq!(250, RxChannel::map_throttle_pwm_to_0_to_1000(1250));
        assert_eq!(500, RxChannel::map_throttle_pwm_to_0_to_1000(1500));
        assert_eq!(750, RxChannel::map_throttle_pwm_to_0_to_1000(1750));
        assert_eq!(1000, RxChannel::map_throttle_pwm_to_0_to_1000(2000));
    }
}
