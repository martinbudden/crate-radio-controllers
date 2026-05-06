use serde::{Deserialize, Serialize};

/// Configuration data for Rates.
#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
pub struct RatesConfig {
    pub limits: [u16; Self::AXIS_COUNT],
    pub rc_rates: [u8; Self::AXIS_COUNT],
    pub rc_expos: [u8; Self::AXIS_COUNT],
    pub rates: [u8; Self::AXIS_COUNT],
    pub throttle_midpoint: u8,
    pub throttle_expo: u8,
    pub throttle_limit_type: u8,
    pub throttle_limit_percent: u8, // Sets the maximum pilot commanded throttle limit
                                    //pub rates_type: u8, // not used
}

impl RatesConfig {
    pub const AXIS_COUNT: usize = 3;

    pub const LIMIT_MAX: u16 = 1998;
    pub const RC_RATES_MAX: u8 = 255;
    pub const RC_EXPOS_MAX: u8 = 100;
    pub const THROTTLE_MAX: u8 = 100;

    pub const TYPE_BETAFLIGHT: u8 = 0;
    pub const TYPE_RACEFLIGHT: u8 = 1;
    pub const TYPE_KISS: u8 = 2;
    pub const TYPE_ACTUAL: u8 = 3;
    pub const TYPE_QUICK: u8 = 4;
    pub const TYPE_COUNT: u8 = 5;

    pub const THROTTLE_LIMIT_TYPE_OFF: u8 = 0;
    pub const THROTTLE_LIMIT_TYPE_SCALE: u8 = 1;
    pub const THROTTLE_LIMIT_TYPE_CLIP: u8 = 2;
    pub const THROTTLE_LIMIT_TYPE_COUNT: u8 = 3;

    pub const fn new() -> Self {
        Self {
            limits: [Self::LIMIT_MAX, Self::LIMIT_MAX, Self::LIMIT_MAX],
            rc_rates: [7, 7, 7],
            rc_expos: [0, 0, 0],
            rates: [67, 67, 67],
            throttle_midpoint: 50,
            throttle_expo: 0,
            throttle_limit_type: Self::THROTTLE_LIMIT_TYPE_OFF,
            throttle_limit_percent: 100,
            //rates_type: Self::TYPE_ACTUAL,
        }
    }
}

impl Default for RatesConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// Rates define the sensitivity of the control sticks, mapping a linear input to a non-linear command.<br><br>
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rates {
    pub limits: [f32; Self::AXIS_COUNT],
    pub rc_rates: [f32; Self::AXIS_COUNT],
    pub rc_expos: [f32; Self::AXIS_COUNT],
    pub rates: [f32; Self::AXIS_COUNT],
    pub throttle_midpoint: f32,
    pub throttle_expo: f32,
    pub throttle_limit_type: u8,
    pub throttle_limit_percent: f32, // Sets the maximum pilot commanded throttle limit
    pub max_roll_angle_degrees: f32,
    pub max_pitch_angle_degrees: f32,
}

impl Rates {
    pub fn new(config: RatesConfig) -> Self {
        Self {
            limits: [f32::from(config.limits[0]), f32::from(config.limits[1]), f32::from(config.limits[2])],
            rc_rates: [f32::from(config.rc_rates[0]), f32::from(config.rc_rates[1]), f32::from(config.rc_rates[2])],
            rc_expos: [f32::from(config.rc_expos[0]), f32::from(config.rc_expos[1]), f32::from(config.rc_expos[2])],
            rates: [f32::from(config.rates[0]), f32::from(config.rates[1]), f32::from(config.rates[2])],
            throttle_midpoint: f32::from(config.throttle_midpoint),
            throttle_expo: f32::from(config.throttle_expo),
            throttle_limit_type: config.throttle_limit_type,
            throttle_limit_percent: f32::from(config.throttle_limit_percent),
            max_roll_angle_degrees: 60.0,
            max_pitch_angle_degrees: 60.0,
            //rates_type: Self::TYPE_ACTUAL,
        }
    }
}

impl Default for Rates {
    fn default() -> Self {
        Self::new(RatesConfig::default())
    }
}

impl Rates {
    pub const ROLL: usize = 0;
    pub const PITCH: usize = 1;
    pub const YAW: usize = 2;
    pub const AXIS_COUNT: usize = 3;
    pub const LIMIT_MAX: f32 = 1998.0;
}

impl Rates {
    pub fn set_to_pass_through(&mut self) {
        self.rc_rates = [100.0, 100.0, 100.0]; // center sensitivity
        self.rc_expos = [0.0, 0.0, 0.0]; // movement sensitivity, nonlinear
        self.rates = [0.0, 0.0, 0.0]; // movement sensitivity, linear
        //self.rates.rates_type = Self::TYPE_ACTUAL;
    }
    pub fn apply(self, axis: usize, rc_command: f32) -> f32 {
        let rc_command2 = rc_command * rc_command;
        let rc_command_abs = rc_command.abs();

        let mut expo = self.rc_expos[axis] / 100.0;
        expo = rc_command_abs * rc_command * (expo * (rc_command2 * rc_command2 - 1.0) + 1.0);

        let center_sensitivity = self.rc_rates[axis];
        let stick_movement = (self.rates[axis] - center_sensitivity).max(0.0);
        let angle_rate = 10.0 * (rc_command * center_sensitivity + expo * stick_movement);

        angle_rate.clamp(-self.limits[axis], self.limits[axis])
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::float_cmp)]

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
        is_config::<RatesConfig>();
        is_full::<Rates>();
    }
    #[test]
    fn new() {
        let rates = RatesConfig::new();

        assert_eq!([1998, 1998, 1998], rates.limits);
        assert_eq!([7, 7, 7], rates.rc_rates);
        assert_eq!([0, 0, 0], rates.rc_expos);
        assert_eq!([67, 67, 67], rates.rates);
        assert_eq!(50, rates.throttle_midpoint);
        assert_eq!(0, rates.throttle_expo);
        assert_eq!(RatesConfig::THROTTLE_LIMIT_TYPE_OFF, rates.throttle_limit_type);
        assert_eq!(100, rates.throttle_limit_percent);
    }
    #[test]
    fn default() {
        let rates = Rates::default();

        let roll = rates.apply(Rates::ROLL, 0.0);
        assert_eq!(0.0, roll);
        let roll = rates.apply(Rates::ROLL, 0.25);
        assert_eq!(55.0, roll);
        let roll = rates.apply(Rates::ROLL, 0.5);
        assert_eq!(185.0, roll);
        let roll = rates.apply(Rates::ROLL, 0.75);
        assert_eq!(390.0, roll);
        let roll = rates.apply(Rates::ROLL, 1.0);
        assert_eq!(670.0, roll);
    }
    #[test]
    fn pass_through() {
        let mut rates = Rates::default();
        rates.set_to_pass_through();

        let roll = rates.apply(Rates::ROLL, 0.0);
        assert_eq!(0.0, roll);
        let roll = rates.apply(Rates::ROLL, 0.25);
        assert_eq!(250.0, roll);
        let roll = rates.apply(Rates::ROLL, 0.5);
        assert_eq!(500.0, roll);
        let roll = rates.apply(Rates::ROLL, 0.75);
        assert_eq!(750.0, roll);
        let roll = rates.apply(Rates::ROLL, 1.0);
        assert_eq!(1000.0, roll);
    }
}
