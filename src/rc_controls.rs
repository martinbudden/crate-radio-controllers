#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct RcControlsConfig {
    pub deadband: u8,
    pub yaw_deadband: u8,         // invert the serial RX protocol compared to its default setting.
    pub yaw_control_reversed: u8, // allow rx to operate in half duplex mode on STM32 F4, ignored for F1 and F3.
}

impl RcControlsConfig {
    pub const fn new() -> Self {
        Self { deadband: 0, yaw_deadband: 0, yaw_control_reversed: 0 }
    }
}

impl Default for RcControlsConfig {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn _is_normal<T: Sized + Send + Sync + Unpin>() {}
    fn is_full<T: Sized + Send + Sync + Unpin + Copy + Clone + Default + PartialEq>() {}
    #[cfg(feature = "serde")]
    fn is_config<
        T: Sized + Send + Sync + Unpin + Copy + Clone + Default + PartialEq + Serialize + for<'a> Deserialize<'a>,
    >() {
    }

    #[test]
    fn normal_types() {
        is_full::<RcControlsConfig>();
        #[cfg(feature = "serde")]
        is_config::<RcControlsConfig>();
    }
    #[test]
    fn test_new() {
        let config = RcControlsConfig::new();
        assert_eq!(0, config.deadband);
    }
}
