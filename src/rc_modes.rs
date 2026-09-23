use super::{RcMode, RxChannel, RxChannelRange, RxFrame};

use simple_bitset::BitSet64;

#[cfg(feature = "storage")]
use sequential_storage::map::PostcardValue;
#[cfg(feature = "serde")]
use {
    postcard::experimental::max_size::MaxSize,
    serde::{Deserialize, Serialize},
};

type MacArrayType = [ModeActivationCondition; RcModes::MAX_MODE_ACTIVATION_CONDITION_COUNT];

/// Mode Activation Condition (MAC).<br><br>
///
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize, MaxSize))]
pub struct ModeActivationCondition {
    pub range: RxChannelRange,
    pub mode_id: u8,
    pub aux_channel_index: u8,
    pub mode_logic: u8,
    pub linked_to: u8,
}

#[cfg(feature = "storage")]
impl PostcardValue<'_> for ModeActivationCondition {}

impl Default for ModeActivationCondition {
    fn default() -> Self {
        Self::new()
    }
}

impl ModeActivationCondition {
    /// Constructor.
    #[must_use]
    pub const fn new() -> Self {
        Self { range: RxChannelRange::new(), mode_id: 0, aux_channel_index: 0, mode_logic: 0, linked_to: 0 }
    }
    /// Set the range of a newly MAC.
    #[must_use]
    pub const fn with_range(mut self, range: RxChannelRange) -> Self {
        self.range = range;
        self
    }
    /// Set the mode id of a newly constructed MAC.
    #[must_use]
    pub const fn with_mode_id(mut self, mode_id: u8) -> Self {
        self.mode_id = mode_id;
        self
    }
    /// Set the aux channel index of a newly constructed MAC.
    #[must_use]
    pub const fn with_aux_channel_index(mut self, aux_channel_index: u8) -> Self {
        self.aux_channel_index = aux_channel_index;
        self
    }
    /// Set the mode logic of a newly constructed MAC.
    #[must_use]
    pub const fn with_mode_logic(mut self, mode_logic: u8) -> Self {
        self.mode_logic = mode_logic;
        self
    }
    /// Set the linked to of a newly constructed MAC.
    #[must_use]
    pub const fn with_linked_to(mut self, linked_to: u8) -> Self {
        self.linked_to = linked_to;
        self
    }
    /// Constructor.
    #[must_use]
    pub const fn from_range_mode_channel(range: RxChannelRange, mode_id: u8, aux_channel_index: u8) -> Self {
        Self { range, mode_id, aux_channel_index, mode_logic: 0, linked_to: 0 }
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
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize, MaxSize))]
pub struct RcModes {
    pub active_mac_count: usize,
    pub linked_mac_count: usize,
    pub active_modes: BitSet64,
    pub sticky_modes_was_ever_disabled: BitSet64,
    pub active_macs: [u8; Self::MAX_MODE_ACTIVATION_CONDITION_COUNT],
    pub linked_macs: [u8; Self::MAX_MODE_ACTIVATION_CONDITION_COUNT],
    pub macs: [ModeActivationCondition; Self::MAX_MODE_ACTIVATION_CONDITION_COUNT],
}

#[cfg(feature = "storage")]
impl PostcardValue<'_> for RcModes {}

impl Default for RcModes {
    fn default() -> Self {
        Self::new()
    }
}

impl RcModes {
    pub const MAX_MODE_ACTIVATION_CONDITION_COUNT: usize = 20;

    /// Constructor.
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

    /// Constructor with a single MAC for ARM mode on AUX1.
    #[must_use]
    pub fn with_mac_arm() -> Self {
        let mut macs = [ModeActivationCondition::new(); Self::MAX_MODE_ACTIVATION_CONDITION_COUNT];
        macs[0] = ModeActivationCondition::from_range_mode_channel(
            RxChannelRange::from_pwm(RxChannel::MID, RxChannel::HIGH),
            RcMode::ARM,
            RxChannel::AUX1_U8,
        );
        let mut this = RcModes {
            active_mac_count: 0,
            linked_mac_count: 0,
            active_modes: BitSet64::new(),
            sticky_modes_was_ever_disabled: BitSet64::new(),
            active_macs: [0u8; Self::MAX_MODE_ACTIVATION_CONDITION_COUNT],
            linked_macs: [0u8; Self::MAX_MODE_ACTIVATION_CONDITION_COUNT],
            macs,
        };
        this.analyze_macs();
        this
    }

