# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).

Releases of the form `0.1.n` do not adhere to [Semantic Versioning](https://semver.org/spec/v2.0.0.html),
that is each release may contain incompatible API changes.

Once the API has stabilized this project will adopt semantic versioning, the first release to do so will be `0.2.0`.

## [Unreleased]

### Added

- `#[must_use]` attribute to selected functions.

### Changed

- updated to `simple-bitset` version 0.1.2.
- `new` functions to `const` where possible.

### Removed

- `allow`s from `lib.rs`.

### Deprecated

### Fixed

### Security

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
