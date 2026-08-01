use crate::{RcMode, RxChannel, RxFrame};
use simple_bitset::BitSet64;

#[cfg(feature = "serde")]
use {
    sequential_storage::map::PostcardValue,
    serde::{Deserialize, Serialize},
};

/// PWM channels are divided into "steps". Steps are 25 units wide<br>
/// There are 48 steps between 900 and 2100.<br>
///     a step value of 0 corresponds to a channel value of 900 or less.<br>
///     a step value of 48 corresponds to a channel value of 2100 or more.<br>
///
/// Steps are used to convert channel values into "switches"
/// So for example if the `CHANNEL_AUX1` is > 1500 that might correspond to the motors being "armed"
/// while a value < 1500 might correspond to the motors being "disarmed".
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct RxChannelRange {
    pub start: u8,
    pub end: u8,
}

#[cfg(feature = "serde")]
impl PostcardValue<'_> for RxChannelRange {}

impl RxChannelRange {
    /// Constructor.
    #[must_use]
    pub const fn new() -> Self {
        Self { start: 0, end: 0 }
    }

    /// Construct from PWM values.
    #[must_use]
    pub fn from_pwm(pwm_start: u16, pwm_end: u16) -> Self {
        Self { start: Self::pwm_to_step(pwm_start), end: Self::pwm_to_step(pwm_end.max(pwm_start)) }
    }
}

impl Default for RxChannelRange {
    fn default() -> Self {
        Self::new()
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
    pub fn is_active(&self, rx_frame: &RxFrame, aux_channel_index: u8) -> bool {
        let channel_value: u16 = rx_frame.channel(aux_channel_index);
        Self::is_range_active(channel_value, self.start, self.end)
    }
}

type MacArrayType = [ModeActivationCondition; RcModes::MAX_MODE_ACTIVATION_CONDITION_COUNT];

/// Mode Activation Condition (MAC).<br><br>
///
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ModeActivationCondition {
    pub range: RxChannelRange,
    pub mode_id: u8,
    pub aux_channel_index: u8,
    pub mode_logic: u8,
    pub linked_to: u8,
}

#[cfg(feature = "serde")]
impl PostcardValue<'_> for ModeActivationCondition {}

impl ModeActivationCondition {
    /// Constructor.
    #[must_use]
    pub const fn new() -> Self {
        Self { range: RxChannelRange::new(), mode_id: 0, aux_channel_index: 0, mode_logic: 0, linked_to: 0 }
    }
    /// Constructor.
    #[must_use]
    pub const fn from_range_mode_channel(range: RxChannelRange, mode_id: u8, aux_channel_index: u8) -> Self {
        Self { range, mode_id, aux_channel_index, mode_logic: 0, linked_to: 0 }
    }
}

impl Default for ModeActivationCondition {
    fn default() -> Self {
        Self::new()
    }
}

impl ModeActivationCondition {
    /// Sets `range`, `mode_id`, and `aux_channel_index`.
    pub fn set(&mut self, range: RxChannelRange, mode_id: u8, aux_channel_index: u8) {
        self.range = range;
        self.mode_id = mode_id;
        self.aux_channel_index = aux_channel_index;
    }
    #[must_use]
    #[inline]
    pub fn is_active(&self, rx_frame: &RxFrame) -> bool {
        //let channel_value: u16 = rx_frame.auxiliary_channel(self.aux_channel_index);
        //RxChannelRange::is_range_active(channel_value, self.range.start, self.range.end)
        self.range.is_active(rx_frame, self.aux_channel_index)
    }
}

/// Radio control modes.<br><br>
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct RcModes {
    pub active_mac_count: usize,
    pub linked_mac_count: usize,
    pub active_modes: BitSet64,
    pub sticky_modes_was_ever_disabled: BitSet64,
    pub active_macs: [u8; Self::MAX_MODE_ACTIVATION_CONDITION_COUNT],
    pub linked_macs: [u8; Self::MAX_MODE_ACTIVATION_CONDITION_COUNT],
    pub macs: [ModeActivationCondition; Self::MAX_MODE_ACTIVATION_CONDITION_COUNT],
}

#[cfg(feature = "serde")]
impl PostcardValue<'_> for RcModes {}

impl RcModes {
    pub const MAX_MODE_ACTIVATION_CONDITION_COUNT: usize = 20;

