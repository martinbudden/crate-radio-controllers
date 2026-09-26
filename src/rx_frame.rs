use super::RxChannel;

/// Crossfire compatible frame types.
#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RxFrameType {
    // see https://github.com/crsf-wg/crsf/wiki/Packet-Types
    Gps = 0x02,
    VarioSensor = 0x07,
    BatterySensor = 0x08,
    BaroAltitude = 0x09,
    #[default]
    Heartbeat = 0x0B,
    LinkStatistics = 0x14,
    RcChannels = 0x16,
    SubsetRcChannels = 0x17,
    LinkStatisticsRx = 0x1C,
    LinkStatisticsTx = 0x1D,
    Attitude = 0x1E,
    FlightMode = 0x21,
    // Extended Header Frames, range: 0x28 to 0x96
    DevicePing = 0x28,
    DeviceInfo = 0x29,
    ParameterSettingsEntry = 0x2B,
    ParameterRead = 0x2C,
    ParameterWrite = 0x2D,
    Command = 0x32,
    // MSP commands
    MspReq = 0x7A,
    MspResp = 0x7B,
    MspWrite = 0x7C,
    DisplayportCmd = 0x7D,
    ArdupilotResp = 0x80,
}

impl RxFrameType {
    /// Forgiving conversion, converts invalid values to default.
    #[must_use]
    pub fn from_u8(value: u8) -> Self {
        match value {
            0x02 => Self::Gps,
            0x07 => Self::VarioSensor,
            0x08 => Self::BatterySensor,
            0x09 => Self::BaroAltitude,
            0x0B => Self::Heartbeat,
            0x14 => Self::LinkStatistics,
            0x16 => Self::RcChannels,
            0x17 => Self::SubsetRcChannels,
            0x1C => Self::LinkStatisticsRx,
            0x1D => Self::LinkStatisticsTx,
            0x1E => Self::Attitude,
            0x21 => Self::FlightMode,
            0x28 => Self::DevicePing,
            0x29 => Self::DeviceInfo,
            0x2B => Self::ParameterSettingsEntry,
            0x2C => Self::ParameterRead,
            0x2D => Self::ParameterWrite,
            0x32 => Self::Command,
            0x7A => Self::MspReq,
            0x7B => Self::MspResp,
            0x7C => Self::MspWrite,
            0x7D => Self::DisplayportCmd,
            0x80 => Self::ArdupilotResp,
            _ => Self::default(),
        }
    }
}

/// Status of radio link.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RxLinkStatus {
    #[default]
    Ok,
    Failsafe,
    NoSignal,
}

impl RxLinkStatus {
    /// Forgiving conversion, converts invalid values to default.
    #[must_use]
    pub fn from_u8(value: u8) -> Self {
        match value {
            0 => Self::Ok,
            1 => Self::Failsafe,
            2 => Self::NoSignal,
            _ => Self::default(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RxFrame {
    ChannelsLink {
        channels_link: RxChannelsLinkStatus,
    },
    LinkStatisticsTx {
        rssi_dbm: u8,
        rssi_percent: u8,
        link_quality: u8, // LQ of 0 may used to indicate a disconnected status to the handset
        snr: i8,
    },
    Battery {
        voltage: u16, // deci-volts
        current: u16, // deci-amps
    },
    Heartbeat(),
    Unknown {
        frame_type: u8,
    },
}

/*/// Receiver frame containing array of rx channel values, link status and RSSI.
#[allow(unused)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RxFrameOld {
    /// The channels in PWM range, nominally `[1000,2000]`.
    pub channels: [u16; Self::CHANNEL_COUNT],
    pub frame_type: RxFrameType,
    pub link_status: RxLinkStatus,
    pub rssi: u8,
}*/

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RxChannelsLinkStatus {
    pub channels: RxChannels,
    pub link_status: RxLinkStatus,
}

impl Default for RxChannelsLinkStatus {
    fn default() -> Self {
        Self::new()
    }
}

impl RxChannelsLinkStatus {
    // SBUS has 18 channels (the last two are digital channels with the two values 1000 or 2000), but we only use 16.
    // IBUS has 14 channels
    // CRSF has 16 channels
    pub const CHANNEL_COUNT: usize = 16;
    pub const DEFAULT_CHANNEL_VALUE: u16 = RxChannel::LOW;

    pub const FAILSAFE_CHANNEL_VALUES: [u16; Self::CHANNEL_COUNT] = [
        RxChannel::MID, // Sticks default to MID.
        RxChannel::MID,
        RxChannel::LOW, // Throttle defaults to LOW.
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
    ];

    /// Constructor.
    #[must_use]
    pub const fn new() -> Self {
        Self { channels: Self::FAILSAFE_CHANNEL_VALUES, link_status: RxLinkStatus::Ok }
    }
}

impl RxChannelsLinkStatus {
    /// Returns true if the frame is safe to use for flight control.
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.link_status == RxLinkStatus::Ok
    }
    /// Returns value of auxiliary channel, or `RxChannel::LOW` if channel index invalid.
    #[must_use]
    pub fn channel(&self, channel_index: u8) -> u16 {
        let index = usize::from(channel_index);
        if index < Self::CHANNEL_COUNT {
            return self.channels[channel_index as usize];
        }
        RxChannel::LOW
    }
    pub fn set_channels_to_failsafe_values(&mut self) {
        self.channels = Self::FAILSAFE_CHANNEL_VALUES;
    }
}

/// Array of RX channels.
pub type RxChannels = [u16; RxChannelsLinkStatus::CHANNEL_COUNT];

#[cfg(test)]
mod test_traits {
    use super::*;

    fn is_full<T: Sized + Send + Sync + Unpin + Copy + Clone + Default + PartialEq>() {}

    #[test]
    fn normal_types() {
        is_full::<RxChannelsLinkStatus>();
        is_full::<RxLinkStatus>();
    }
}
