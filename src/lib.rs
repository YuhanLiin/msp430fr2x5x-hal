//! TODO replace
#![no_std]
#![feature(specialization)]
#![deny(missing_docs)]

/// GPIO batch configuration
pub mod batch_gpio;
mod bits;
/// Microcontroller clock control and selection (CS)
pub mod clock;
/// General purpose digital I/O
pub mod gpio;
mod hw_traits;
/// Power management module
pub mod pmm;
/// Watchdog timer
pub mod watchdog;
