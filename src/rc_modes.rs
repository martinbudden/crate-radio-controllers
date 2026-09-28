use super::{RcMode, RxChannel, RxChannelRange, RxChannels};

use simple_bitset::BitSet64;

#[cfg(feature = "storage")]
use sequential_storage::map::PostcardValue;
#[cfg(feature = "serde")]
use {
    postcard::experimental::max_size::MaxSize,
    serde::{Deserialize, Serialize},
};

/// Mode Activation Condition (MAC).<br><br>
///
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize, MaxSize))]
pub struct ModeActivationCondition {
    pub range: RxChannelRange,
    pub mode_id: RcMode,
    pub channel: RxChannel,
    pub mode_logic: u8,
    pub linked_to: u8,
}

#[cfg(feature = "storage")]
impl PostcardValue<'_> for ModeActivationCondition {}

impl Default for ModeActivationCondition {
    fn default() -> Self {
        Self::new(RcMode::Arm)
    }
}

impl ModeActivationCondition {
    /// Constructor.
    #[must_use]
    pub const fn new(mode_id: RcMode) -> Self {
        Self { range: RxChannelRange::new(), mode_id, channel: RxChannel::Aux1, mode_logic: 0, linked_to: 0 }
    }
    /// Set the aux channel index of a newly constructed MAC.
    #[must_use]
    pub const fn with_channel(mut self, channel: RxChannel) -> Self {
        self.channel = channel;
        self
    }
    /// Set the range of a newly constructed MAC.
    #[must_use]
    pub const fn with_range(mut self, range: RxChannelRange) -> Self {
        self.range = range;
        self
    }
    /// Set the mode id of a newly constructed MAC.
    #[must_use]
    pub const fn with_mode_id(mut self, mode_id: RcMode) -> Self {
        self.mode_id = mode_id;
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
    pub const fn from_range_mode_channel(range: RxChannelRange, mode_id: RcMode, channel: RxChannel) -> Self {
        Self { range, mode_id, channel, mode_logic: 0, linked_to: 0 }
    }
}

impl ModeActivationCondition {
    /// Sets `range`, `mode_id`, and `aux_channel_index`.
    pub fn set(&mut self, range: RxChannelRange, mode_id: RcMode, channel: RxChannel) {
        self.range = range;
        self.mode_id = mode_id;
        self.channel = channel;
    }
    #[must_use]
    #[inline]
    pub fn is_active(&self, channels: &RxChannels) -> bool {
        //let channel_value: u16 = rx_frame.auxiliary_channel(self.aux_channel_index);
        //RxChannelRange::is_range_active(channel_value, self.range.start, self.range.end)
        self.range.is_active(channels, self.channel)
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
    pub macs: [Option<ModeActivationCondition>; Self::MAX_MODE_ACTIVATION_CONDITION_COUNT],
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
            macs: [None; Self::MAX_MODE_ACTIVATION_CONDITION_COUNT],
        }
    }

