use super::{
    ModeActivationCondition, ModeActivationConditions, RcMode, RcModeLogic, RxChannel, RxChannelRange, RxChannels,
};

use simple_bitset::BitSet64;

#[cfg(feature = "storage")]
use sequential_storage::map::PostcardValue;
#[cfg(feature = "serde")]
use {
    postcard::experimental::max_size::MaxSize,
    serde::{Deserialize, Serialize},
};

/// Radio control modes.<br><br>
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize, MaxSize))]
pub struct RcModes {
    pub active_mac_count: usize,
    pub linked_mac_count: usize,
    pub active_modes: BitSet64,
    pub sticky_modes_was_ever_disabled: BitSet64,
    pub active_macs: [u8; ModeActivationConditions::COUNT],
    pub linked_macs: [u8; ModeActivationConditions::COUNT],
    pub macs: ModeActivationConditions,
}

#[cfg(feature = "storage")]
impl PostcardValue<'_> for RcModes {}

impl Default for RcModes {
    fn default() -> Self {
        Self::new()
    }
}

impl RcModes {
    /// Constructor.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            active_mac_count: 0,
            linked_mac_count: 0,
            active_modes: BitSet64::new(),
            sticky_modes_was_ever_disabled: BitSet64::new(),
            active_macs: [0u8; ModeActivationConditions::COUNT],
            linked_macs: [0u8; ModeActivationConditions::COUNT],
            macs: ModeActivationConditions::new(),
        }
    }

    /// Add a single MAC for ARM mode on AUX1 to newly constructed MAC.
    #[must_use]
    pub fn with_mac_arm(mut self) -> Self {
        let mac = ModeActivationCondition::new(RcMode::Arm)
            .with_channel(RxChannel::Aux1)
            .with_range(RxChannelRange::from_pwm(RxChannel::MID, RxChannel::HIGH));
        _ = self.push_mac(mac);

        self
    }

    /// Add a set of conventional MACs for common modes to a newly constructed MAC:
    /// - `ARM` on `AUX1`
    /// - `HORIZON` on `AUX2` mid-low to mid-high
    /// - `ANGLE` on `AUX2` mid-high to high
    /// - `BEEPER_ON` on `AUX3`
    /// - `CRASH_FLIP` on `AUX4`
    /// - `GPS_RESCUE` on `AUX5`
    #[must_use]
    pub fn with_macs_conventional(mut self) -> Self {
        let mac = ModeActivationCondition::new(RcMode::Arm)
            .with_channel(RxChannel::Aux1)
            .with_range(RxChannelRange::from_pwm(RxChannel::MID, RxChannel::HIGH));
        _ = self.push_mac(mac);

        let mac = ModeActivationCondition::new(RcMode::Horizon)
            .with_channel(RxChannel::Aux2)
            .with_range(RxChannelRange::from_pwm(RxChannel::MID_LOW, RxChannel::MID_HIGH));
        _ = self.push_mac(mac);

        let mac = ModeActivationCondition::new(RcMode::Angle)
            .with_channel(RxChannel::Aux2)
            .with_range(RxChannelRange::from_pwm(RxChannel::MID_HIGH, RxChannel::HIGH));
        _ = self.push_mac(mac);

        let mac = ModeActivationCondition::new(RcMode::BeeperOn)
            .with_channel(RxChannel::Aux3)
            .with_range(RxChannelRange::from_pwm(RxChannel::MID, RxChannel::HIGH));
        _ = self.push_mac(mac);

        let mac = ModeActivationCondition::new(RcMode::CrashFlip)
            .with_channel(RxChannel::Aux4)
            .with_range(RxChannelRange::from_pwm(RxChannel::MID, RxChannel::HIGH));
        _ = self.push_mac(mac);

        let mac = ModeActivationCondition::new(RcMode::GpsRescue)
            .with_channel(RxChannel::Aux5)
            .with_range(RxChannelRange::from_pwm(RxChannel::MID, RxChannel::HIGH));
        _ = self.push_mac(mac);

        self
    }
}

impl RcModes {
    #[must_use]
    pub fn mac(&self, index: usize) -> Option<ModeActivationCondition> {
        if index < ModeActivationConditions::COUNT { self.macs[index] } else { None }
    }

    pub fn set_mac(&mut self, index: usize, mac: ModeActivationCondition) {
        if index < ModeActivationConditions::COUNT {
            self.macs[index] = Some(mac);
        }
    }