    /// Constructor with a set of conventional MACs for common modes:
    /// - `ARM` on `AUX1`
    /// - `HORIZON` on `AUX2` mid-low to mid-high
    /// - `ANGLE` on `AUX2` mid-high to high
    /// - `BEEPER_ON` on `AUX3`
    /// - `CRASH_FLIP` on `AUX4`
    /// - `GPS_RESCUE` on `AUX5`
    #[must_use]
    pub fn with_macs_conventional() -> Self {
        let mut macs = [ModeActivationCondition::new(); Self::MAX_MODE_ACTIVATION_CONDITION_COUNT];
        macs[0] = ModeActivationCondition::from_range_mode_channel(
            RxChannelRange::from_pwm(RxChannel::MID, RxChannel::HIGH),
            RcMode::ARM,
            RxChannel::AUX1_U8,
        );
        macs[1] = ModeActivationCondition::from_range_mode_channel(
            RxChannelRange::from_pwm(RxChannel::MID_LOW, RxChannel::MID_HIGH),
            RcMode::HORIZON,
            RxChannel::AUX2_U8,
        );
        macs[2] = ModeActivationCondition::from_range_mode_channel(
            RxChannelRange::from_pwm(RxChannel::MID_HIGH, RxChannel::HIGH),
            RcMode::ANGLE,
            RxChannel::AUX2_U8,
        );
        macs[3] = ModeActivationCondition::from_range_mode_channel(
            RxChannelRange::from_pwm(RxChannel::MID, RxChannel::HIGH),
            RcMode::BEEPER_ON,
            RxChannel::AUX3_U8,
        );
        macs[4] = ModeActivationCondition::from_range_mode_channel(
            RxChannelRange::from_pwm(RxChannel::MID, RxChannel::HIGH),
            RcMode::CRASH_FLIP,
            RxChannel::AUX4_U8,
        );
        macs[5] = ModeActivationCondition::from_range_mode_channel(
            RxChannelRange::from_pwm(RxChannel::MID, RxChannel::HIGH),
            RcMode::GPS_RESCUE,
            RxChannel::AUX5_U8,
        );
        let mut this = RcModes {
            active_mac_count: 0,
            linked_mac_count: 0,
            active_modes: BitSet64::new(),
            sticky_modes_was_ever_disabled: BitSet64::new(),
            active_macs: [0u8; Self::MAX_MODE_ACTIVATION_CONDITION_COUNT],
            linked_macs: [0u8; Self::MAX_MODE_ACTIVATION_CONDITION_COUNT],
            macs,
        };
        this.analyze_macs();
        this
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

    #[inline]
    fn is_mac_configured(mac: ModeActivationCondition) -> bool {
        const EMPTY_MAC: ModeActivationCondition = ModeActivationCondition::new();
        mac != EMPTY_MAC
    }

    /// Build the list of used mac indices.
    /// We can then use this to speed up processing by only evaluating used conditions.
    /// Must be called before `update_activated_modes` is called.
    pub fn analyze_macs(&mut self) {
        self.active_mac_count = 0;
        self.linked_mac_count = 0;

        for (ii, mac) in self.macs.into_iter().enumerate() {
            #[allow(clippy::cast_possible_truncation)]
            if mac.linked_to != 0 {
                self.linked_macs[self.linked_mac_count] = ii as u8;
                self.linked_mac_count += 1;
            } else if Self::is_mac_configured(mac) {
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
        range_is_active: bool,
    ) {
        if and_bitset.test(mac.mode_id) || !new_bitset.test(mac.mode_id) {
            if mac.mode_logic == Self::LOGIC_AND {
                // AND mode_activation_condition
                and_bitset.set(mac.mode_id);
                if !range_is_active {
                    new_bitset.set(mac.mode_id);
                }
            } else {
                // OR mode_activation_condition
                if range_is_active {
                    and_bitset.reset(mac.mode_id);
                    new_bitset.set(mac.mode_id);
                }
            }
        }
    }

    fn update_masks_for_sticky_modes(
        active_modes: BitSet64,
        sticky_modes_was_ever_disabled: &mut BitSet64,
        mac: ModeActivationCondition,
        and_bitset: &mut BitSet64,
        new_bitset: &mut BitSet64,
        range_active: bool,
    ) {
        const STICKY_MODE_BOOT_DELAY_US: u32 = 5_000_000; // 5 seconds
        if active_modes.test(mac.mode_id) {
            and_bitset.reset(mac.mode_id);
            new_bitset.set(mac.mode_id);
        } else if sticky_modes_was_ever_disabled.test(mac.mode_id) {
            Self::update_masks_for_mac(mac, and_bitset, new_bitset, range_active);
        } else {
            let time_us: u32 = 4;
            if time_us >= STICKY_MODE_BOOT_DELAY_US && !range_active {
                sticky_modes_was_ever_disabled.set(mac.mode_id);
            }
        }
    }

    /// Updates the activated modes using the `RxFrame` values and the mode activation conditions.
    /// `analyze_macs` must have been called before this function is called.
    pub fn update_activated_modes(&mut self, rx_frame: &RxFrame) {
        let mut new_bitset = BitSet64::default();
        let mut and_bitset = BitSet64::default();
        let mut sticky_modes = BitSet64::default();
        sticky_modes.set(RcMode::PARALYZE);

        // Determine which conditions set/clear the mode.
        for mac in &self.macs[..self.active_mac_count] {
            if sticky_modes.test(mac.mode_id) {
                let range_is_active = mac.range.is_active(rx_frame, mac.aux_channel_index);
                Self::update_masks_for_sticky_modes(
                    self.active_modes,
                    &mut self.sticky_modes_was_ever_disabled,
                    *mac,
                    &mut and_bitset,
                    &mut new_bitset,
                    range_is_active,
                );
            } else if mac.mode_id < RcMode::COUNT {
                let range_is_active = mac.range.is_active(rx_frame, mac.aux_channel_index);
                Self::update_masks_for_mac(*mac, &mut and_bitset, &mut new_bitset, range_is_active);
            }
        }
        // Update linked modes
        for mac in &self.macs[..self.linked_mac_count] {
            let range_is_active = and_bitset.test(mac.linked_to) != new_bitset.test(mac.linked_to);
            Self::update_masks_for_mac(*mac, &mut and_bitset, &mut new_bitset, range_is_active);
        }

        self.active_modes = new_bitset ^ and_bitset;
    }
}

#[cfg(test)]
mod test_traits {
    use super::*;

    fn _is_normal<T: Sized + Send + Sync + Unpin>() {}
    fn is_full<T: Sized + Send + Sync + Unpin + Copy + Clone + Default + PartialEq>() {}
    #[cfg(feature = "serde")]
    fn is_serde<T: Serialize + MaxSize + for<'a> Deserialize<'a>>() {}
    #[cfg(feature = "storage")]
    fn is_storage<T: for<'a> PostcardValue<'a>>() {}

    #[test]
    fn normal_types() {
        is_full::<ModeActivationCondition>();
        is_full::<RcModes>();
    }
    #[cfg(feature = "serde")]
    #[test]
    fn serde_types() {
        is_serde::<ModeActivationCondition>();
        is_serde::<RcModes>();
    }
    #[cfg(feature = "storage")]
    #[test]
    fn storage_types() {
        is_storage::<ModeActivationCondition>();
        is_storage::<RcModes>();
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::panic)]

    use super::*;
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
        rc_modes.analyze_macs();

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
        rc_modes.analyze_macs();

        let mut rx_frame = RxFrame::default();
        rx_frame.channels[RxChannel::AUX1] = RxChannel::MID_HIGH;

        rx_frame.channels[RxChannel::AUX2] = 1125;

        rc_modes.update_activated_modes(&rx_frame);
        assert!(rc_modes.is_mode_active(RcMode::ARM));
        assert!(rc_modes.is_mode_active(RcMode::ANGLE));
        assert!(!rc_modes.is_mode_active(RcMode::ALTITUDE_HOLD));
    }
}