    pub const FLIGHT_STABILIZATION_MODE_RATE: u8 = 0; // aka acro mode
    pub const FLIGHT_STABILIZATION_MODE_ANGLE: u8 = 1;
    pub const FLIGHT_STABILIZATION_MODE_HORIZON: u8 = 2;
    pub const FLIGHT_STABILIZATION_MODE_LEVEL_RACE: u8 = 3;

    #[must_use]
    pub const fn new() -> Self {
        Self {
            active_mac_count: 0,
            linked_mac_count: 0,
            active_modes: BitSet64::new(),
            sticky_modes_was_ever_disabled: BitSet64::new(),
            active_macs: [0u8; Self::MAX_MODE_ACTIVATION_CONDITION_COUNT],
            linked_macs: [0u8; Self::MAX_MODE_ACTIVATION_CONDITION_COUNT],
            macs: [ModeActivationCondition::new(); Self::MAX_MODE_ACTIVATION_CONDITION_COUNT],
        }
    }
    #[must_use]
    pub fn with_mac_arm() -> Self {
        let mac_arm = ModeActivationCondition::from_range_mode_channel(
            RxChannelRange::from_pwm(RxChannel::MID, RxChannel::HIGH),
            RcMode::ARM,
            RxChannel::AUX1_U8,
        );
        let mut macs = [ModeActivationCondition::new(); Self::MAX_MODE_ACTIVATION_CONDITION_COUNT];
        macs[0] = mac_arm;
        Self {
            active_mac_count: 0,
            linked_mac_count: 0,
            active_modes: BitSet64::new(),
            sticky_modes_was_ever_disabled: BitSet64::new(),
            active_macs: [0u8; Self::MAX_MODE_ACTIVATION_CONDITION_COUNT],
            linked_macs: [0u8; Self::MAX_MODE_ACTIVATION_CONDITION_COUNT],
            macs,
        }
    }
}

impl Default for RcModes {
    fn default() -> Self {
        Self::new()
    }
}

impl RcModes {
    //const LOGIC_OR: u8 = 0;
    const LOGIC_AND: u8 = 1;

    pub fn set_macs(&mut self, macs: &MacArrayType) {
        self.macs = *macs;
    }

    /// # Panics
    #[must_use]
    pub fn mac(&self, index: usize) -> ModeActivationCondition {
        assert!(index < Self::MAX_MODE_ACTIVATION_CONDITION_COUNT);
        self.macs[index]
    }

    pub fn set_mac(&mut self, index: usize, mac: ModeActivationCondition) {
        if index < Self::MAX_MODE_ACTIVATION_CONDITION_COUNT {
            self.macs[index] = mac;
        }
    }

    #[must_use]
    pub fn is_mode_active(&self, rc_mode: u8) -> bool {
        self.active_modes.test(rc_mode)
    }

    fn is_mac_configured(mac: ModeActivationCondition, empty_mac: ModeActivationCondition) -> bool {
        if mac == empty_mac {
            return true;
        }
        false
    }

    /// Build the list of used mac indices
    /// We can then use this to speed up processing by only evaluating used conditions.
    pub fn analyze_macs(&mut self) {
        let empty_mac = ModeActivationCondition::default();

        self.active_mac_count = 0;
        self.linked_mac_count = 0;

        #[allow(clippy::cast_possible_truncation)]
        for (ii, mac) in self.macs.into_iter().enumerate() {
            if mac.linked_to != 0 {
                self.linked_macs[self.linked_mac_count] = ii as u8;
                self.linked_mac_count += 1;
            } else if Self::is_mac_configured(mac, empty_mac) {
                self.active_macs[self.active_mac_count] = ii as u8;
                self.active_mac_count += 1;
            }
        }
    }

    /// `update_masks_for_mac`:
    ///
    /// The following are the possible logic states at each MAC update:
    ///     AND     NEW
    ///     ---     ---
    ///      F       F      - no previous AND macs evaluated, no previous active OR macs.
    ///      F       T      - at least 1 previous active OR mac (***this state is latched true***).
    ///      T       F      - all previous AND macs active, no previous active OR macs.
    ///      T       T      - at least 1 previous inactive AND mac, no previous active OR macs.
    ///
    fn update_masks_for_mac(
        mac: ModeActivationCondition,
        and_bitset: &mut BitSet64,
        new_bitset: &mut BitSet64,
        range_active: bool,
    ) {
        if and_bitset.test(mac.mode_id) || !new_bitset.test(mac.mode_id) {
            let b_and: bool = mac.mode_logic == Self::LOGIC_AND;
            #[allow(clippy::if_not_else)] // TODO: sort this if logic
            if !b_and {
                // OR mode_activation_condition
                if range_active {
                    and_bitset.reset(mac.mode_id);
                    new_bitset.set(mac.mode_id);
                }
            } else {
                // AND mode_activation_condition
                and_bitset.set(mac.mode_id);
                if !range_active {
                    new_bitset.set(mac.mode_id);
                }
            }
        }
    }

