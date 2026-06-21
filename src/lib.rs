#![doc = include_str!("../README.md")]
#![no_std]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![deny(unused_must_use)]
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
mod rc_modes;
mod rx_config;
mod rx_receiver;

pub use crate::protocols::receiver_crsf::CrsfReceiver;
pub use controls::{RcSticks, RxControlsPwm};
pub use failsafe::FailsafeConfig;
pub use mock_uart::MockUart;
pub use rates::{Rates, RatesConfig};
pub use rc_adjustments::{
    RcAdjustmentConfig, RcAdjustmentData, RcAdjustmentMode, RcAdjustmentRange, RcContinuosAdjustmentState,
    RcTimedAdjustmentState,
};
pub use rc_controls::RcControlsConfig;
pub use rc_modes::{ModeActivationCondition, RcModes, RcModesArray, RxChannelRange};
pub use rx_config::RxConfig;
pub use rx_receiver::{Eui48, RxChannel, RxFrame, RxLinkStatus, RxReceiver, RxReceiverCommon};
