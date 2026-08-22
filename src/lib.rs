#![doc = include_str!("../README.md")]
#![no_std]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
//#![deny(missing_docs)]
#![deny(
    missing_copy_implementations,
    missing_debug_implementations,
    trivial_casts,
    trivial_numeric_casts,
    unused_must_use,
    unused_extern_crates,
    unused_import_braces,
    unused_qualifications,
    unused_results
)]
#![warn(unused_results)]
#![warn(clippy::pedantic)]
#![warn(clippy::doc_paragraphs_missing_punctuation)]

mod controls;
mod failsafe;
mod mock_uart;
mod protocols;
mod rates;
mod rc_adjustments;
mod rc_controls;
mod rc_mode;
mod rc_modes;
mod rx_config;
mod rx_radio;

pub use crate::protocols::{CrsfRadio, IbusRadio, MockRadio};
pub use controls::{RcSticks, RxControlsPwm};
pub use failsafe::{FailsafeConfig, FailsafeProcedure, FailsafeSwitchMode};
pub use mock_uart::MockUart;
pub use rates::{Rates, RatesConfig, RatesType, ThrottleLimitType};
pub use rc_adjustments::{
    RcAdjustmentConfig, RcAdjustmentData, RcAdjustmentMode, RcAdjustmentRange, RcContinuosAdjustmentState,
    RcTimedAdjustmentState,
};
pub use rc_controls::RcControlsConfig;
pub use rc_mode::RcMode;
pub use rc_modes::{ModeActivationCondition, RcModes, RxChannelRange};
pub use rx_config::{RadioType, RxConfig};
pub use rx_radio::{Eui48, Radio, RxChannel, RxFrame, RxLinkStatus, RxRadio, RxRadioCommon};
