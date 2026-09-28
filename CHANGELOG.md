# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).

Releases of the form `0.1.n` do not adhere to [Semantic Versioning](https://semver.org/spec/v2.0.0.html),
that is each release may contain incompatible API changes.

Once the API has stabilized this project will adopt semantic versioning, the first release to do so will be `0.2.0`.

## [0.1.11] - 2026-09-xx

### Added

- support for Continuous Integration.
- Added `RxFrameType`.
- Added `RcModeLogic`
- Added CRSF decoder.
- Added IBus Decoder.
- Added `ModeActivationConditions` pseudo-array.

### Changed

- Split `serde` feature into `serde` and `storage`.
- Updated to Rust version 1.89.
- Changed `RxChannel` from `struct` to `enum`.
- Improved `ModeActivationCondition` constructors.
- Changed IBUS and SBUS `on_byte_received` to return `Option<(RxChannels, RxLinkStatus)>`.
- Split common payload parsing out of SBUS and CRSF.
- Made SBUS and IBUS decoders more consistent
- Changed `RxChannels` from `array` to `struct` pseudo-array.
- Changed to use discriminated `enum` for `RxFrame`.
- `Radio` `on_byte_received` now returns Option.
- Improved `parse_sbus_channels`.
- Updated Ibus and Sbus decoder state machines.
- Split structs into separate modules.
- Improved build support.
- Rearranged directory structure.
- Split `serde` feature into `serde` and `storage`.
- Updated `crc`.

### Removed

- Got rid of all the configuration structs and moved them to Protoflight.
- Got rid of `RxRadio` trait.
- Got rid of `RxChannelsLink` struct.
- Got rid of a lot of unneeded code.
- Removed `RxRadioCommon`.
- Removed dependency on embassy.

## [0.1.7] - 2026-09-01

### Added

- support for `postcard` `MaxSize`.
- more tests.

### Changed

- General renaming from "Receiver" to "Radio" to avoid confusion with Embassy `Watch` `Receiver`s.
- Updated to `sequential-storage` `0.8.1`.
- use `enum`s rather than `u8`s for config values where appropriate.
- improved conversion of `enum`s to `u8`s.
- fixed CRC calculation.
- no longer dependent on `libm`.

## [0.1.6] - 2026-08-04

### Added

- `#[must_use]` attribute to selected functions.
- Mode Activation Condition code and tests.

### Changed

- updated to `simple-bitset` version 0.1.5.
- `new` functions to `const` where possible.
- updated documentation.
- subsumed `RcModesArray` into `RcMode`.
- tidied `RcModes`.

### Removed

- `allow`s from `lib.rs`.
- `RcModesArray`.

## [0.1.5] - 2026-05-23

### Added

- `.cargo/cargo.toml`.

### Changed

- Made `serde` an optional feature.

### Removed

- `katex-header.html`.

## [0.1.4] - 2026-05-15

### Changed

- Changed to use `simple-bitset` crate.

## [0.1.3] - 2026-05-13

### Added

- RC Adjustments.
- Serialization/deserialization to `RcModes` and related.

### Changed

- Updated to vqm 0.1.4.
- Made some additional `RcModes` and related fields public.

## [0.1.2] - 2026-05-10

### Added

- `RxConfig`.

### Changed

- `RadioControlMessage::from_rx_frame` to `RadioControlMessage::new_from`.

## [0.1.1] - 2026-05-06

### Changed

- Made `new` functions const where possible.
- Updated to latest crates.

## [0.1.0] - 2026-04-28

Initial release.
