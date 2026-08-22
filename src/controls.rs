use crate::{RxChannel, RxFrame};

/// Control values from receiver scaled to the range `[-1.0, 1.0]`.
#[derive(Clone, Copy, Debug, PartialEq)]
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
            roll: (f32::from(frame.channels[RxChannel::ROLL]) - RxChannel::MID_F32) / RxChannel::HALF_RANGE_F32,
            pitch: (f32::from(frame.channels[RxChannel::PITCH]) - RxChannel::MID_F32) / RxChannel::HALF_RANGE_F32,
            yaw: (f32::from(frame.channels[RxChannel::YAW]) - RxChannel::MID_F32) / RxChannel::HALF_RANGE_F32,
            throttle: (f32::from(frame.channels[RxChannel::THROTTLE]) - RxChannel::LOW_F32) / RxChannel::RANGE_F32,
        }
    }
}

impl From<RxControlsPwm> for RcSticks {
    fn from(controls_pwm: RxControlsPwm) -> Self {
        // Map channels in range [1000,2000] to floats in range [0,1] for throttle, [-1,1] for roll, pitch yaw
        RcSticks {
            roll: (f32::from(controls_pwm.roll) - RxChannel::MID_F32 / RxChannel::HALF_RANGE_F32),
            pitch: (f32::from(controls_pwm.pitch) - RxChannel::MID_F32 / RxChannel::HALF_RANGE_F32),
            yaw: (f32::from(controls_pwm.yaw) - RxChannel::MID_F32 / RxChannel::HALF_RANGE_F32),
            throttle: (f32::from(controls_pwm.throttle) - RxChannel::LOW_F32 / RxChannel::RANGE_F32),
        }
    }
}

impl Default for RcSticks {
    fn default() -> Self {
        Self::new()
    }
}

impl RcSticks {
    /// Constructor.
    #[must_use]
    pub const fn new() -> Self {
        Self { roll: 0.0, pitch: 0.0, yaw: 0.0, throttle: 0.0 }
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
    // Constructor.
    #[must_use]
    pub const fn new() -> Self {
        Self { throttle: RxChannel::LOW, roll: RxChannel::MID, pitch: RxChannel::MID, yaw: RxChannel::MID }
    }
}

impl RxControlsPwm {
    // course gained values of pwm. Can be used to allow the radio act like cursor keys to navigate a menu system
    #[must_use]
    pub fn pwm_is_high(pwm: u16) -> bool {
        pwm >= 1750
    }
    #[must_use]
    pub fn pwm_is_low(pwm: u16) -> bool {
        pwm <= 1250
    }
    #[must_use]
    pub fn pwm_is_mid(pwm: u16) -> bool {
        pwm > 1250 && pwm < 1750
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::float_cmp)]
    use super::*;

    fn _is_normal<T: Sized + Send + Sync + Unpin>() {}
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
    #[test]
    fn from_rx_frame() {
        let mut rx_frame = RxFrame::new();
        rx_frame.channels[RxChannel::ROLL] = 1250;
        rx_frame.channels[RxChannel::PITCH] = 1500;
        rx_frame.channels[RxChannel::YAW] = 1750;
        rx_frame.channels[RxChannel::THROTTLE] = 1000;

        // maps [1000, 2000] to [-1.0, 1.0] for roll, pitch, yaw, [0.0, 1.0] for throttle
        let rc_sticks = RcSticks::from(rx_frame);
        assert_eq!(-0.5, rc_sticks.roll);
        assert_eq!(0.0, rc_sticks.pitch);
        assert_eq!(0.5, rc_sticks.yaw);
        assert_eq!(0.0, rc_sticks.throttle);

        rx_frame.channels[RxChannel::THROTTLE] = 1250;
        let rc_sticks = RcSticks::from(rx_frame);
        assert_eq!(0.25, rc_sticks.throttle);
        rx_frame.channels[RxChannel::THROTTLE] = 1750;
        let rc_sticks = RcSticks::from(rx_frame);
        assert_eq!(0.75, rc_sticks.throttle);
    }
}