    /// Constructor with a single MAC for ARM mode on AUX1.
    #[must_use]
    pub fn with_mac_arm() -> Self {
        let mut macs = [None; Self::MAX_MODE_ACTIVATION_CONDITION_COUNT];
        macs[0] = Some(
            ModeActivationCondition::new(RcMode::Arm)
                .with_channel(RxChannel::Aux1)
                .with_range(RxChannelRange::from_pwm(RxChannel::MID, RxChannel::HIGH)),
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
        let mut macs = [None; Self::MAX_MODE_ACTIVATION_CONDITION_COUNT];
        macs[0] = Some(
            ModeActivationCondition::new(RcMode::Arm)
                .with_channel(RxChannel::Aux1)
                .with_range(RxChannelRange::from_pwm(RxChannel::MID, RxChannel::HIGH)),
        );

        macs[1] = Some(
            ModeActivationCondition::new(RcMode::Horizon)
                .with_channel(RxChannel::Aux2)
                .with_range(RxChannelRange::from_pwm(RxChannel::MID_LOW, RxChannel::MID_HIGH)),
        );

        macs[2] = Some(
            ModeActivationCondition::new(RcMode::Angle)
                .with_channel(RxChannel::Aux2)
                .with_range(RxChannelRange::from_pwm(RxChannel::MID_HIGH, RxChannel::HIGH)),
        );

        macs[3] = Some(
            ModeActivationCondition::new(RcMode::BeeperOn)
                .with_channel(RxChannel::Aux3)
                .with_range(RxChannelRange::from_pwm(RxChannel::MID, RxChannel::HIGH)),
        );

        macs[4] = Some(
            ModeActivationCondition::new(RcMode::CrashFlip)
                .with_channel(RxChannel::Aux4)
                .with_range(RxChannelRange::from_pwm(RxChannel::MID, RxChannel::HIGH)),
        );

        macs[5] = Some(
            ModeActivationCondition::new(RcMode::GpsRescue)
                .with_channel(RxChannel::Aux5)
                .with_range(RxChannelRange::from_pwm(RxChannel::MID, RxChannel::HIGH)),
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

    #[must_use]
    pub fn mac(&self, index: usize) -> Option<ModeActivationCondition> {
        if index < Self::MAX_MODE_ACTIVATION_CONDITION_COUNT { self.macs[index] } else { None }
    }

    pub fn set_mac(&mut self, index: usize, mac: ModeActivationCondition) {
        if index < Self::MAX_MODE_ACTIVATION_CONDITION_COUNT {
            self.macs[index] = Some(mac);
        }
    }

    #[must_use]
    pub fn is_mode_active(&self, rc_mode: RcMode) -> bool {
        self.active_modes.test(rc_mode as u8)
    }

    /// Build the list of used mac indices.
    /// We can then use this to speed up processing by only evaluating used conditions.
    /// Must be called before `update_activated_modes` is called.
    pub fn analyze_macs(&mut self) {
        self.active_mac_count = 0;
        self.linked_mac_count = 0;

        for (ii, mac) in self.macs.into_iter().enumerate() {
            if let Some(mac) = mac {
                #[allow(clippy::cast_possible_truncation)]
                if mac.linked_to != 0 {
                    self.linked_macs[self.linked_mac_count] = ii as u8;
                    self.linked_mac_count += 1;
                } else {
                    self.active_macs[self.active_mac_count] = ii as u8;
                    self.active_mac_count += 1;
                }
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
        let mac_mode_id = mac.mode_id as u8;
        if and_bitset.test(mac_mode_id) || !new_bitset.test(mac_mode_id) {
            if mac.mode_logic == Self::LOGIC_AND {
                // AND mode_activation_condition
                and_bitset.set(mac_mode_id);
                if !range_is_active {
                    new_bitset.set(mac_mode_id);
                }
            } else {
                // OR mode_activation_condition
                if range_is_active {
                    and_bitset.reset(mac_mode_id);
                    new_bitset.set(mac_mode_id);
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
        let mac_mode_id = mac.mode_id as u8;
        if active_modes.test(mac_mode_id) {
            and_bitset.reset(mac_mode_id);
            new_bitset.set(mac_mode_id);
        } else if sticky_modes_was_ever_disabled.test(mac_mode_id) {
            Self::update_masks_for_mac(mac, and_bitset, new_bitset, range_active);
        } else {
            let time_us: u32 = 4;
            if time_us >= STICKY_MODE_BOOT_DELAY_US && !range_active {
                sticky_modes_was_ever_disabled.set(mac_mode_id);
            }
        }
    }

    /// Updates the activated modes using the `RxFrame` values and the mode activation conditions.
    /// NOTE: `analyze_macs` must have been called before this function is used.
    pub fn update_activated_modes(&mut self, rx_channels: &RxChannels) {
        let mut new_bitset = BitSet64::default();
        let mut and_bitset = BitSet64::default();
        let mut sticky_modes = BitSet64::default();
        sticky_modes.set(RcMode::Paralyze as u8);

        // Determine which conditions set/clear the mode.
        for mac in self.macs[..self.active_mac_count].iter().flatten() {
            let mac_mode_id = mac.mode_id as u8;
            if sticky_modes.test(mac_mode_id) {
                let range_is_active = mac.range.is_active(rx_channels, mac.channel);
                Self::update_masks_for_sticky_modes(
                    self.active_modes,
                    &mut self.sticky_modes_was_ever_disabled,
                    *mac,
                    &mut and_bitset,
                    &mut new_bitset,
                    range_is_active,
                );
            } else if mac_mode_id < RcMode::COUNT {
                let range_is_active = mac.range.is_active(rx_channels, mac.channel);
                Self::update_masks_for_mac(*mac, &mut and_bitset, &mut new_bitset, range_is_active);
            }
        }
        // Update linked modes
        for mac in self.macs[..self.linked_mac_count].iter().flatten() {
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
    fn example() {
        let mut rc_modes = RcModes::default();

        let mac_arm = ModeActivationCondition::new(RcMode::Arm)
            .with_channel(RxChannel::Aux1)
            .with_range(RxChannelRange::from_pwm(1500, 2000));

        rc_modes.set_mac(0, mac_arm);

        let mut rx_channels = RxChannels::default();

        rx_channels[RxChannel::Aux1] = 1750;
        assert!(mac_arm.is_active(&rx_channels));

        rx_channels[RxChannel::Aux1] = 1250;
        assert!(!mac_arm.is_active(&rx_channels));
    }

    #[test]
    fn mac() {
        let mut rc_modes = RcModes::default();

        let mac_arm = ModeActivationCondition::from_range_mode_channel(
            RxChannelRange::from_pwm(RxChannel::MID, RxChannel::HIGH),
            RcMode::Arm,
            RxChannel::Aux1,
        );
        rc_modes.set_mac(0, mac_arm);
        let mac_angle = ModeActivationCondition::from_range_mode_channel(
            RxChannelRange::from_pwm(1000, 1250),
            RcMode::Angle,
            RxChannel::Aux2,
        );
        rc_modes.set_mac(1, mac_angle);
        rc_modes.analyze_macs();

        let mut rx_channels = RxChannels::default();
        rx_channels[RxChannel::Aux1] = RxChannel::MID_HIGH;
        let channel_value: u16 = rx_channels.channel(mac_arm.channel);
        assert_eq!(1750, channel_value);

        assert!(mac_arm.is_active(&rx_channels));
        assert!(mac_arm.range.is_active(&rx_channels, mac_arm.channel));

        rx_channels[RxChannel::Aux2] = 1125;
        assert!(mac_angle.is_active(&rx_channels));

        rc_modes.update_activated_modes(&rx_channels);
        assert!(rc_modes.is_mode_active(RcMode::Arm));
        assert!(rc_modes.is_mode_active(RcMode::Angle));
        assert!(!rc_modes.is_mode_active(RcMode::AltitudeHold));
    }

    #[test]
    fn mac_armed() {
        let mut rc_modes = RcModes::with_mac_arm();

        let mac_angle = ModeActivationCondition::from_range_mode_channel(
            RxChannelRange::from_pwm(1000, 1250),
            RcMode::Angle,
            RxChannel::Aux2,
        );
        rc_modes.set_mac(1, mac_angle);
        rc_modes.analyze_macs();

        let mut rx_channels = RxChannels::default();
        rx_channels[RxChannel::Aux1] = RxChannel::MID_HIGH;

        rx_channels[RxChannel::Aux2] = 1125;

        rc_modes.update_activated_modes(&rx_channels);
        assert!(rc_modes.is_mode_active(RcMode::Arm));
        assert!(rc_modes.is_mode_active(RcMode::Angle));
        assert!(!rc_modes.is_mode_active(RcMode::AltitudeHold));
    }
}
