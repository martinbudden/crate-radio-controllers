use crate::{RxChannel, RxFrame};

/// Control values from receiver scaled to the range `[-1.0, 1.0]`.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct RcSticks {
    pub roll: f32,
    pub pitch: f32,
    pub yaw: f32,
    pub throttle: f32,
}

impl From<RxFrame> for RcSticks {
    fn from(frame: RxFrame) -> Self {
        // Map channels in range [1000,2000] to floats in range [0,1] for throttle, [-1,1] for roll, pitch yaw
        RcSticks {
            roll: (f32::from(frame.channels[RxChannel::ROLL] - RxChannel::MID) / f32::from(RxChannel::RANGE)),
            pitch: (f32::from(frame.channels[RxChannel::PITCH] - RxChannel::MID) / f32::from(RxChannel::RANGE)),
            yaw: (f32::from(frame.channels[RxChannel::YAW] - RxChannel::MID) / f32::from(RxChannel::RANGE)),
            throttle: (f32::from(frame.channels[RxChannel::THROTTLE] - RxChannel::RANGE) / f32::from(RxChannel::RANGE)),
        }
    }
}

impl From<RxControlsPwm> for RcSticks {
    fn from(controls_pwm: RxControlsPwm) -> Self {
        // Map channels in range [1000,2000] to floats in range [0,1] for throttle, [-1,1] for roll, pitch yaw
        RcSticks {
            roll: (f32::from(controls_pwm.roll - RxChannel::MID) / f32::from(RxChannel::RANGE)),
            pitch: (f32::from(controls_pwm.pitch - RxChannel::MID) / f32::from(RxChannel::RANGE)),
            yaw: (f32::from(controls_pwm.yaw - RxChannel::MID) / f32::from(RxChannel::RANGE)),
            throttle: (f32::from(controls_pwm.throttle - RxChannel::RANGE) / f32::from(RxChannel::RANGE)),
        }
    }
}

impl RcSticks {
    pub fn new() -> Self {
        Self::default()
    }
}

/// Controls values from receiver in the Pulse Width Modulation (PWM) range, nominally `[1000, 2000]`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RxControlsPwm {
    pub roll: u16,
    pub pitch: u16,
    pub yaw: u16,
    pub throttle: u16,
}

impl Default for RxControlsPwm {
    fn default() -> Self {
        Self::new()
    }
}

impl RxControlsPwm {
    pub fn new() -> Self {
        Self { throttle: RxChannel::LOW, roll: RxChannel::MID, pitch: RxChannel::MID, yaw: RxChannel::MID }
    }
}

impl RxControlsPwm {
    // course gained values of pwm. Can be used to allow the receiver act like cursor keys to navigate a menu system
    pub fn pwm_is_high(pwm: u16) -> bool {
        pwm >= 1750
    }
    pub fn pwm_is_low(pwm: u16) -> bool {
        pwm <= 1250
    }
    pub fn pwm_is_mid(pwm: u16) -> bool {
        pwm > 1250 && pwm < 1750
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::float_cmp)]
    use super::*;

    #[allow(unused)]
    fn is_normal<T: Sized + Send + Sync + Unpin>() {}
    fn is_full<T: Sized + Send + Sync + Unpin + Copy + Clone + Default + PartialEq>() {}

    #[test]
    fn normal_types() {
        is_full::<RcSticks>();
        is_full::<RxControlsPwm>();
    }
    #[test]
    fn new() {
        let controls = RcSticks::new();
        assert_eq!(0.0, controls.throttle);
    }
}
