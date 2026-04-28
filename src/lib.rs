#![doc = include_str!("../README.md")]
#![no_std]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![deny(unused_must_use)]
#![warn(unused_results)]
#![warn(clippy::pedantic)]
#![warn(clippy::doc_paragraphs_missing_punctuation)]
#![allow(clippy::return_self_not_must_use)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::manual_unwrap_or_default)]
#![allow(clippy::manual_unwrap_or)]

mod controls;
mod failsafe;
mod mock_uart;
mod protocols;
mod rates;
mod rc_message;
mod rc_modes;
mod rx_receiver;

pub use crate::protocols::receiver_crsf::CrsfReceiver;
pub use controls::{RcSticks, RxControlsPwm};
pub use failsafe::FailsafeConfig;
pub use mock_uart::MockUart;
pub use rates::{Rates, RatesConfig};
pub use rc_modes::{ModeActivationCondition, RcMode, RcModes, RxChannelRange};
pub use rx_receiver::{Eui48, RxChannel, RxFrame, RxLinkStatus, RxReceiver, RxReceiverCommon};

pub use rc_message::RadioControlMessage;
