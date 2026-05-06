use crate::{RadioControlMessage, RxFrame};
use vqm::BitSet64;

/// PWM channels are divided into "steps". Steps are 25 units wide<br>
/// There are 48 steps between 900 and 2100.<br>
///     a step value of 0 corresponds to a channel value of 900 or less.<br>
///     a step value of 48 corresponds to a channel value of 2100 or more.<br>
///
/// Steps are used to convert channel values into "switches"
/// So for example if the `CHANNEL_AUX1` is > 1500 that might correspond to the motors being "armed"
/// while a value < 1500 might correspond to the motors being "disarmed".
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct RxChannelRange {
    start: u8,
    end: u8,
}

impl RxChannelRange {
    pub const MIN: u16 = 900;
    pub const MID: u16 = 1500;
    pub const MAX: u16 = 2100;

    pub const STEP: u16 = 25;
    pub const STEP_MIN: u16 = 0;
    pub const STEP_MID: u16 = ((Self::MID - Self::MIN) / Self::STEP);
    pub const STEP_MAX: u16 = ((Self::MAX - Self::MIN) / Self::STEP);

    fn step_to_pwm(step: u8) -> u16 {
        Self::MIN + Self::STEP * u16::from(step)
    }

    #[allow(clippy::cast_possible_truncation)]
    fn pwm_to_step(pwm: u16) -> u8 {
        ((pwm.clamp(Self::MIN, Self::MAX) - Self::MIN) / Self::STEP) as u8
    }

    pub fn set(&mut self, pwm_start: u16, pwm_end: u16) {
        if pwm_end > pwm_start {
            self.start = Self::pwm_to_step(pwm_start);
            self.end = Self::pwm_to_step(pwm_end);
        }
    }

    pub fn pwm_range(&self) -> (u16, u16) {
        (Self::step_to_pwm(self.start), Self::step_to_pwm(self.end))
    }

    pub fn is_range_active(channel_value: u16, start: u8, end: u8) -> bool {
        if channel_value >= Self::MIN + u16::from(start) * Self::STEP
            && channel_value < Self::MIN + u16::from(end) * Self::STEP
        {
            return true;
        }
        false
    }

    pub fn is_active(&self, rx_frame: &RxFrame, aux_channel_index: u8) -> bool {
        let channel_value: u16 = rx_frame.auxiliary_channel(aux_channel_index);
        Self::is_range_active(channel_value, self.start, self.end)
    }
}

/// Mode Activation Condition (MAC).<br><br>
///
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ModeActivationCondition {
    pub range: RxChannelRange,
    pub mode_id: u8,
    pub aux_channel_index: u8,
    pub mode_logic: u8,
    pub linked_to: u8,
}

/// Radio control modes.<br><br>
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct RcModes {
    active_mac_count: usize,
    linked_mac_count: usize,
    active_modes: BitSet64,
    sticky_modes_was_ever_disabled: BitSet64,
    active_macs: [u8; Self::MAX_MODE_ACTIVATION_CONDITION_COUNT],
    linked_macs: [u8; Self::MAX_MODE_ACTIVATION_CONDITION_COUNT],
    macs: [ModeActivationCondition; Self::MAX_MODE_ACTIVATION_CONDITION_COUNT],
}

impl RcModes {
    pub fn new() -> Self {
        Self::default()
    }
}

impl RcModes {
    pub const MAX_MODE_ACTIVATION_CONDITION_COUNT: usize = 20;
    const PARALYZE: u8 = 34;
    const MSP_COUNT: u8 = 48;
}

type MacArray = [ModeActivationCondition; RcModes::MAX_MODE_ACTIVATION_CONDITION_COUNT];

impl RcModes {
    //const LOGIC_OR: u8 = 0;
    const LOGIC_AND: u8 = 1;

    pub fn set_macs(&mut self, macs: &MacArray) {
        self.macs = *macs;
    }

    /// # Panics
    pub fn mac(&self, index: usize) -> ModeActivationCondition {
        assert!(index < Self::MAX_MODE_ACTIVATION_CONDITION_COUNT);
        self.macs[index]
    }

