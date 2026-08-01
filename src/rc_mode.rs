#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct RcMode {
    pub id: u8,
    pub permanent_id: u8,
    pub name: &'static str,
}

impl RcMode {
    #[must_use]
    pub fn find_rc_mode_by_id(id: u8) -> Option<RcMode> {
        Self::RC_MODES.into_iter().find(|&mode_name| id == mode_name.id)
    }
    #[must_use]
    pub fn find_rc_mode_by_permanent_id(id: u8) -> Option<RcMode> {
        Self::RC_MODES.into_iter().find(|&mode_name| id == mode_name.permanent_id)
    }
}

#[allow(missing_docs)]
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
    pub const AUTOPILOT: u8 = 11; // GPS path following
    pub const FLIGHTMODE_COUNT: u8 = 12;

    // RC mode flags
    pub const ANTIGRAVITY: u8 = Self::FLIGHTMODE_COUNT;
    pub const HEADADJ: u8 = 13;
    pub const CAMSTAB: u8 = 14;
    pub const BEEPER_ON: u8 = 15;
    pub const LED_LOW: u8 = 16;
    pub const CALIBRATE: u8 = 17;
    pub const OSD: u8 = 18;
    pub const TELEMETRY: u8 = 19;
    pub const SERVO1: u8 = 20;
    pub const SERVO2: u8 = 21;
    pub const SERVO3: u8 = 22;
    pub const BLACKBOX: u8 = 23;
    pub const AIRMODE: u8 = 24;
    pub const MODE_3D: u8 = 25;
    pub const FPV_ANGLE_MIX: u8 = 26;
    pub const BLACKBOX_ERASE: u8 = 27;
    pub const CAMERA1: u8 = 28;
    pub const CAMERA2: u8 = 29;
    pub const CAMERA3: u8 = 30;
    pub const CRASH_FLIP: u8 = 31;
    pub const PREARM: u8 = 32;
    pub const BEEP_GPS_COUNT: u8 = 33;
    pub const VTX_PIT_MODE: u8 = 34;
    pub const PARALYZE: u8 = 35;
    pub const USER1: u8 = 36;
    pub const USER2: u8 = 37;
    pub const USER3: u8 = 38;
    pub const USER4: u8 = 39;
    pub const PID_AUDIO: u8 = 40;
    pub const ACRO_TRAINER: u8 = 41;
    pub const VTX_CONTROL_DISABLE: u8 = 42;
    pub const LAUNCH_CONTROL: u8 = 43;
    pub const MSP_OVERRIDE: u8 = 44;
    pub const STICK_COMMAND_DISABLE: u8 = 45;
    pub const BEEPER_MUTE: u8 = 46;
    pub const READY: u8 = 47;
    pub const LAP_TIMER_RESET: u8 = 48;
    pub const COUNT: u8 = 49;

    // `permanent_id`s must uniquely identify `RcMode`, DO NOT REUSE THEM!
    pub const RC_MODES: [RcMode; Self::COUNT as usize] = [
        RcMode { id: Self::ARM, permanent_id: 0, name: "ARM" },
        RcMode { id: Self::ANGLE, permanent_id: 1, name: "ANGLE" },
        RcMode { id: Self::HORIZON, permanent_id: 2, name: "HORIZON" },
        RcMode { id: Self::ALTITUDE_HOLD, permanent_id: 3, name: "ALTHOLD" },
        RcMode { id: Self::ANTIGRAVITY, permanent_id: 4, name: "ANTI GRAVITY" },
        RcMode { id: Self::MAG, permanent_id: 5, name: "MAG" },
        RcMode { id: Self::HEADFREE, permanent_id: 6, name: "HEADFREE" },
        RcMode { id: Self::HEADADJ, permanent_id: 7, name: "HEADADJ" },
        RcMode { id: Self::CAMSTAB, permanent_id: 8, name: "CAMSTAB" },
        // RcMode { id: Self::CAM_TRIG, permanent_id: 9,  name:"CAM_TRIG", }, // (removed)
        // RcMode { id: Self::GPS_HOME, permanent_id: 10, name:"GPS HOME" }, // (removed)
        RcMode { id: Self::POSITION_HOLD, permanent_id: 11, name: "POS HOLD" },
        RcMode { id: Self::PASSTHRU, permanent_id: 12, name: "PASSTHRU" },
        RcMode { id: Self::BEEPER_ON, permanent_id: 13, name: "BEEPER" },
        // RcMode { id: Self::LEDMAX, permanent_id:14, name:"LEDMAX" }, // (removed)
        RcMode { id: Self::LED_LOW, permanent_id: 15, name: "LEDLOW" },
        // RcMode { id: Self::LLIGHTS, permanent_id:16, name:"LLIGHTS" }, // (removed)
        RcMode { id: Self::CALIBRATE, permanent_id: 17, name: "CALIBRATE" },
        // RcMode { id: Self::GOVERNOR, permanent_id: 18, name:"GOVERNOR" }, // (removed)
        RcMode { id: Self::OSD, permanent_id: 19, name: "OSD DISABLE" },
        RcMode { id: Self::TELEMETRY, permanent_id: 20, name: "TELEMETRY" },
        // RcMode { id: Self::GTUNE, permanent_id: 21, name: "GTUNE" }, // (removed)
        // RcMode { id: Self::RANGEFINDER, permanent_id: 22, name: "RANGEFINDER" }, // (removed)
        RcMode { id: Self::SERVO1, permanent_id: 23, name: "SERVO1" },
        RcMode { id: Self::SERVO2, permanent_id: 24, name: "SERVO2" },
        RcMode { id: Self::SERVO3, permanent_id: 25, name: "SERVO3" },
        RcMode { id: Self::BLACKBOX, permanent_id: 26, name: "BLACK" },
        RcMode { id: Self::FAILSAFE, permanent_id: 27, name: "FAILSAFE" },
        RcMode { id: Self::AIRMODE, permanent_id: 28, name: "AIR MODE" },
        RcMode { id: Self::MODE_3D, permanent_id: 29, name: "3D DISABLE / SWITCH" },
        RcMode { id: Self::FPV_ANGLE_MIX, permanent_id: 30, name: "FPV ANGLE MIX" },
        RcMode { id: Self::BLACKBOX_ERASE, permanent_id: 31, name: "BLACK ERASE" },
        RcMode { id: Self::CAMERA1, permanent_id: 32, name: "CAMERA CONTROL 1" },
        RcMode { id: Self::CAMERA2, permanent_id: 33, name: "CAMERA CONTROL 2" },
        RcMode { id: Self::CAMERA3, permanent_id: 34, name: "CAMERA CONTROL 3" },
        RcMode { id: Self::CRASH_FLIP, permanent_id: 35, name: "FLIP OVER AFTER CRASH" },
        RcMode { id: Self::PREARM, permanent_id: 36, name: "PREARM" },
        RcMode { id: Self::BEEP_GPS_COUNT, permanent_id: 37, name: "GPS BEEP SATELLITE COUNT" },
        // RcMode { id: Self::BOX3D_ON_A_SWITCH, permanent_id: 38, name: "3D ON A SWITCH", }, // (removed)
        RcMode { id: Self::VTX_PIT_MODE, permanent_id: 39, name: "VTX PIT MODE" },
        RcMode { id: Self::USER1, permanent_id: 40, name: "USER1" }, // may be overridden
        RcMode { id: Self::USER2, permanent_id: 41, name: "USER2" },
        RcMode { id: Self::USER3, permanent_id: 42, name: "USER3" },
        RcMode { id: Self::USER4, permanent_id: 43, name: "USER4" },
        RcMode { id: Self::PID_AUDIO, permanent_id: 44, name: "PID AUDIO" },
        RcMode { id: Self::PARALYZE, permanent_id: 45, name: "PARALYZE" },
        RcMode { id: Self::GPS_RESCUE, permanent_id: 46, name: "GPS RESCUE" },
        RcMode { id: Self::ACRO_TRAINER, permanent_id: 47, name: "ACRO TRAINER" },
        RcMode { id: Self::VTX_CONTROL_DISABLE, permanent_id: 48, name: "VTX CONTROL DISABLE" },
        RcMode { id: Self::LAUNCH_CONTROL, permanent_id: 49, name: "LAUNCH CONTROL" },
        RcMode { id: Self::MSP_OVERRIDE, permanent_id: 50, name: "MSP OVERRIDE" },
        RcMode { id: Self::STICK_COMMAND_DISABLE, permanent_id: 51, name: "STICK COMMANDS DISABLE" },
        RcMode { id: Self::BEEPER_MUTE, permanent_id: 52, name: "BEEPER MUTE" },
        RcMode { id: Self::READY, permanent_id: 53, name: "READY" },
        RcMode { id: Self::LAP_TIMER_RESET, permanent_id: 54, name: "LAP TIMER RESET" },
        RcMode { id: Self::CHIRP, permanent_id: 55, name: "CHIRP" },
        RcMode { id: Self::AUTOPILOT, permanent_id: 56, name: "AUTOPILOT" },
    ];
}

#[cfg(test)]
mod tests {
    use super::*;

    fn _is_normal<T: Sized + Send + Sync + Unpin>() {}
    fn is_full<T: Sized + Send + Sync + Unpin + Copy + Clone + Default + PartialEq>() {}

    #[test]
    fn normal_types() {
        is_full::<RcMode>();
    }
}
