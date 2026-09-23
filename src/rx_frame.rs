use super::RxChannel;

/// Status of radio link.
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum RxLinkStatus {
    #[default]
    Ok,
    Failsafe,
    NoSignal,
}

impl RxLinkStatus {
    /// Constructor.
    #[must_use]
    pub const fn new() -> Self {
        Self::Ok
    }
}

/// Receiver frame containing array of rx channel values, link status and RSSI.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RxFrame {
    /// The channels in PWM range, nominally `[1000,2000]`.
    pub channels: [u16; RxFrame::MAX_CHANNEL_COUNT],
    pub status: RxLinkStatus,
    pub rssi: u8,
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
    pub const DEFAULT_CHANNEL_VALUE: u16 = RxChannel::LOW;

    /// Constructor.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            channels: [
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
            ],
            status: RxLinkStatus::new(),
            rssi: 0,
        }
    }
}

impl RxFrame {
    /// Returns true if the frame is safe to use for flight control.
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.status == RxLinkStatus::Ok
    }
    /// Returns value of auxiliary channel, or `RxChannel::LOW` if channel index invalid.
    #[must_use]
    pub fn channel(&self, channel_index: u8) -> u16 {
        let index = usize::from(channel_index);
        if index < Self::MAX_CHANNEL_COUNT {
            return self.channels[channel_index as usize];
        }
        RxChannel::LOW
    }
}
#[cfg(test)]
mod test_traits {
    use super::*;

    fn is_full<T: Sized + Send + Sync + Unpin + Copy + Clone + Default + PartialEq>() {}

    #[test]
    fn normal_types() {
        is_full::<RxFrame>();
        is_full::<RxLinkStatus>();
    }
}
