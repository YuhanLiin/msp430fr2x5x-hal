//! Implementation of [`embedded_hal`] traits for MSP430FR2x5x family of microcontrollers.
//! Here are the [`datasheet`] and [`User's guide`] for reference.
//!
//! As of this writing, the only supported board is the MSP430FR2355.
//!
//! [`embedded_hal`]: https://github.com/rust-embedded/embedded-hal
//! [`datasheet`]: http://www.ti.com/lit/ds/symlink/msp430fr2355.pdf
//! [`User's guide`]: http://www.ti.com/lit/ug/slau445i/slau445i.pdf
//!
//! # Usage
//!
//! Requires `msp430-elf-gcc` installed and in $PATH to build
//!
//! When using this crate as a dependency, make sure you include the appropriate `memory.x` file for
//! your microcontroller.
//!
//! # Examples
//!
//! The `examples/` directory contains binary code examples using the HAL abstractions.
//! To flash the examples, make sure you have `mspdebug` with `tilib` support installed and in
//! $PATH. Invoke `cargo run --example whatever` with the board plugged and the scripts should do
//! the trick, assuming your host is Linux and you are connected via Launchpad.
//!
//! # Features
//!
//! An implementation of the pre-1.0 version of embedded-hal (e.g. 0.2.7 at time of writing) is
//! available behind the `embedded-hal-02` feature flag. These traits are implemented on the same
//! structs as the current embedded-hal implementation, so with this feature enabled you may mix and
//! match crates that require the pre-1.0 version with those that require the latest version. It isn't enabled by
//! default, as many of the trait names are similar (or identical) to their counterparts in the current
//! version, which can be confusing.

#![no_std]
#![allow(incomplete_features)] // Enable specialization without warnings
#![feature(specialization)]
#![feature(asm_experimental_arch)]

#![allow(stable_features)] // Feature flags used on older compiler versions
#![feature(const_option)]

#![deny(missing_docs)]

pub mod adc;
pub mod bak_mem;
pub mod batch_gpio;
pub mod capture;
pub mod clock;
pub mod crc;
pub mod delay;
pub mod ecomp;
pub mod fram;
pub mod gpio;
pub mod i2c;
pub mod info_mem;
pub mod lpm;
pub mod pmm;
pub mod prelude;
pub mod pwm;
pub mod rtc;
pub mod sac;
pub mod serial;
pub mod spi;
pub mod timer;
pub mod watchdog;

mod device_specific;
mod hw_traits;
mod util;

pub use device_specific::pac;

#[cfg(feature = "embedded-hal-02")]
pub use embedded_hal_02 as ehal_02;

pub use embedded_hal as ehal;
