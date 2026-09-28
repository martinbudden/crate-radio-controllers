# radio-controllers Rust Crate<br>![License: MIT](https://img.shields.io/badge/license-MIT-green) [![License: Apache 2.0](https://img.shields.io/badge/License-Apache_2.0-blue.svg)](https://opensource.org/licenses/Apache-2.0) ![open source](https://badgen.net/badge/open/source/blue?icon=github)

## Receivers

Drivers for SBUS, IBUS, and Crossfire/ExpressLRS receivers.

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

This example shows setting a `mac_arm` mode activation condition, on channel `AUX1`.

It is set so that that if `AUX1` is in the range 1500-2000, arming is on.

```rust
use radio_controllers::{ModeActivationCondition, RcMode, RcModes, RxChannel, RxChannels, RxChannelRange};

let mut rc_modes = RcModes::default();
let mut rx_channels = RxChannels::default();

// set a MAC for arming that is true if AUX1 ins in the range 1500-2000.
let mac_arm = ModeActivationCondition::new(RcMode::Arm)
    .with_channel(RxChannel::Aux1)
    .with_range(RxChannelRange::from_pwm(1500, 2000));
rc_modes.set_mac(0, mac_arm);

// Set AUX1 channel to 1750 and confirm is arming is on.
rx_channels[RxChannel::Aux1] = 1750;
assert!(mac_arm.is_active(&rx_channels));

// Set AUX1 channel to 1250 and confirm is arming is off.
rx_channels[RxChannel::Aux1] = 1250;
assert!(!mac_arm.is_active(&rx_channels));
```

## Original implementation

I originally implemented this crate as a C++ library:
[Library-Receivers](https://github.com/martinbudden/Library-Receivers).

## License

Licensed under either of:

* Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.
