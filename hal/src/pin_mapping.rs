//! Pin mapping strategies for peripherals.
//!
//! Many peripherals support multiple pin multiplexing configurations depending on the
//! device's port mapping or remapping capabilities. This module provides marker types
//! that describe which pin configuration a peripheral should use.
//!
//! The mapping type is used at compile time to select the correct implementation for a
//! peripheral. HAL drivers may use this information to configure device registers that
//! control pin routing or remapping.
//!
//! In device configuration crates, the mapping implementation typically performs the
//! required register operations to enable the selected pin layout before the peripheral
//! is initialized.
//!
//! Three common mapping strategies are provided:
//!
//! * `DefaultMapping` — Uses the primary pin layout defined by the device.
//! * `RemappedMapping` — Uses an alternate pin layout enabled through a remapping register.
//! * `FixedMapping` — For peripherals with a single, non-configurable pin layout.
//!
//! Peripheral implementations select one of these mapping strategies when implementing
//! traits such as `SerialUsci<M>`, allowing the HAL to remain generic while supporting
//! multiple device pin configurations.

/// Trait for types that define a specific pin multiplexing strategy.
pub trait PinMap {}

/// Use the primary/default pin configuration for the peripheral.
pub struct DefaultMapping;
impl PinMap for DefaultMapping {}

/// Use the alternate/secondary pin configuration for the peripheral.
pub struct RemappedMapping;
impl PinMap for RemappedMapping {}

/// For peripherals with a single, non-configurable pin layout.
/// Performs no register operations.
pub struct FixedMapping;
impl PinMap for FixedMapping {}
