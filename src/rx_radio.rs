#[cfg(feature = "storage")]
use sequential_storage::map::PostcardValue;
#[cfg(feature = "serde")]
use {
    postcard::experimental::max_size::MaxSize,
    serde::{Deserialize, Serialize},
};

use super::{
    RxFrame,
    protocols::{CrsfDecoder, IbusDecoder, SbusDecoder},
};

#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize, MaxSize))]
pub enum RadioType {
    #[default]
    Crsf,
    Ibus,
    Sbus,
}

#[cfg(feature = "storage")]
impl PostcardValue<'_> for RadioType {}

impl_try_from_u8!(RadioType);

impl RadioType {
    #[must_use]
    pub fn from_u8(value: u8) -> Self {
        match value {
            0 => Self::Crsf,
            1 => Self::Ibus,
            2 => Self::Sbus,
            _ => Self::default(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Radio {
    Crsf(CrsfDecoder),
    Ibus(IbusDecoder),
    Sbus(SbusDecoder),
}

impl Radio {
    #[must_use]
    pub const fn new(radio_type: RadioType) -> Radio {
        match radio_type {
            RadioType::Crsf => Self::Crsf(CrsfDecoder::new()),
            RadioType::Ibus => Self::Ibus(IbusDecoder::new()),
            RadioType::Sbus => Self::Sbus(SbusDecoder::new()),
        }
    }
}

impl Radio {
    pub fn on_byte_received(&mut self, byte: u8) -> Option<RxFrame> {
        match self {
            // CRSF can return different types of frames.
            Self::Crsf(decoder) => decoder.on_byte_received(byte),
            // IBUS always returns channels and link status, so convert to an RxFrame.
            Self::Ibus(decoder) => {
                if let Some((channels, link_status)) = decoder.on_byte_received(byte) {
                    Some(RxFrame::ChannelsLinkStatus { channels, link_status })
                } else {
                    None
                }
            }
            // SBUS always returns channels and link status, so convert to an RxFrame.
            Self::Sbus(decoder) => {
                if let Some((channels, link_status)) = decoder.on_byte_received(byte) {
                    Some(RxFrame::ChannelsLinkStatus { channels, link_status })
                } else {
                    None
                }
            }
        }
    }
}

#[cfg(test)]
mod test_traits {
    use super::*;

    fn is_full_no_default<T: Sized + Send + Sync + Unpin + Copy + Clone + PartialEq>() {}
    fn is_full<T: Sized + Send + Sync + Unpin + Copy + Clone + Default + PartialEq>() {}
    #[cfg(feature = "serde")]
    fn is_serde<T: Serialize + MaxSize + for<'a> Deserialize<'a>>() {}
    #[cfg(feature = "storage")]
    fn is_storage<T: for<'a> PostcardValue<'a>>() {}

    #[test]
    fn normal_types() {
        is_full_no_default::<Radio>();
        is_full::<RadioType>();
        #[cfg(feature = "serde")]
        is_serde::<RadioType>();
        #[cfg(feature = "storage")]
        is_storage::<RadioType>();
    }
}
