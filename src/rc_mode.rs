#[cfg(feature = "storage")]
use sequential_storage::map::PostcardValue;
#[cfg(feature = "serde")]
use {
    postcard::experimental::max_size::MaxSize,
    serde::{Deserialize, Serialize},
};

/// Betaflight compatible RC modes.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize, MaxSize))]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum RcMode {
    /// Arming flag.
    #[default]
    Arm = 0,

    // Flight mode flags
    Angle = 1,
    Horizon = 2,
    Mag = 3,
    AltitudeHold = 4,
    PositionHold = 5,
    Headfree = 6,
    Chirp = 7,
    Passthru = 8,
    Failsafe = 9,
    GpsRescue = 10,
    Autopilot = 11, // GPS path following

    // RC mode flags
    Antigravity = 12,
    HeadAdj = 13,
    CameraStabilization = 14,
    BeeperOn = 15,
    LedLow = 16,
    Calibrate = 17,
    Osd = 18,
    Telemetry = 19,
    Servo1 = 20,
    Servo2 = 21,
    Servo3 = 22,
    Blackbox = 23,
    Airmode = 24,
    Mode3d = 25,
    FpvAngleMix = 26,
    BlackboxErase = 27,
    Camera1 = 28,
    Camera2 = 29,
    Camera3 = 30,
    CrashFlip = 31,
    Prearm = 32,
    BeepGpsCount = 33,
    VtxPitMode = 34,
    Paralyze = 35,
    User1 = 36,
    User2 = 37,
    User3 = 38,
    User4 = 39,
    PidAudio = 40,
    AcroTrainer = 41,
    VtxControlDisable = 42,
    LaunchControl = 43,
    MspOverride = 44,
    StickCommandDisable = 45,
    BeeperMute = 46,
    Ready = 47,
    LapTimerReset = 48,
}

impl RcMode {
    pub const COUNT: u8 = 49; // NOTE: this must be one more that the value of the last item in `RcMode`.
    pub const FLIGHTMODE_COUNT: u8 = 12;

    // RcMode values as u8, for convenience.
    pub const ARM: u8 = Self::Arm as u8;
    pub const ANGLE: u8 = Self::Angle as u8;
    pub const HORIZON: u8 = Self::Horizon as u8;
    pub const MAG: u8 = Self::Mag as u8;
    pub const ALTITUDE_HOLD: u8 = Self::AltitudeHold as u8;
    pub const POSITION_HOLD: u8 = Self::PositionHold as u8;
    pub const HEADFREE: u8 = Self::Headfree as u8;
    pub const CHIRP: u8 = Self::Chirp as u8;
    pub const PASSTHRU: u8 = Self::Passthru as u8;
    pub const FAILSAFE: u8 = Self::Failsafe as u8;
    pub const GPS_RESCUE: u8 = Self::GpsRescue as u8;
    pub const AUTOPILOT: u8 = Self::Autopilot as u8;