    fn update_masks_for_sticky_modes(
        &mut self,
        mac: ModeActivationCondition,
        and_bitset: &mut BitSet64,
        new_bitset: &mut BitSet64,
        range_active: bool,
    ) {
        const STICKY_MODE_BOOT_DELAY_US: u32 = 5_000_000; // 5 seconds
        if self.is_mode_active(mac.mode_id) {
            and_bitset.reset(mac.mode_id);
            new_bitset.set(mac.mode_id);
        } else if self.sticky_modes_was_ever_disabled.test(mac.mode_id) {
            Self::update_masks_for_mac(mac, and_bitset, new_bitset, range_active);
        } else {
            let time_us: u32 = 4;
            if time_us >= STICKY_MODE_BOOT_DELAY_US && !range_active {
                self.sticky_modes_was_ever_disabled.set(mac.mode_id);
            }
        }
    }

    pub fn update_activated_modes(&mut self, rx_frame: &RxFrame) {
        let mut new_bitset = BitSet64::default();
        let mut and_bitset = BitSet64::default();
        let mut sticky_modes = BitSet64::default();
        sticky_modes.set(RcMode::PARALYZE);

        // TODO: use enumerate in for
        // determine which conditions set/clear the mode
        let mut ii: usize = 0;
        for mac in self.macs {
            if sticky_modes.test(mac.mode_id) {
                let range_active = mac.range.is_active(rx_frame, mac.aux_channel_index);
                self.update_masks_for_sticky_modes(mac, &mut and_bitset, &mut new_bitset, range_active);
            } else if mac.mode_id < RcMode::COUNT {
                let range_active = mac.range.is_active(rx_frame, mac.aux_channel_index);
                Self::update_masks_for_mac(mac, &mut and_bitset, &mut new_bitset, range_active);
            }
            ii += 1;
            if ii == self.active_mac_count {
                break;
            }
        }

        // Update linked modes
        ii = 0;
        for mac in self.macs {
            let range_active = and_bitset.test(mac.linked_to) != new_bitset.test(mac.linked_to);
            Self::update_masks_for_mac(mac, &mut and_bitset, &mut new_bitset, range_active);
            ii += 1;
            if ii == self.linked_mac_count {
                break;
            }
        }

        self.active_modes = new_bitset ^ and_bitset;
    }

    #[must_use]
    pub fn update_modes(&self) -> (BitSet64, u8) {
        let mut rc_modes = BitSet64::default();
        let mut flight_stabilization_mode = Self::FLIGHT_STABILIZATION_MODE_RATE;

        if self.is_mode_active(RcMode::ANGLE) {
            rc_modes.set(RcMode::ANGLE);
            flight_stabilization_mode = Self::FLIGHT_STABILIZATION_MODE_ANGLE;
        }
        if self.is_mode_active(RcMode::HORIZON) {
            rc_modes.set(RcMode::HORIZON);
            // we don't support horizon mode, instead we use the horizon mode setting to invoke level race mode
            flight_stabilization_mode = Self::FLIGHT_STABILIZATION_MODE_LEVEL_RACE;
        }
        if self.is_mode_active(RcMode::ALTITUDE_HOLD) {
            rc_modes.set(RcMode::ALTITUDE_HOLD);
            flight_stabilization_mode = Self::FLIGHT_STABILIZATION_MODE_ANGLE;
        }
        if self.is_mode_active(RcMode::POSITION_HOLD) {
            rc_modes.set(RcMode::POSITION_HOLD);
            flight_stabilization_mode = Self::FLIGHT_STABILIZATION_MODE_ANGLE;
        }
        if self.is_mode_active(RcMode::FAILSAFE) {
            rc_modes.set(RcMode::FAILSAFE);
            flight_stabilization_mode = Self::FLIGHT_STABILIZATION_MODE_ANGLE;
        }
        if self.is_mode_active(RcMode::GPS_RESCUE) {
            rc_modes.set(RcMode::GPS_RESCUE);
            flight_stabilization_mode = Self::FLIGHT_STABILIZATION_MODE_ANGLE;
        }
        if self.is_mode_active(RcMode::AUTOPILOT) {
            rc_modes.set(RcMode::AUTOPILOT);
            flight_stabilization_mode = Self::FLIGHT_STABILIZATION_MODE_ANGLE;
        }

        if self.is_mode_active(RcMode::ARM) {
            rc_modes.set(RcMode::ARM);
        }
        if self.is_mode_active(RcMode::MAG) {
            rc_modes.set(RcMode::MAG);
        }
        if self.is_mode_active(RcMode::HEADFREE) {
            rc_modes.set(RcMode::HEADFREE);
        }
        if self.is_mode_active(RcMode::CHIRP) {
            rc_modes.set(RcMode::CHIRP);
        }
        if self.is_mode_active(RcMode::PASSTHRU) {
            rc_modes.set(RcMode::PASSTHRU);
        }
        (rc_modes, flight_stabilization_mode)
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::panic)]

