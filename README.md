# `msp430fr2x5x-hal`

> A high-level Hardware Abstraction Layer (HAL) for the MSP430FR2xxx / 4xxx series of microcontrollers.

[![Crates.io](https://img.shields.io/crates/v/msp430fr2x5x-hal.svg)](https://crates.io/crates/msp430fr2x5x-hal)
[![Docs.rs](https://docs.rs/msp430fr2x5x-hal/badge.svg)](https://docs.rs/msp430fr2x5x-hal)
[![CI](https://github.com/YuhanLiin/msp430fr2x5x-hal/actions/workflows/build.yml/badge.svg)](https://github.com/YuhanLiin/msp430fr2x5x-hal/actions)
[![License](https://img.shields.io/crates/l/msp430fr2x5x-hal.svg)](https://crates.io/crates/msp430fr2x5x-hal)
[![MSRV](https://img.shields.io/badge/rust-1.82%2B-blue.svg)](https://www.rust-lang.org)

This crate is primarily designed to be used as a dependency in another project, but 
the examples in the repo can be build and flashed directly to a device, provided the 
required dependencies are installed.

# Dependencies

To build the examples [`msp430-gcc`](https://www.ti.com/tool/MSP430-GCC-OPENSOURCE) 
should be available on your PATH.

To flash an example [`mspdebug`](https://dlbeer.co.nz/mspdebug/) should also be 
available on your PATH.

# Usage

The `device-examples/` folder contains typical projects for various supported devices, each of which containing a number of examples
They can be built by moving into the relevant project folder, then running `cargo build --example <example_name>`

An example can be flashed to a connected device with 
`cargo run --example <example_name>`

# Supported Devices
The library currently supports the MSP430FR2x5x and MSP430FR247x subfamilies, and the MSP430FR2433.
Support for other devices in the MSP430FR2xxx/4xxx family is possible, see [Supporting additional devices](##Supporting-additional-devices). 

The device being targetted is must be specified by enabling exactly one device feature, such as 
`msp430fr2355`. This is required to build any code from this library.

### Currently Supported Devices
| Device | Feature name |
| ------ | ------------ |
| MSP430FR2476 | `msp430fr2476` |
| MSP430FR2475 | `msp430fr2475` |
| MSP430FR2433 | `msp430fr2433` |
| MSP430FR2355 | `msp430fr2355` |
| MSP430FR2353 | `msp430fr2353` |
| MSP430FR2155 | `msp430fr2155` |
| MSP430FR2153 | `msp430fr2153` |

The documentation on crates.rs (and example programs) target the MSP430FR2355. Documentation for a particular device can be 
built by running `cargo doc --open --features <device>` from within the `hal/` folder, or `cargo doc --open --package msp430fr2x5x-hal` in a 
cargo project with `msp430fr2x5x-hal` correctly configured as a dependency, such as the projects in the `device-examples/` folder.

## Supporting additional devices

The maintainers don't have access to every device in the MSP430FR2xxx / 4xxx family, so if you want to add support for a particular device (or subfamily) we are happy to accept pull requests.
To add support for a device (or subfamily) you should fork this repo and:
1. Add a new device feature in `hal/cargo.toml` and determine which features should be derived for your device.
2. Create a new file in `hal/src/device_specific/`, import a Peripheral Access Crate (PAC) for your device, and follow `hal/src/device_specific/msp430fr2x5x.rs` as an example to see how to e.g. mark which GPIO pins are capable of what functionality.
3. Append an entry to `hal/src/device_specific.rs` to re-export your PAC and any device-specific constants to the rest of the library.
4. Add a project crate to `device_examples/` and add some examples to test everything works.
5. Add your device name to the CI in `.github/workflows/build.yml` and check that the CI passes.

# Functionality
The library is mostly feature complete for the MSP430FR2x5x subfamily. There are a few edge cases not yet supported, such as:
- Arbitrary DCO clock speed support (currently supports 1, 2, 4, 8, 12, 16, 20, 24 MHz)
- External oscillator support
- Some RTC clock sources (currently only supports SMCLK and VLOCLK)
- ADC reference voltage selection

If you encounter any use cases not supported please open an issue (or submit a pull request).

## Other MSP430FR2xxx / 4xxx MCUs

The library currently lacks support for some peripherals available on other MSP430FR2xxx / 4xxx devices, namely:
- LCD driver
- CapTIvate
- TIA
- SAC-L1

PRs with implementations for these peripherals are welcome.

# Feature Flags

In addition to the device feature flags mentioned above, this crate provides an implementation of the legacy 0.2.7 version of embedded-hal behind the `embedded-hal-02` feature. Support for embedded-hal 1.0 is available by default.

Support for `defmt` is available through the `defmt` feature. See `device_examples/msp430fr2355/defmt.rs` for a defmt implementation on the MSP430.

# Minimum Supported Rust Version (MSRV)

This crate requires the `nightly` toolchain to compile, currently targetting `nightly-2024-09-01` 
(Rust 1.82) or later. It might compile with older versions but that may change in any new patch release.

# Assumptions

The HAL provides a maximal set of GPIO pins (targetting the package with the most pins). If you 
are using one of the variants with fewer pins then it is up to you to ensure that the GPIO pins 
you use are in fact available.
For example, on the 28-pin variant of the MSP430FR2355 eUSCI_B1 only supports I2C due to missing pins. 
This is not checked in the HAL. 

# Panics

The library is intended to be panic-free, though this hasn't been verified. If you encounter panics
while using the library (or `panic-never` points to the existence of possible panics) please open an issue.

# License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  http://www.apache.org/licenses/LICENSE-2.0)

- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
