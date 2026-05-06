#[allow(unused)]
use crate::{Rates, RcModes, RcSticks, RxFrame};
use vqm::BitSet64;

/// Message for communicating a radio control command between tasks.<br><br>
#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(C)]
pub struct RadioControlMessage {
    pub rc_modes: BitSet64,
    pub tick_count: u32,
    pub throttle_stick: f32,
    pub roll_stick_dps: f32,
    pub pitch_stick_dps: f32,
    pub yaw_stick_dps: f32,
    pub roll_stick_degrees: f32,
    pub pitch_stick_degrees: f32,
    pub stabilization_mode: u8,
    pub failsafe: u8,
}
const _: () = assert!(core::mem::size_of::<RadioControlMessage>() == 40);

impl Default for RadioControlMessage {
    fn default() -> Self {
        Self::new()
    }
}

impl RadioControlMessage {
    pub const STABILIZATION_MODE_RATE: u8 = 0; // aka acro mode
    pub const STABILIZATION_MODE_ANGLE: u8 = 1;
    pub const STABILIZATION_MODE_HORIZON: u8 = 2;
    pub const STABILIZATION_MODE_LEVEL_RACE: u8 = 3;

    pub const fn new() -> Self {
        Self {
            rc_modes: BitSet64::new(),
            tick_count: 0,
            throttle_stick: 0.0,
            roll_stick_dps: 0.0,
            pitch_stick_dps: 0.0,
            yaw_stick_dps: 0.0,
            roll_stick_degrees: 0.0,
            pitch_stick_degrees: 0.0,
            stabilization_mode: 0,
            failsafe: 0,
        }
    }
}

impl RadioControlMessage {
    /// Create a `RadioControlMessage` from an `RxFrame`, applying rates and including `RcModes`.
    pub fn from_rx_frame(
        rx_frame: &RxFrame,
        rates: &Rates,
        rc_modes: &RcModes,
        tick_count: u32,
        failsafe: u8,
    ) -> RadioControlMessage {
        // get the stick values from the rx_frame.
        let sticks = RcSticks::from(*rx_frame);

        // apply rates to the stick values.
        let roll_stick_dps = rates.apply(Rates::ROLL, sticks.roll);
        let pitch_stick_dps = rates.apply(Rates::PITCH, sticks.pitch);
        let yaw_stick_dps = rates.apply(Rates::YAW, sticks.yaw);

        // scale the stick angles.
        let roll_stick_degrees = sticks.roll * rates.max_roll_angle_degrees;
        let pitch_stick_degrees = sticks.pitch * rates.max_pitch_angle_degrees;

        // Update rc_modes from the frame that has just come in from the radio.
        //rc_modes.update_activated_modes(rx_frame);

        // Set the stabilization mode (eg STABILIZATION_MODE_RATE) (used by the flight controller),
        // Set the rc_modes (eg altitude hold, gps home) (used by the autopilot).
        let (rc_modes, stabilization_mode) = rc_modes.update_modes();

        RadioControlMessage {
            rc_modes,
            tick_count,

            throttle_stick: sticks.throttle,
            roll_stick_dps,
            pitch_stick_dps,
            yaw_stick_dps,
            roll_stick_degrees,
            pitch_stick_degrees,

            stabilization_mode,
            failsafe,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[allow(unused)]
    fn is_normal<T: Sized + Send + Sync + Unpin>() {}
    fn is_full<T: Sized + Send + Sync + Unpin + Copy + Clone + Default + PartialEq>() {}

    #[test]
    fn normal_types() {
        is_full::<RadioControlMessage>();
    }
    #[test]
    fn sizeof() {
        assert_eq!(40, core::mem::size_of::<RadioControlMessage>());
    }
}
