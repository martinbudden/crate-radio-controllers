use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
pub struct RxConfig {
    pub serial_rx_provider: u8,
    pub serial_rx_inverted: u8, // invert the serial RX protocol compared to its default setting.
    pub half_duplex: u8,        // allow rx to operate in half duplex mode on STM32 F4, ignored for F1 and F3.
    pub rssi_channel: u8,
    pub rssi_scale: u8,
    pub rssi_invert: u8,
    pub rssi_offset: i8,                 // offset applied to the RSSI value
    pub fpv_cam_angle_degrees: u8,       // Camera angle to be scaled into rc commands
    pub air_mode_activate_threshold: u8, // Throttle setpoint percent where airmode gets activated
    pub spektrum_sat_bind: u8,           // number of bind pulses for Spektrum satellite receivers
    pub mid_rc: u16, // Some radios don't have a neutral point centered on 1500. This can be changed here.
    pub min_check: u16, // minimum rc
    pub max_check: u16, // maximum rc
    pub rx_min_us: u16, // rx_min in microseconds
    pub rx_max_us: u16, // rx_max in microseconds
}

impl RxConfig {
    pub const fn new() -> Self {
        Self {
            serial_rx_provider: 0,
            serial_rx_inverted: 0,
            half_duplex: 0,
            rssi_channel: 0,
            rssi_scale: 0,
            rssi_invert: 0,
            rssi_offset: 0,
            fpv_cam_angle_degrees: 0,
            air_mode_activate_threshold: 0,
            spektrum_sat_bind: 0,
            mid_rc: 0,
            min_check: 0,
            max_check: 0,
            rx_min_us: 0,
            rx_max_us: 0,
        }
    }
}

impl Default for RxConfig {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
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
        is_config::<RxConfig>();
    }
    #[test]
    fn test_new() {
        let config = RxConfig::new();
        assert_eq!(0, config.serial_rx_provider);
    }
}
