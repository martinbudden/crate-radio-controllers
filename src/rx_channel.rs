use core::ops::{Index, IndexMut, Range};

#[cfg(feature = "storage")]
use sequential_storage::map::PostcardValue;
#[cfg(feature = "serde")]
use {
    postcard::experimental::max_size::MaxSize,
    serde::{Deserialize, Serialize},
};

/// RX channel constants.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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

/// PWM channels are divided into "steps". Steps are 25 units wide<br>
/// There are 48 steps between 900 and 2100.<br>
///     a step value of 0 corresponds to a channel value of 900 or less.<br>
///     a step value of 48 corresponds to a channel value of 2100 or more.<br>
///
/// Steps are used to convert channel values into "switches"
/// So for example if the `CHANNEL_AUX1` is > 1500 that might correspond to the motors being "armed"
/// while a value < 1500 might correspond to the motors being "disarmed".
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize, MaxSize))]
pub struct RxChannelRange {
    pub start: u8,
    pub end: u8,
}

#[cfg(feature = "storage")]
impl PostcardValue<'_> for RxChannelRange {}

impl Default for RxChannelRange {
    fn default() -> Self {
        Self::new()
    }
}

impl RxChannelRange {
    /// Constructor.
    #[must_use]
    pub const fn new() -> Self {
        Self { start: 0, end: 0 }
    }
    /// Set the start of a newly constructed range.
    #[must_use]
    pub const fn with_start(mut self, start: u8) -> Self {
        self.start = start;
        self
    }
    /// Set the end of a newly constructed range.
    #[must_use]
    pub const fn with_end(mut self, end: u8) -> Self {
        self.end = end;
        self
    }

    /// Construct from PWM values.
    #[must_use]
    pub fn from_pwm(pwm_start: u16, pwm_end: u16) -> Self {
        Self { start: Self::pwm_to_step(pwm_start), end: Self::pwm_to_step(pwm_end.max(pwm_start)) }
    }
}

impl RxChannelRange {
    pub const MIN: u16 = 900;
    pub const MID: u16 = 1500;
    pub const MAX: u16 = 2100;

    pub const STEP: u16 = 25;
    pub const STEP_MIN: u16 = 0;
    pub const STEP_MID: u16 = ((Self::MID - Self::MIN) / Self::STEP);
    pub const STEP_MAX: u16 = ((Self::MAX - Self::MIN) / Self::STEP);

    #[inline]
    #[must_use]
    pub fn step_to_pwm(step: u8) -> u16 {
        Self::MIN + Self::STEP * u16::from(step)
    }

    #[allow(clippy::cast_possible_truncation)]
    #[inline]
    #[must_use]
    pub fn pwm_to_step(pwm: u16) -> u8 {
        ((pwm.clamp(Self::MIN, Self::MAX) - Self::MIN) / Self::STEP) as u8
    }

    #[inline]
    pub fn set(&mut self, pwm_start: u16, pwm_end: u16) {
        if pwm_end > pwm_start {
            self.start = Self::pwm_to_step(pwm_start);
            self.end = Self::pwm_to_step(pwm_end);
        }
    }

    #[inline]
    #[must_use]
    pub fn pwm_range(&self) -> (u16, u16) {
        (Self::step_to_pwm(self.start), Self::step_to_pwm(self.end))
    }

    #[inline]
    #[must_use]
    pub fn is_range_active(channel_value: u16, start: u8, end: u8) -> bool {
        if channel_value >= Self::MIN + u16::from(start) * Self::STEP
            && channel_value < Self::MIN + u16::from(end) * Self::STEP
        {
            return true;
        }
        false
    }

    #[must_use]
    #[inline]
    pub fn is_active(&self, rx_channels: &RxChannels, aux_channel_index: u8) -> bool {
        let index = usize::from(aux_channel_index);
        let channel_value = if index < RxChannels::CHANNEL_COUNT { rx_channels[index] } else { RxChannel::LOW };

        Self::is_range_active(channel_value, self.start, self.end)
    }
    /*#[must_use]
    #[inline]
    pub fn is_active(&self, rx_channels: &RxChannels, aux_channel_index: u8) -> bool {
        let channel_value: u16 = rx_channels[aux_channel_index as usize];
        Self::is_range_active(channel_value, self.start, self.end)
    }*/
}
/// Array of RX channels.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RxChannels([u16; Self::CHANNEL_COUNT]);

impl RxChannels {
    pub const CHANNEL_COUNT: usize = 16;

    pub const FAILSAFE_CHANNEL_VALUES: [u16; Self::CHANNEL_COUNT] = [
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
    ];

    /// Constructor.
    #[must_use]
    pub const fn new() -> Self {
        Self(RxChannels::FAILSAFE_CHANNEL_VALUES)
    }
    #[must_use]
    pub const fn from_channels(channels: [u16; Self::CHANNEL_COUNT]) -> Self {
        Self(channels)
    }
}

impl Default for RxChannels {
    fn default() -> Self {
        Self::new()
    }
}

impl Index<usize> for RxChannels {
    type Output = u16;

    /// Access channel by index.
    #[inline]
    fn index(&self, index: usize) -> &u16 {
        &self.0[index]
    }
}

impl IndexMut<usize> for RxChannels {
    /// Set channel by index.
    #[inline]
    fn index_mut(&mut self, index: usize) -> &mut u16 {
        &mut self.0[index]
    }
}

impl Index<Range<usize>> for RxChannels {
    type Output = [u16];

    #[inline]
    fn index(&self, index: Range<usize>) -> &[u16] {
        &self.0[index]
    }
}

impl IndexMut<Range<usize>> for RxChannels {
    #[inline]
    fn index_mut(&mut self, index: Range<usize>) -> &mut [u16] {
        &mut self.0[index]
    }
}

impl RxChannels {
    /// Returns value of channel, or `RxChannel::LOW` if channel index invalid.
    #[must_use]
    pub fn channel(&self, channel_index: u8) -> u16 {
        let index = usize::from(channel_index);
        if index < Self::CHANNEL_COUNT {
            return self.0[channel_index as usize];
        }
        RxChannel::LOW
    }
    #[must_use]
    pub fn channels(&self) -> [u16; Self::CHANNEL_COUNT] {
        self.0
    }
    pub fn set_channels_to_failsafe_values(&mut self) {
        self.0 = Self::FAILSAFE_CHANNEL_VALUES;
    }
}

#[cfg(test)]
mod test_traits {
    use super::*;

    fn is_full<T: Sized + Send + Sync + Unpin + Copy + Clone + Default + PartialEq>() {}
    fn is_full_no_default<T: Sized + Send + Sync + Unpin + Copy + Clone + PartialEq>() {}
    #[cfg(feature = "serde")]
    fn is_serde<T: Serialize + MaxSize + for<'a> Deserialize<'a>>() {}
    #[cfg(feature = "storage")]
    fn is_storage<T: for<'a> PostcardValue<'a>>() {}

    #[test]
    fn normal_types() {
        is_full_no_default::<RxChannel>();
        is_full::<RxChannelRange>();
        is_full::<RxChannels>();
    }
    #[cfg(feature = "serde")]
    #[test]
    fn serde_types() {
        is_serde::<RxChannelRange>();
    }
    #[cfg(feature = "storage")]
    #[test]
    fn storage_types() {
        is_storage::<RxChannelRange>();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
