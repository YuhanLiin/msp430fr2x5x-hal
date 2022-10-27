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
//! $PATH. Invoke `xargo run --example whatever` with the board plugged and the scripts should do
//! the trick, assuming your host is Linux and you are connected via Launchpad.

#![no_std]
#![allow(incomplete_features)] // Enable specialization without warnings
#![feature(specialization)]
#![feature(asm_experimental_arch)]
#![deny(missing_docs)]

pub mod batch_gpio;
pub mod capture;
pub mod clock;
pub mod fram;
pub mod gpio;
pub mod pmm;
pub mod prelude;
pub mod pwm;
pub mod rtc;
pub mod serial;
pub mod timer;
pub mod watchdog;

mod hw_traits;
mod util;

pub use msp430fr2355 as pac;
