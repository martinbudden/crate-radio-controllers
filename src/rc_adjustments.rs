use crate::RxChannelRange;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
pub struct RcAdjustmentRange {
    // when aux channel is in range...
    pub range: RxChannelRange,
    pub aux_channel_index: u8,
    // ..then apply the adjustment function to the aux_switch_channel ...
    pub adjustment_config: u8,
    pub aux_switch_channel_index: u8,
    pub adjustment_center: u8,
    pub adjustment_scale: u16,
}

impl RcAdjustmentRange {
    pub const fn new() -> Self {
        Self {
            range: RxChannelRange::new(),
            aux_channel_index: 0,
            adjustment_config: 0,
            aux_switch_channel_index: 0,
            adjustment_center: 0,
            adjustment_scale: 0,
        }
    }
}

impl Default for RcAdjustmentRange {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
pub enum RcAdjustmentMode {
    #[default]
    Step,
    Select,
}

impl RcAdjustmentMode {
    pub const fn new() -> Self {
        Self::Step
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
pub struct RcTimedAdjustmentState {
    pub timeout_at_milliseconds: u32,
    pub adjustment_range_index: u8,
    pub ready: u8,
}

impl RcTimedAdjustmentState {
    pub const fn new() -> Self {
        Self { timeout_at_milliseconds: 0, adjustment_range_index: 0, ready: 0 }
    }
}

impl Default for RcTimedAdjustmentState {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
pub struct RcContinuosAdjustmentState {
    pub adjustment_range_index: u8,
    pub last_rc_data: u16,
}

impl RcContinuosAdjustmentState {
    pub const fn new() -> Self {
        Self { adjustment_range_index: 0, last_rc_data: 0 }
    }
}

impl Default for RcContinuosAdjustmentState {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
pub struct RcAdjustmentData {
    pub step: u8,
    pub switch_positions: u8,
}

impl RcAdjustmentData {
    pub const fn new() -> Self {
        Self { step: 0, switch_positions: 0 }
    }
}

impl Default for RcAdjustmentData {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
pub struct RcAdjustmentConfig {
    pub adjustment: u8,
    pub adjustment_mode: u8,
    pub data: u8,
}

impl RcAdjustmentConfig {
    pub const fn new() -> Self {
        Self { adjustment: 0, adjustment_mode: 0, data: 0 }
    }
}

impl Default for RcAdjustmentConfig {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn _is_normal<T: Sized + Send + Sync + Unpin>() {}
    fn _is_full<T: Sized + Send + Sync + Unpin + Copy + Clone + Default + PartialEq>() {}
    fn is_config<
        T: Sized + Send + Sync + Unpin + Copy + Clone + Default + PartialEq + Serialize + for<'a> Deserialize<'a>,
    >() {
    }

    #[test]
    fn normal_types() {
        is_config::<RcAdjustmentRange>();
        is_config::<RcAdjustmentMode>();
        is_config::<RcTimedAdjustmentState>();
        is_config::<RcContinuosAdjustmentState>();
        is_config::<RcAdjustmentData>();
        is_config::<RcAdjustmentRange>();
    }
    #[test]
    fn test_new() {
        let config = RcAdjustmentConfig::new();
        assert_eq!(0, config.adjustment);
    }
}