    use super::*;

    fn _is_normal<T: Sized + Send + Sync + Unpin>() {}
    fn is_full<T: Sized + Send + Sync + Unpin + Copy + Clone + Default + PartialEq>() {}
    #[cfg(feature = "serde")]
    fn is_config<T: Serialize + for<'a> Deserialize<'a> + for<'a> PostcardValue<'a>>() {}

    #[test]
    fn normal_types() {
        is_full::<RxChannelRange>();
        is_full::<ModeActivationCondition>();
        is_full::<RcModes>();

        #[cfg(feature = "serde")]
        is_config::<RxChannelRange>();
        #[cfg(feature = "serde")]
        is_config::<ModeActivationCondition>();
        #[cfg(feature = "serde")]
        is_config::<RcModes>();
    }
    #[test]
    fn test_new() {
        let rc_modes = RcModes::default();
        assert_eq!(0, rc_modes.active_mac_count);
    }

    #[test]
    fn mac() {
        let mut rc_modes = RcModes::default();

        let mac_arm = ModeActivationCondition::from_range_mode_channel(
            RxChannelRange::from_pwm(RxChannel::MID, RxChannel::HIGH),
            RcMode::ARM,
            RxChannel::AUX1_U8,
        );
        rc_modes.set_mac(0, mac_arm);
        let mac_angle = ModeActivationCondition::from_range_mode_channel(
            RxChannelRange::from_pwm(1000, 1250),
            RcMode::ANGLE,
            RxChannel::AUX2_U8,
        );
        rc_modes.set_mac(1, mac_angle);

        let mut rx_frame = RxFrame::default();
        rx_frame.channels[RxChannel::AUX1] = RxChannel::MID_HIGH;
        let channel_value: u16 = rx_frame.channel(mac_arm.aux_channel_index);
        assert_eq!(1750, channel_value);

        assert!(mac_arm.is_active(&rx_frame));
        assert!(mac_arm.range.is_active(&rx_frame, mac_arm.aux_channel_index));

        rx_frame.channels[RxChannel::AUX2] = 1125;
        assert!(mac_angle.is_active(&rx_frame));

        rc_modes.update_activated_modes(&rx_frame);
        assert!(rc_modes.is_mode_active(RcMode::ARM));
        assert!(rc_modes.is_mode_active(RcMode::ANGLE));
        assert!(!rc_modes.is_mode_active(RcMode::ALTITUDE_HOLD));

        let (rc_modes_bitset, flight_stabilization_mode) = rc_modes.update_modes();
        assert!(rc_modes_bitset.test(RcMode::ARM));
        assert!(rc_modes_bitset.test(RcMode::ANGLE));
        assert!(!rc_modes_bitset.test(RcMode::ALTITUDE_HOLD));

        assert_eq!(RcModes::FLIGHT_STABILIZATION_MODE_ANGLE, flight_stabilization_mode);
    }

    #[test]
    fn mac_armed() {
        let mut rc_modes = RcModes::with_mac_arm();

        let mac_angle = ModeActivationCondition::from_range_mode_channel(
            RxChannelRange::from_pwm(1000, 1250),
            RcMode::ANGLE,
            RxChannel::AUX2_U8,
        );
        rc_modes.set_mac(1, mac_angle);

        let mut rx_frame = RxFrame::default();
        rx_frame.channels[RxChannel::AUX1] = RxChannel::MID_HIGH;

        rx_frame.channels[RxChannel::AUX2] = 1125;

        rc_modes.update_activated_modes(&rx_frame);
        assert!(rc_modes.is_mode_active(RcMode::ARM));
        assert!(rc_modes.is_mode_active(RcMode::ANGLE));
        assert!(!rc_modes.is_mode_active(RcMode::ALTITUDE_HOLD));
    }
}
