//! TODO replace
#![no_std]
#![feature(specialization)]
#![feature(asm)]
#![deny(missing_docs)]

/// GPIO batch configuration
pub mod batch_gpio;
mod bits;
/// Microcontroller clock control and selection (CS)
pub mod clock;
/// FRAM controller
pub mod fram;
/// General purpose digital I/O
pub mod gpio;
mod hw_traits;
/// Power management module
pub mod pmm;
/// Serial UART
pub mod serial;
/// Watchdog timer
pub mod watchdog;
