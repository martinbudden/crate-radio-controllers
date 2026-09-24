use crate::{RxFrame, RxLinkStatus};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CrsfFrame {
    pub channels: [u16; Self::CHANNEL_COUNT],
    pub failsafe: bool,
    pub frame_lost: bool,
    pub rssi: u8,
}

impl Default for CrsfFrame {
    fn default() -> Self {
        Self::new()
    }
}

impl CrsfFrame {
    pub const CHANNEL_COUNT: usize = 16;

    /// Constructor.
    pub const fn new() -> Self {
        Self { channels: [0u16; Self::CHANNEL_COUNT], failsafe: false, frame_lost: false, rssi: 0 }
    }
}

impl From<CrsfFrame> for RxFrame {
    fn from(frame: CrsfFrame) -> Self {
        let status = if frame.failsafe {
            RxLinkStatus::Failsafe
        } else if frame.frame_lost {
            RxLinkStatus::NoSignal
        } else {
            RxLinkStatus::Ok
        };

        let mut channels = [Self::DEFAULT_CHANNEL_VALUE; Self::MAX_CHANNEL_COUNT];
        channels[..frame.channels.len()].copy_from_slice(&frame.channels);

        Self { channels, status, rssi: frame.rssi }
    }
}
