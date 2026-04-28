use serde::{Deserialize, Serialize};

/// Configuration of failsafe behavior.<br><br>
#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
pub struct FailsafeConfig {
    pub throttle_pwm: u16,
    pub throttle_low_delay_deciseconds: u16,
    pub recovery_delay_deciseconds: u16, // time of valid rx data needed to allow recovery from failsafe and re-arming
    pub delay_deciseconds: u8,
    pub landing_time_seconds: u8, // time allowed in landing phase before disarm
    pub procedure: u8,
    pub switch_mode: u8,
    pub stick_threshold_percent: u8, // _stick deflection percentage to exit GPS Rescue procedure
}

impl FailsafeConfig {
    pub const DISARMED: u8 = 0;
    pub const IDLE: u8 = 1;
    pub const RX_LOSS_DETECTED: u8 = 2;
    pub const RX_LOSS_MONITORING: u8 = 3;
    pub const RX_LOSS_RECOVERED: u8 = 4;
    pub const LANDING: u8 = 5;
    pub const LANDED: u8 = 6;
    pub const GPS_RESCUE: u8 = 7;

    pub const PROCEDURE_DROP_IT: u8 = 0;
    pub const PROCEDURE_AUTO_LANDING: u8 = 1;
    pub const PROCEDURE_GPS_RESCUE: u8 = 2;
    pub const PROCEDURE_COUNT: u8 = 3;

    pub const SWITCH_MODE_STAGE1: u8 = 0;
    pub const SWITCH_MODE_STAGE2: u8 = 2;
    pub const SWITCH_MODE_KILL: u8 = 3;

    fn new() -> Self {
        Self {
            throttle_pwm: 1000, // throttle off
            throttle_low_delay_deciseconds: 100,
            recovery_delay_deciseconds: 5,
            delay_deciseconds: 15,
            landing_time_seconds: 60,
            procedure: Self::PROCEDURE_DROP_IT,
            switch_mode: Self::SWITCH_MODE_STAGE1,
            stick_threshold_percent: 30,
        }
    }
}

impl Default for FailsafeConfig {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::float_cmp)]
    #![allow(unused_results)]

    #[allow(unused)]
    use super::*;

    #[allow(unused)]
    fn is_normal<T: Sized + Send + Sync + Unpin>() {}
    #[allow(unused)]
    fn is_full<T: Sized + Send + Sync + Unpin + Copy + Clone + Default + PartialEq>() {}
    fn is_config<
        T: Sized + Send + Sync + Unpin + Copy + Clone + Default + PartialEq + Serialize + for<'a> Deserialize<'a>,
    >() {
    }

    #[test]
    fn normal_types() {
        is_config::<FailsafeConfig>();
    }
    #[test]
    fn new() {
        let failsafe = FailsafeConfig::new();

        assert_eq!(failsafe.throttle_pwm, 1000); // throttle off
        assert_eq!(failsafe.throttle_low_delay_deciseconds, 100);
        assert_eq!(failsafe.recovery_delay_deciseconds, 5);
        assert_eq!(failsafe.delay_deciseconds, 15);
        assert_eq!(failsafe.landing_time_seconds, 60);
        assert_eq!(failsafe.procedure, FailsafeConfig::PROCEDURE_DROP_IT);
        assert_eq!(failsafe.switch_mode, FailsafeConfig::SWITCH_MODE_STAGE1);
        assert_eq!(failsafe.stick_threshold_percent, 30);
    }
}