    /// Appends an item to the first available `None` slot.
    /// # Errors
    /// Returns `Ok(usize)` with the slot index on success, or `Err(ModeActivationCondition)` if full.
    pub fn push_mac(&mut self, mac: ModeActivationCondition) -> Result<u8, ModeActivationCondition> {
        let result = self.macs.push(mac);
        if let Ok(index) = result {
            if mac.linked_to != 0 {
                self.linked_macs[self.linked_mac_count] = index;
                self.linked_mac_count += 1;
            } else {
                self.active_macs[self.active_mac_count] = index;
                self.active_mac_count += 1;
            }
            Ok(index)
        } else {
            result
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
            if mac.mode_logic == RcModeLogic::And {
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
        is_full::<RcModes>();
        #[cfg(feature = "serde")]
        is_serde::<RcModes>();
        #[cfg(feature = "storage")]
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
            .with_range(RxChannelRange::from_pwm(1500, 2100));
        _ = rc_modes.push_mac(mac_arm);

        let mac_horizon = ModeActivationCondition::new(RcMode::Horizon)
            .with_channel(RxChannel::Aux2)
            .with_range(RxChannelRange::from_pwm(1250, 1750));
        _ = rc_modes.push_mac(mac_horizon);

        let mac_angle = ModeActivationCondition::new(RcMode::Angle)
            .with_channel(RxChannel::Aux2)
            .with_range(RxChannelRange::from_pwm(1750, 2100));
        _ = rc_modes.push_mac(mac_angle);

        let mut rx_channels = RxChannels::default();

        rx_channels[RxChannel::Aux1] = 1750;
        rx_channels[RxChannel::Aux2] = 1800;

        // need to call update activated modes when the channel values change
        rc_modes.update_activated_modes(&rx_channels);

        // Check arm mode is active.
        assert!(rc_modes.is_mode_active(RcMode::Arm));
        // Check angle mode is active.
        assert!(rc_modes.is_mode_active(RcMode::Angle));
        // Check horizon mode is NOT active.
        assert!(!rc_modes.is_mode_active(RcMode::Horizon));
    }

    #[rustfmt::skip]
    #[test]
    fn example_combined() {
        use crate::IbusDecoder;

        // Simulated byte stream from serial port.
        let byte_stream = [
            0x20, 0x40, 0xDC, 0x05, 0xDC, 0x05, 0x4C, 0x04, 0xDC, 0x05, 0xE8, 0x03, 0xD0, 0x07, 0xDC, 0x05,
            0xE8, 0x03, 0xB0, 0x04, 0x14, 0x05, 0x78, 0x05, 0x40, 0x06, 0xA4, 0x06, 0x08, 0x07, 0xD5, 0xF6,
        ];

        // Initialization: set up decoder and`RcModes`.
        let mut decoder = IbusDecoder::new();
        let mut rc_modes = RcModes::default();

        let mac_arm = ModeActivationCondition::new(RcMode::Arm)
            .with_channel(RxChannel::Aux1)
            .with_range(RxChannelRange::from_pwm(1500, RxChannelRange::MAX));
        _ = rc_modes.push_mac(mac_arm);
        let mac_horizon = ModeActivationCondition::new(RcMode::Horizon)
            .with_channel(RxChannel::Aux2)
            .with_range(RxChannelRange::from_pwm(1250, 1750));
        _ = rc_modes.push_mac(mac_horizon);
        let mac_angle = ModeActivationCondition::new(RcMode::Angle)
            .with_channel(RxChannel::Aux2)
            .with_range(RxChannelRange::from_pwm(1750, RxChannelRange::MAX));
        _ = rc_modes.push_mac(mac_angle);

        // Main program loop: decode the byte stream from the radio,
        // and take action depending on the RC modes.

        let mut result = None;
        for &byte in byte_stream.iter() {
            result = decoder.on_byte_received(byte);
        }

        if let Some((rx_channels, _rx_link_status)) = result {
            // Call `update_activated_modes` since we have new channel values.
            rc_modes.update_activated_modes(&rx_channels);

            // From the decoded byte steam we have: AUX1 = 1000, AUX2 = 2000

            // Check arm mode is NOT active.
            assert!(!rc_modes.is_mode_active(RcMode::Arm));
            // Check angle mode is active.
            assert!(rc_modes.is_mode_active(RcMode::Angle));
            // Check horizon mode is NOT active.
            assert!(!rc_modes.is_mode_active(RcMode::Horizon));
        } else {
            panic!("decode failed");
        }
    }

    #[test]
    fn mac() {
        let mut rc_modes = RcModes::default();

        let mac_arm = ModeActivationCondition::new(RcMode::Arm)
            .with_channel(RxChannel::Aux1)
            .with_range(RxChannelRange::from_pwm(RxChannel::MID, RxChannel::HIGH));
        _ = rc_modes.push_mac(mac_arm);
        let mac_angle = ModeActivationCondition::new(RcMode::Angle)
            .with_channel(RxChannel::Aux2)
            .with_range(RxChannelRange::from_pwm(1000, 1250));
        _ = rc_modes.push_mac(mac_angle);

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
        let mut rc_modes = RcModes::new().with_mac_arm();

        let mac_angle = ModeActivationCondition::new(RcMode::Angle)
            .with_channel(RxChannel::Aux2)
            .with_range(RxChannelRange::from_pwm(1000, 1250));
        _ = rc_modes.push_mac(mac_angle);

        let mut rx_channels = RxChannels::default();
        rx_channels[RxChannel::Aux1] = RxChannel::MID_HIGH;

        rx_channels[RxChannel::Aux2] = 1125;

        rc_modes.update_activated_modes(&rx_channels);
        assert!(rc_modes.is_mode_active(RcMode::Arm));
        assert!(rc_modes.is_mode_active(RcMode::Angle));
        assert!(!rc_modes.is_mode_active(RcMode::AltitudeHold));
    }
}