    pub const ANTIGRAVITY: u8 = Self::Antigravity as u8;
    pub const HEADADJ: u8 = Self::HeadAdj as u8;
    pub const CAMERA_STABILIZATION: u8 = Self::CameraStabilization as u8;
    pub const BEEPER_ON: u8 = Self::BeeperOn as u8;
    pub const LED_LOW: u8 = Self::LedLow as u8;
    pub const CALIBRATE: u8 = Self::Calibrate as u8;
    pub const OSD: u8 = Self::Osd as u8;
    pub const TELEMETRY: u8 = Self::Telemetry as u8;
    pub const SERVO1: u8 = Self::Servo1 as u8;
    pub const SERVO2: u8 = Self::Servo2 as u8;
    pub const SERVO3: u8 = Self::Servo3 as u8;
    pub const BLACKBOX: u8 = Self::Blackbox as u8;
    pub const AIRMODE: u8 = Self::Airmode as u8;
    pub const MODE_3E: u8 = Self::Mode3d as u8;
    pub const FPV_ANGLE_MIX: u8 = Self::FpvAngleMix as u8;
    pub const BLACKBOX_ERASE: u8 = Self::BlackboxErase as u8;
    pub const CAMERA1: u8 = Self::Camera1 as u8;
    pub const CAMERA2: u8 = Self::Camera2 as u8;
    pub const CAMERA3: u8 = Self::Camera3 as u8;
    pub const CRASH_FLIP: u8 = Self::CrashFlip as u8;
    pub const PRE_ARM: u8 = Self::Prearm as u8;
    pub const BEEP_GPS_COUNT: u8 = Self::BeepGpsCount as u8;
    pub const VTX_PIT_MODE: u8 = Self::VtxPitMode as u8;
    pub const PARALYZE: u8 = Self::Paralyze as u8;
    pub const USER1: u8 = Self::User1 as u8;
    pub const USER2: u8 = Self::User2 as u8;
    pub const USER3: u8 = Self::User3 as u8;
    pub const USER4: u8 = Self::User4 as u8;
    pub const PID_AUDIO: u8 = Self::PidAudio as u8;
    pub const ACRO_TRAINER: u8 = Self::AcroTrainer as u8;
    pub const VTX_CONTROL_DISABLE: u8 = Self::VtxControlDisable as u8;
    pub const LAUNCH_CONTROL: u8 = Self::LaunchControl as u8;
    pub const MPS_OVERRIDE: u8 = Self::MspOverride as u8;
    pub const STICK_COMMAND_DISABLE: u8 = Self::StickCommandDisable as u8;
    pub const BEEPER_MUTE: u8 = Self::BeeperMute as u8;
    pub const READY: u8 = Self::Ready as u8;
    pub const LAP_TIMER_RESET: u8 = Self::LapTimerReset as u8;
}

#[cfg(feature = "storage")]
impl PostcardValue<'_> for RcMode {}

impl_try_from_u8!(RcMode);

impl RcMode {
    #[must_use]
    pub fn from_u8(value: u8) -> Self {
        match value {
            0 => Self::Arm,

            // Flight mode flags
            1 => Self::Angle,
            2 => Self::Horizon,
            3 => Self::Mag,
            4 => Self::AltitudeHold,
            5 => Self::PositionHold,
            6 => Self::Headfree,
            7 => Self::Chirp,
            8 => Self::Passthru,
            9 => Self::Failsafe,
            10 => Self::GpsRescue,
            11 => Self::Autopilot, // GPS path following

            // RC mode flags
            12 => Self::Antigravity,
            13 => Self::HeadAdj,
            14 => Self::CameraStabilization,
            15 => Self::BeeperOn,
            16 => Self::LedLow,
            17 => Self::Calibrate,
            18 => Self::Osd,
            19 => Self::Telemetry,
            20 => Self::Servo1,
            21 => Self::Servo2,
            22 => Self::Servo3,
            23 => Self::Blackbox,
            24 => Self::Airmode,
            25 => Self::Mode3d,
            26 => Self::FpvAngleMix,
            27 => Self::BlackboxErase,
            28 => Self::Camera1,
            29 => Self::Camera2,
            30 => Self::Camera3,
            31 => Self::CrashFlip,
            32 => Self::Prearm,
            33 => Self::BeepGpsCount,
            34 => Self::VtxPitMode,
            35 => Self::Paralyze,
            36 => Self::User1,
            37 => Self::User2,
            38 => Self::User3,
            39 => Self::User4,
            40 => Self::PidAudio,
            41 => Self::AcroTrainer,
            42 => Self::VtxControlDisable,
            43 => Self::LaunchControl,
            44 => Self::MspOverride,
            45 => Self::StickCommandDisable,
            46 => Self::BeeperMute,
            47 => Self::Ready,
            48 => Self::LapTimerReset,
            _ => Self::default(),
        }
    }
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize, MaxSize))]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum RcModeLogic {
    //const LOGIC_OR: u8 = 0;
    //const LOGIC_AND: u8 = 1;
    #[default]
    Or = 0,
    And = 1,
}
#[cfg(feature = "storage")]
impl PostcardValue<'_> for RcModeLogic {}

impl_try_from_u8!(RcModeLogic);

impl RcModeLogic {
    #[must_use]
    pub fn from_u8(value: u8) -> Self {
        match value {
            0 => Self::Or,

            // Flight mode flags
            1 => Self::And,
            _ => Self::default(),
        }
    }
}

#[allow(unused)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct RcModeDescriptor {
    pub id: RcMode,
    pub permanent_id: u8,
    pub name: &'static str,
}

