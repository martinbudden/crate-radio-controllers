# radio-controllers Rust Crate<br>![License: MIT](https://img.shields.io/badge/license-MIT-green) [![License: Apache 2.0](https://img.shields.io/badge/License-Apache_2.0-blue.svg)](https://opensource.org/licenses/Apache-2.0) ![open source](https://badgen.net/badge/open/source/blue?icon=github)

Implements decoders for SBUS, IBUS, and Crossfire/ExpressLRS receivers.

`radio-controllers` also implements Betaflight compatible Mode Activation Conditions (MACs),
whereby a mode can be activated when a channel is in a given range. It also has logic
to activate a mode when a combination of channels has a combination of values.

## Examples

### Decoding an IBUS serial byte stream

```rust
use radio_controllers::{ IbusDecoder, RxChannels, RxLinkStatus };

let expected_channels = RxChannels::from_channels([
    1500, 1500, 1100, 1500, // Roll, Pitch, Throttle, Yaw
    1000, 2000, 1500, 1000, // AUX 1-4
    1200, 1300, 1400, 1600, // AUX 5-8
    1700, 1800, 1000, 1000, // AUX 9-12
]);

// simulate a byte stream from the serial port using an array.
let byte_stream = [
    0x20, 0x40, 0xDC, 0x05, 0xDC, 0x05, 0x4C, 0x04, 0xDC, 0x05, 0xE8, 0x03, 0xD0, 0x07, 0xDC, 0x05,
    0xE8, 0x03, 0xB0, 0x04, 0x14, 0x05, 0x78, 0x05, 0x40, 0x06, 0xA4, 0x06, 0x08, 0x07, 0xD5, 0xF6,
];

// decode the byte stream
let mut decoder = IbusDecoder::new();
let mut result = None;
for &byte in byte_stream.iter() {
    result = decoder.on_byte_received(byte);
}

// deconstruct result and check it is correct
if let Some((channels, link_status)) = result {
    assert_eq!(channels, expected_channels);
    assert_eq!(link_status, RxLinkStatus::Ok);
} else {
    panic!("decode failed");
}
```

### Mode activation conditions

This example shows setting a mode activation conditions for:

* Arming on channel `AUX1`, values 1500-2100
* Horizon mode on channel `AUX2`, values 1250-1750
* Angle mode on channel `AUX2`, values 1750-2100

```rust
use radio_controllers::{ModeActivationCondition, RcMode, RcModes, RxChannel, RxChannels, RxChannelRange};

let mut rc_modes = RcModes::default();

let mac_arm = ModeActivationCondition::new(RcMode::Arm)
    .with_channel(RxChannel::Aux1)
    .with_range(RxChannelRange::from_pwm(1500, 2100));
_ = rc_modes.push_mac(mac_arm);

let mac_horizon = ModeActivationCondition::new(RcMode::Horizon)
    .with_channel(RxChannel::Aux2)
    .with_range(RxChannelRange::from_pwm(1250, 1750));
_ = rc_modes.push_mac(mac_horizon);

let mac_angle = ModeActivationCondition::new(RcMode::Angle)
    .with_channel(RxChannel::Aux2)
    .with_range(RxChannelRange::from_pwm(1750, 2100));
_ = rc_modes.push_mac(mac_angle);

let mut rx_channels = RxChannels::default();

rx_channels[RxChannel::Aux1] = 1750;
rx_channels[RxChannel::Aux2] = 1800;

// need to call update activated modes when the channel values change
rc_modes.update_activated_modes(&rx_channels);

// Check arm mode is active.
assert!(rc_modes.is_mode_active(RcMode::Arm));
// Check angle mode is active.
assert!(rc_modes.is_mode_active(RcMode::Angle));
// Check horizon mode is NOT active.
assert!(!rc_modes.is_mode_active(RcMode::Horizon));
```

### Combined example

This shows how the an IBUS decoder might be combined with Mode Activation Conditions in an application:

1. The decoder and the MACs are setup in initialization
2. In the main loop:
    1. The serial byte stream from the radio is decoded.
    2. The MACs are updated
    3. Action is taken according to what modes are active.

```rust
use radio_controllers::{IbusDecoder, ModeActivationCondition, RcMode, RcModes, RxChannel, RxChannels, RxChannelRange};

// Simulated byte stream from serial port.
let byte_stream = [
    0x20, 0x40, 0xDC, 0x05, 0xDC, 0x05, 0x4C, 0x04, 0xDC, 0x05, 0xE8, 0x03, 0xD0, 0x07, 0xDC, 0x05,
    0xE8, 0x03, 0xB0, 0x04, 0x14, 0x05, 0x78, 0x05, 0x40, 0x06, 0xA4, 0x06, 0x08, 0x07, 0xD5, 0xF6,
];

// Initialization: set up decoder and`RcModes`.
let mut decoder = IbusDecoder::new();
let mut rc_modes = RcModes::default();

let mac_arm = ModeActivationCondition::new(RcMode::Arm)
    .with_channel(RxChannel::Aux1)
    .with_range(RxChannelRange::from_pwm(1500, RxChannelRange::MAX));
_ = rc_modes.push_mac(mac_arm);
let mac_horizon = ModeActivationCondition::new(RcMode::Horizon)
    .with_channel(RxChannel::Aux2)
    .with_range(RxChannelRange::from_pwm(1250, 1750));
_ = rc_modes.push_mac(mac_horizon);
let mac_angle = ModeActivationCondition::new(RcMode::Angle)
    .with_channel(RxChannel::Aux2)
    .with_range(RxChannelRange::from_pwm(1750, RxChannelRange::MAX));
_ = rc_modes.push_mac(mac_angle);

// Main program loop: decode the byte stream from the radio,
// and take action depending on the RC modes.

let mut result = None;
for &byte in byte_stream.iter() {
    result = decoder.on_byte_received(byte);
}

if let Some((rx_channels, _rx_link_status)) = result {
    // Call `update_activated_modes` since we have new channel values.
    rc_modes.update_activated_modes(&rx_channels);

    // Take action according to what modes are active.

    // Check arm mode is NOT active.
    assert!(!rc_modes.is_mode_active(RcMode::Arm));
    // Check angle mode is active.
    assert!(rc_modes.is_mode_active(RcMode::Angle));
    // Check horizon mode is NOT active.
    assert!(!rc_modes.is_mode_active(RcMode::Horizon));
} else {
    // Do some failsafe handling here.
}
```

## Original implementation

I originally implemented this crate as a C++ library:
[Library-Receivers](https://github.com/martinbudden/Library-Receivers).

## License

Licensed under either of:

* Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.