    pub fn set_mac(&mut self, index: usize, mac: ModeActivationCondition) {
        if index < Self::MAX_MODE_ACTIVATION_CONDITION_COUNT {
            self.macs[index] = mac;
        }
    }
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
    #[allow(clippy::cast_possible_truncation)]
    pub fn analyze_macs(&mut self) {
        let empty_mac = ModeActivationCondition::default();

        self.active_mac_count = 0;
        self.linked_mac_count = 0;

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
        sticky_modes.set(Self::PARALYZE);

        // TODO: use enumerate in for
        // determine which conditions set/clear the mode
        let mut ii: usize = 0;
        for mac in self.macs {
            if sticky_modes.test(mac.mode_id) {
                let range_active = mac.range.is_active(rx_frame, mac.aux_channel_index);
                self.update_masks_for_sticky_modes(mac, &mut and_bitset, &mut new_bitset, range_active);
            } else if mac.mode_id < Self::MSP_COUNT {
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

    pub fn update_modes(&self) -> (BitSet64, u8) {
        let mut rc_modes = BitSet64::default();
        let mut stabilization_mode = 0u8;
        if self.is_mode_active(RcMode::ANGLE) {
            rc_modes.set(RcMode::ANGLE);
            stabilization_mode = RadioControlMessage::STABILIZATION_MODE_ANGLE;
        }

        if self.is_mode_active(RcMode::HORIZON) {
            rc_modes.set(RcMode::HORIZON);
            // we don't support horizon mode, instead we use the horizon mode setting to invoke level race mode
            stabilization_mode = RadioControlMessage::STABILIZATION_MODE_LEVEL_RACE;
        }
        if self.is_mode_active(RcMode::ALTITUDE_HOLD) {
            rc_modes.set(RcMode::ALTITUDE_HOLD);
            stabilization_mode = RadioControlMessage::STABILIZATION_MODE_ANGLE;
        }
        if self.is_mode_active(RcMode::POSITION_HOLD) {
            rc_modes.set(RcMode::POSITION_HOLD);
            stabilization_mode = RadioControlMessage::STABILIZATION_MODE_ANGLE;
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
        if self.is_mode_active(RcMode::FAILSAFE) {
            rc_modes.set(RcMode::FAILSAFE);
            stabilization_mode = RadioControlMessage::STABILIZATION_MODE_ANGLE;
        }
        if self.is_mode_active(RcMode::GPS_RESCUE) {
            rc_modes.set(RcMode::GPS_RESCUE);
            stabilization_mode = RadioControlMessage::STABILIZATION_MODE_ANGLE;
        }
        (rc_modes, stabilization_mode)
    }
}

/// Human readable name for an `RcMode`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RcModeName {
    pub id: u8,
    pub permanent_id: u8,
    pub name: &'static str,
}

/// Radio control modes, including armed/disarmed and flight modes.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RcMode {}

impl RcMode {
    pub const MAX_MODES_PER_PAGE: u8 = 32;
    pub const PERMANENT_ID_NONE: u8 = 255;

    // Arming flag
    pub const ARM: u8 = 0;

    // Flight mode flags
    pub const ANGLE: u8 = 1;
    pub const HORIZON: u8 = 2;
    pub const MAG: u8 = 3;
    pub const ALTITUDE_HOLD: u8 = 4;
    pub const POSITION_HOLD: u8 = 5;
    pub const HEADFREE: u8 = 6;
    pub const CHIRP: u8 = 7;
    pub const PASSTHRU: u8 = 8;
    pub const FAILSAFE: u8 = 9;
    pub const GPS_RESCUE: u8 = 10;
    pub const FLIGHTMODE_COUNT: u8 = 11;

    // RC mode flags
    pub const ANTIGRAVITY: u8 = Self::FLIGHTMODE_COUNT;
    pub const HEADADJ: u8 = 12;
    pub const CAMSTAB: u8 = 13;
    pub const BEEPER_ON: u8 = 14;
    pub const LED_LOW: u8 = 15;
    pub const CALIBRATE: u8 = 16;
    pub const OSD: u8 = 17;
    pub const TELEMETRY: u8 = 18;
    pub const SERVO1: u8 = 19;
    pub const SERVO2: u8 = 20;
    pub const SERVO3: u8 = 21;
    pub const BLACKBOX: u8 = 22;
    pub const AIRMODE: u8 = 23;
    pub const MODE_3D: u8 = 24;
    pub const FPV_ANGLE_MIX: u8 = 25;
    pub const BLACKBOX_ERASE: u8 = 26;
    pub const CAMERA1: u8 = 27;
    pub const CAMERA2: u8 = 28;
    pub const CAMERA3: u8 = 29;
    pub const CRASH_FLIP: u8 = 30;
    pub const PREARM: u8 = 31;
    pub const BEEP_GPS_COUNT: u8 = 32;
    pub const VTX_PIT_MODE: u8 = 33;
    pub const PARALYZE: u8 = 34;
    pub const USER1: u8 = 35;
    pub const USER2: u8 = 36;
    pub const USER3: u8 = 37;
    pub const USER4: u8 = 38;
    pub const PID_AUDIO: u8 = 39;
    pub const ACRO_TRAINER: u8 = 40;
    pub const VTX_CONTROL_DISABLE: u8 = 41;
    pub const LAUNCH_CONTROL: u8 = 42;
    pub const MSP_OVERRIDE: u8 = 43;
    pub const STICK_COMMAND_DISABLE: u8 = 44;
    pub const BEEPER_MUTE: u8 = 45;
    pub const READY: u8 = 46;
    pub const LAP_TIMER_RESET: u8 = 47;
    pub const COUNT: u8 = 47;

    pub const RC_MODES: [RcModeName; Self::COUNT as usize] = [
        RcModeName { id: Self::ARM, permanent_id: 0, name: "ARM" },
        RcModeName { id: Self::ANGLE, permanent_id: 1, name: "ANGLE" },
        RcModeName { id: Self::HORIZON, permanent_id: 2, name: "HORIZON" },
        RcModeName { id: Self::ALTITUDE_HOLD, permanent_id: 3, name: "ALTHOLD" },
        RcModeName { id: Self::ANTIGRAVITY, permanent_id: 4, name: "ANTI GRAVITY" },
        RcModeName { id: Self::MAG, permanent_id: 5, name: "MAG" },
        RcModeName { id: Self::HEADFREE, permanent_id: 6, name: "HEADFREE" },
        RcModeName { id: Self::HEADADJ, permanent_id: 7, name: "HEADADJ" },
        RcModeName { id: Self::CAMSTAB, permanent_id: 8, name: "CAMSTAB" },
        // RcModeName {id: Self::CAM_TRIG,    permanent_id:9,  name:"CAM_TRIG", },
        // RcModeName {id: Self::GPS_HOME,    permanent_id:10, name:"GPS HOME" },
        RcModeName { id: Self::POSITION_HOLD, permanent_id: 11, name: "POS HOLD" },
        RcModeName { id: Self::PASSTHRU, permanent_id: 12, name: "PASSTHRU" },
        RcModeName { id: Self::BEEPER_ON, permanent_id: 13, name: "BEEPER" },
        // RcModeName {id: Self::LEDMAX,     permanent_id:14, name:"LEDMAX" }, (removed)
        RcModeName { id: Self::LED_LOW, permanent_id: 15, name: "LEDLOW" },
        // RcModeName {id: Self::LLIGHTS,     permanent_id:16, name:"LLIGHTS" }, (removed)
        RcModeName { id: Self::CALIBRATE, permanent_id: 17, name: "CALIBRATE" },
        // RcModeName {id: Self::GOVERNOR,    permanent_id:18, name:"GOVERNOR" }, (removed)
        RcModeName { id: Self::OSD, permanent_id: 19, name: "OSD DISABLE" },
        RcModeName { id: Self::TELEMETRY, permanent_id: 20, name: "TELEMETRY" },
        // RcModeName {id: Self::GTUNE,       permanent_id:21, name:"GTUNE" }, (removed)
        // RcModeName {id: Self::RANGEFINDER, permanent_id:22, name:"RANGEFINDER" }, (removed)
        RcModeName { id: Self::SERVO1, permanent_id: 23, name: "SERVO1" },
        RcModeName { id: Self::SERVO2, permanent_id: 24, name: "SERVO2" },
        RcModeName { id: Self::SERVO3, permanent_id: 25, name: "SERVO3" },
        RcModeName { id: Self::BLACKBOX, permanent_id: 26, name: "BLACK" },
        RcModeName { id: Self::FAILSAFE, permanent_id: 27, name: "FAILSAFE" },
        RcModeName { id: Self::AIRMODE, permanent_id: 28, name: "AIR MODE" },
        RcModeName { id: Self::MODE_3D, permanent_id: 29, name: "3D DISABLE / SWITCH" },
        RcModeName { id: Self::FPV_ANGLE_MIX, permanent_id: 30, name: "FPV ANGLE MIX" },
        RcModeName { id: Self::BLACKBOX_ERASE, permanent_id: 31, name: "BLACK ERASE" },
        RcModeName { id: Self::CAMERA1, permanent_id: 32, name: "CAMERA CONTROL 1" },
        RcModeName { id: Self::CAMERA2, permanent_id: 33, name: "CAMERA CONTROL 2" },
        RcModeName { id: Self::CAMERA3, permanent_id: 34, name: "CAMERA CONTROL 3" },
        RcModeName { id: Self::CRASH_FLIP, permanent_id: 35, name: "FLIP OVER AFTER CRASH" },
        RcModeName { id: Self::PREARM, permanent_id: 36, name: "PREARM" },
        RcModeName { id: Self::BEEP_GPS_COUNT, permanent_id: 37, name: "GPS BEEP SATELLITE COUNT" },
        // RcModeName {id: Self::BOX3D_ON_A_SWITCH,.permanent_id= 38, name:"3D ON A SWITCH", }, (removed)
        RcModeName { id: Self::VTX_PIT_MODE, permanent_id: 39, name: "VTX PIT MODE" },
        RcModeName { id: Self::USER1, permanent_id: 40, name: "USER1" }, // may be overridden
        RcModeName { id: Self::USER2, permanent_id: 41, name: "USER2" },
        RcModeName { id: Self::USER3, permanent_id: 42, name: "USER3" },
        RcModeName { id: Self::USER4, permanent_id: 43, name: "USER4" },
        RcModeName { id: Self::PID_AUDIO, permanent_id: 44, name: "PID AUDIO" },
        RcModeName { id: Self::PARALYZE, permanent_id: 45, name: "PARALYZE" },
        RcModeName { id: Self::GPS_RESCUE, permanent_id: 46, name: "GPS RESCUE" },
        RcModeName { id: Self::ACRO_TRAINER, permanent_id: 47, name: "ACRO TRAINER" },
        RcModeName { id: Self::VTX_CONTROL_DISABLE, permanent_id: 48, name: "VTX CONTROL DISABLE" },
        RcModeName { id: Self::LAUNCH_CONTROL, permanent_id: 49, name: "LAUNCH CONTROL" },
        RcModeName { id: Self::MSP_OVERRIDE, permanent_id: 50, name: "MSP OVERRIDE" },
        RcModeName { id: Self::STICK_COMMAND_DISABLE, permanent_id: 51, name: "STICK COMMANDS DISABLE" },
        RcModeName { id: Self::BEEPER_MUTE, permanent_id: 52, name: "BEEPER MUTE" },
        RcModeName { id: Self::READY, permanent_id: 53, name: "READY" },
        RcModeName { id: Self::LAP_TIMER_RESET, permanent_id: 54, name: "LAP TIMER RESET" },
    ];
}

#[cfg(test)]
mod tests {
    use super::*;

    fn is_normal<T: Sized + Send + Sync + Unpin>() {}
    fn is_full<T: Sized + Send + Sync + Unpin + Copy + Clone + Default + PartialEq>() {}

    #[test]
    fn normal_types() {
        is_full::<RxChannelRange>();
        is_full::<ModeActivationCondition>();
        is_full::<RcModes>();
        is_normal::<RcMode>();
        is_normal::<RcModeName>();
    }
    #[test]
    fn new() {
        let rc_modes = RcModes::default();
        assert_eq!(0, rc_modes.active_mac_count);
    }
}