#[allow(unused)]
impl RcModeDescriptor {
    #[must_use]
    pub fn find_rc_mode_by_id(id: RcMode) -> Option<RcModeDescriptor> {
        Self::RC_MODES.into_iter().find(|&mode_name| id == mode_name.id)
    }
    #[must_use]
    pub fn find_rc_mode_by_permanent_id(id: u8) -> Option<RcModeDescriptor> {
        Self::RC_MODES.into_iter().find(|&mode_name| id == mode_name.permanent_id)
    }
}

#[allow(unused)]
#[allow(missing_docs)]
impl RcModeDescriptor {
    pub const COUNT: usize = RcMode::COUNT as usize;
    pub const MAX_MODES_PER_PAGE: u8 = 32;
    pub const PERMANENT_ID_NONE: u8 = 255;

    // `permanent_id`s must uniquely identify `RcMode`, DO NOT REUSE THEM!
    pub const RC_MODES: [RcModeDescriptor; Self::COUNT] = [
        RcModeDescriptor { id: RcMode::Arm, permanent_id: 0, name: "ARM" },
        RcModeDescriptor { id: RcMode::Angle, permanent_id: 1, name: "ANGLE" },
        RcModeDescriptor { id: RcMode::Horizon, permanent_id: 2, name: "HORIZON" },
        RcModeDescriptor { id: RcMode::AltitudeHold, permanent_id: 3, name: "ALTHOLD" },
        RcModeDescriptor { id: RcMode::Antigravity, permanent_id: 4, name: "ANTI GRAVITY" },
        RcModeDescriptor { id: RcMode::Mag, permanent_id: 5, name: "MAG" },
        RcModeDescriptor { id: RcMode::Headfree, permanent_id: 6, name: "HEADFREE" },
        RcModeDescriptor { id: RcMode::HeadAdj, permanent_id: 7, name: "HEADADJ" },
        RcModeDescriptor { id: RcMode::CameraStabilization, permanent_id: 8, name: "CAMSTAB" },
        // RcModeStruct { id: RcMode::CAM_TRIG, permanent_id: 9,  name:"CAM_TRIG", }, // (removed)
        // RcModeStruct { id: RcMode::GPS_HOME, permanent_id: 10, name:"GPS HOME" }, // (removed)
        RcModeDescriptor { id: RcMode::PositionHold, permanent_id: 11, name: "POS HOLD" },
        RcModeDescriptor { id: RcMode::Passthru, permanent_id: 12, name: "PASSTHRU" },
        RcModeDescriptor { id: RcMode::BeeperOn, permanent_id: 13, name: "BEEPER" },
        // RcModeStruct { id: RcMode::LEDMAX, permanent_id:14, name:"LEDMAX" }, // (removed)
        RcModeDescriptor { id: RcMode::LedLow, permanent_id: 15, name: "LEDLOW" },
        // RcModeStruct { id: RcMode::LLIGHTS, permanent_id:16, name:"LLIGHTS" }, // (removed)
        RcModeDescriptor { id: RcMode::Calibrate, permanent_id: 17, name: "CALIBRATE" },
        // RcModeStruct { id: RcMode::GOVERNOR, permanent_id: 18, name:"GOVERNOR" }, // (removed)
        RcModeDescriptor { id: RcMode::Osd, permanent_id: 19, name: "OSD DISABLE" },
        RcModeDescriptor { id: RcMode::Telemetry, permanent_id: 20, name: "TELEMETRY" },
        // RcModeStruct { id: RcMode::GTUNE, permanent_id: 21, name: "GTUNE" }, // (removed)
        // RcModeStruct { id: RcMode::RANGEFINDER, permanent_id: 22, name: "RANGEFINDER" }, // (removed)
        RcModeDescriptor { id: RcMode::Servo1, permanent_id: 23, name: "SERVO1" },
        RcModeDescriptor { id: RcMode::Servo2, permanent_id: 24, name: "SERVO2" },
        RcModeDescriptor { id: RcMode::Servo3, permanent_id: 25, name: "SERVO3" },
        RcModeDescriptor { id: RcMode::Blackbox, permanent_id: 26, name: "BLACK" },
        RcModeDescriptor { id: RcMode::Failsafe, permanent_id: 27, name: "FAILSAFE" },
        RcModeDescriptor { id: RcMode::Airmode, permanent_id: 28, name: "AIR MODE" },
        RcModeDescriptor { id: RcMode::Mode3d, permanent_id: 29, name: "3D DISABLE / SWITCH" },
        RcModeDescriptor { id: RcMode::FpvAngleMix, permanent_id: 30, name: "FPV ANGLE MIX" },
        RcModeDescriptor { id: RcMode::BlackboxErase, permanent_id: 31, name: "BLACK ERASE" },
        RcModeDescriptor { id: RcMode::Camera1, permanent_id: 32, name: "CAMERA CONTROL 1" },
        RcModeDescriptor { id: RcMode::Camera2, permanent_id: 33, name: "CAMERA CONTROL 2" },
        RcModeDescriptor { id: RcMode::Camera3, permanent_id: 34, name: "CAMERA CONTROL 3" },
        RcModeDescriptor { id: RcMode::CrashFlip, permanent_id: 35, name: "FLIP OVER AFTER CRASH" },
        RcModeDescriptor { id: RcMode::Prearm, permanent_id: 36, name: "PREARM" },
        RcModeDescriptor { id: RcMode::BeepGpsCount, permanent_id: 37, name: "GPS BEEP SATELLITE COUNT" },
        // RcModeStruct { id: RcMode::BOX3D_ON_A_SWITCH, permanent_id: 38, name: "3D ON A SWITCH", }, // (removed)
        RcModeDescriptor { id: RcMode::VtxPitMode, permanent_id: 39, name: "VTX PIT MODE" },
        RcModeDescriptor { id: RcMode::User1, permanent_id: 40, name: "USER1" }, // may be overridden
        RcModeDescriptor { id: RcMode::User2, permanent_id: 41, name: "USER2" },
        RcModeDescriptor { id: RcMode::User3, permanent_id: 42, name: "USER3" },
        RcModeDescriptor { id: RcMode::User4, permanent_id: 43, name: "USER4" },
        RcModeDescriptor { id: RcMode::PidAudio, permanent_id: 44, name: "PID AUDIO" },
        RcModeDescriptor { id: RcMode::Paralyze, permanent_id: 45, name: "PARALYZE" },
        RcModeDescriptor { id: RcMode::GpsRescue, permanent_id: 46, name: "GPS RESCUE" },
        RcModeDescriptor { id: RcMode::AcroTrainer, permanent_id: 47, name: "ACRO TRAINER" },
        RcModeDescriptor { id: RcMode::VtxControlDisable, permanent_id: 48, name: "VTX CONTROL DISABLE" },
        RcModeDescriptor { id: RcMode::LaunchControl, permanent_id: 49, name: "LAUNCH CONTROL" },
        RcModeDescriptor { id: RcMode::MspOverride, permanent_id: 50, name: "MSP OVERRIDE" },
        RcModeDescriptor { id: RcMode::StickCommandDisable, permanent_id: 51, name: "STICK COMMANDS DISABLE" },
        RcModeDescriptor { id: RcMode::BeeperMute, permanent_id: 52, name: "BEEPER MUTE" },
        RcModeDescriptor { id: RcMode::Ready, permanent_id: 53, name: "READY" },
        RcModeDescriptor { id: RcMode::LapTimerReset, permanent_id: 54, name: "LAP TIMER RESET" },
        RcModeDescriptor { id: RcMode::Chirp, permanent_id: 55, name: "CHIRP" },
        RcModeDescriptor { id: RcMode::Autopilot, permanent_id: 56, name: "AUTOPILOT" },
    ];
}

#[cfg(test)]
mod tests {
    use super::*;

    fn is_full<T: Sized + Send + Sync + Unpin + Copy + Clone + Default + PartialEq>() {}
    #[cfg(feature = "serde")]
    fn is_serde<T: Serialize + MaxSize + for<'a> Deserialize<'a>>() {}
    #[cfg(feature = "storage")]
    fn is_storage<T: for<'a> PostcardValue<'a>>() {}

    #[test]
    fn normal_types() {
        is_full::<RcMode>();
        is_full::<RcModeLogic>();
        #[cfg(feature = "serde")]
        is_serde::<RcMode>();
        #[cfg(feature = "storage")]
        is_storage::<RcMode>();
        #[cfg(feature = "serde")]
        is_serde::<RcModeLogic>();
        #[cfg(feature = "storage")]
        is_storage::<RcModeLogic>();
    }
}
