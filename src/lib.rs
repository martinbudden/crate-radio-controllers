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

#[macro_use]
mod macros;

mod controls;
mod failsafe;
mod protocols;
mod rates;
mod rc_adjustments;
mod rc_controls_config;
mod rc_mode;
mod rc_modes;
mod rx_channel;
mod rx_config;
mod rx_frame;
mod rx_radio;

pub use crate::protocols::{CrsfRadio, IbusRadio, SbusRadio};

pub use controls::{RcSticks, RxControlsPwm};
pub use failsafe::{FailsafeConfig, FailsafeProcedure, FailsafeSwitchMode};
pub use rates::{Rates, RatesConfig, RatesType, ThrottleLimitType};

pub use rc_adjustments::{
    RcAdjustmentConfig, RcAdjustmentData, RcAdjustmentMode, RcAdjustmentRange, RcContinuosAdjustmentState,
    RcTimedAdjustmentState,
};
pub use rc_controls_config::RcControlsConfig;
pub use rc_mode::RcMode;
pub use rc_modes::{ModeActivationCondition, RcModes};

pub use rx_channel::{RxChannel, RxChannelRange, RxChannels};
pub use rx_config::{RadioType, RxConfig};
pub use rx_frame::{RxFrame, RxLinkStatus};
pub use rx_radio::{Radio, RxRadio};
