/// 48-bit extended unique identifier (often synonymous with MAC address).<br><br>
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Eui48 {
    pub octets: [u8; 6],
}

impl Eui48 {
    pub fn new() -> Self {
        Self::default()
    }
}

/// Properties common to all RX receivers.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RxReceiverCommon {
    pub packet_received: bool, // may be invalid packet
    pub new_packet_available: bool,
    pub positive_half_throttle: bool,
    pub packet_count: i32,
    pub dropped_packet_count_delta: i32,
    pub dropped_packet_count: i32,
    pub dropped_packet_count_previous: i32,
    pub tick_count_delta: i32,
}

impl Default for RxReceiverCommon {
    fn default() -> Self {
        Self::new()
    }
}

impl RxReceiverCommon {
    // standardize receivers to use AETR (Ailerons, Elevator, Throttle, Rudder), ie ROLL, PITCH, THROTTLE, YAW
    pub fn new() -> Self {
        Self {
            packet_received: false,
            new_packet_available: false,
            positive_half_throttle: false,
            packet_count: 0,
            dropped_packet_count_delta: 0,
            dropped_packet_count: 0,
            dropped_packet_count_previous: 0,
            tick_count_delta: 0,
        }
    }
}

/// Status of radio link.
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum RxLinkStatus {
    #[default]
    Ok,
    Failsafe,
    NoSignal,
}

impl RxLinkStatus {
    pub const fn new() -> Self {
        Self::Ok
    }
}

/// RX channel constants.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RxChannel {}

impl RxChannel {
    pub const ROLL: usize = 0;
    pub const PITCH: usize = 1;
    pub const THROTTLE: usize = 2;
    pub const YAW: usize = 3;
    pub const AUX1: usize = 4;
    pub const AUX2: usize = 5;
    pub const AUX3: usize = 6;
    pub const AUX4: usize = 7;
    pub const AUX5: usize = 8;
    pub const AUX6: usize = 9;
    pub const AUX7: usize = 10;
    pub const AUX8: usize = 11;
    pub const AUX9: usize = 12;
    pub const AUX10: usize = 13;
    pub const AUX11: usize = 14;
    pub const AUX12: usize = 15;
    pub const AUX13: usize = 16;
    pub const AUX14: usize = 17;
    pub const AUX15: usize = 18;
    pub const AUX16: usize = 19;

    // PWM ranges
    pub const LOW: u16 = 1000;
    pub const HIGH: u16 = 2000;
    pub const MID: u16 = 1500;
    pub const RANGE: u16 = Self::HIGH - Self::LOW;
}

/// Array of RX channels.
pub(crate) type RxChannels = [u16; RxFrame::MAX_CHANNEL_COUNT];

/// Receiver frame containing array of rx channel values, link status and RSSI.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RxFrame {
    /// The channels in PWM range, nominally `[1000,2000]`.
    pub channels: [u16; RxFrame::MAX_CHANNEL_COUNT],
    pub status: RxLinkStatus,
    pub rssi: u8,
}

impl RxFrame {
    pub const fn new() -> Self {
        Self {
            channels: [
                RxChannel::MID,
                RxChannel::MID,
                RxChannel::LOW,
                RxChannel::MID,
                RxChannel::LOW,
                RxChannel::LOW,
                RxChannel::LOW,
                RxChannel::LOW,
                RxChannel::LOW,
                RxChannel::LOW,
                RxChannel::LOW,
                RxChannel::LOW,
                RxChannel::LOW,
                RxChannel::LOW,
                RxChannel::LOW,
                RxChannel::LOW,
            ],
            status: RxLinkStatus::new(),
            rssi: 0,
        }
    }
}

impl Default for RxFrame {
    fn default() -> Self {
        Self::new()
    }
}

impl RxFrame {
    // SBUS has 18 channels (the last two are digital channels with the two values 1000 or 2000), but we only use 16.
    // IBUS has 14 channels
    // CRSF has 16 channels
    pub const MAX_CHANNEL_COUNT: usize = 16;
    pub const DEFAULT_CHANNEL_VALUE: u16 = RxChannel::LOW; // center

    /// Returns true if the frame is safe to use for flight control.
    pub fn is_valid(&self) -> bool {
        self.status == RxLinkStatus::Ok
    }
    /// Returns value of auxiliary channel, or `RxChannel::LOW` if channel index invalid.
    pub fn auxiliary_channel(&self, channel_index: u8) -> u16 {
        let index = usize::from(channel_index);
        if index < Self::MAX_CHANNEL_COUNT && index > RxChannel::AUX1 {
            return self.channels[channel_index as usize - RxChannel::AUX1];
        }
        RxChannel::LOW
    }
}

/// The common interface for all RC protocols.
pub trait RxReceiver {
    /// Associated type for the frame/data format.
    type Frame: Into<RxFrame>; // Every Frame must be convertible to RcFrame

    //fn update(&mut self) -> Result<Option<Self::Frame>, Error>;
    //fn update(&mut self, tick_count_delta: u32);

    fn is_data_available(&self) -> bool;
    fn on_data_received_from_isr(&mut self, data: u8) -> bool;
    fn read_byte(&mut self) -> u8;

    fn channel_pwm(&self, channel_index: u8) -> u16;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[allow(unused)]
    fn is_normal<T: Sized + Send + Sync + Unpin>() {}
    fn is_full<T: Sized + Send + Sync + Unpin + Copy + Clone + Default + PartialEq>() {}

    #[test]
    fn normal_types() {
        is_full::<Eui48>();
        is_full::<RxReceiverCommon>();
    }
    #[test]
    fn test_new() {
        let receiver = RxReceiverCommon::new();
        assert!(!receiver.packet_received);
    }
}
