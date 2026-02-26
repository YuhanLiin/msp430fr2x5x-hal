// This file re-exports device-specific implementation details in a generic manner.
// The actual business logic for device-specific implementations are handled in the device_specific/ folder.

// MSP430FR2x5x series
#[cfg(feature = "2x5x")]
#[path="device_specific/fr2x5x.rs"]
pub mod device;

// MSP430FR2433
#[cfg(feature = "msp430fr2433")]
#[path="device_specific/fr2433.rs"]
pub mod device;

pub use device::*;
