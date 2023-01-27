//! I2C
//!
//! Peripherals eUSCI_B0 and eUSCI_B1 can be used for I2C communication.
//!
//! Pins used:
//!
//! eUSCI_B0: {SCL:P1.3, SDA:P1.2}
//!
//! eUSCI_B1: {SCL:P4.7, SDA:P4.6}
//!

use crate::{
    gpio::{Alternate1, Pin, P1, Pin2, Pin3, Pin6, Pin7},
    hal::blocking::i2c::{Read, Write, WriteRead},
    hw_traits::eusci::{EUsciI2C},
    pac,
};

/// Configure bus to use 7bit or 10bit I2C slave addressing mode
#[derive(Clone, Copy)]
pub enum AddressingMode {
    /// 7 Bit addressing mode
    SevenBit,
    /// 10 bit addressing mode
    TenBit,
}

/// Configures bus to act as master or slave I2C device
#[derive(Clone, Copy)]
pub enum MasterSlaveMode {
    ///Device acts as master, sends data to slaves.
    MasterReceiver,
    ///Device acts as master, requests data from slaves.
    MasterTransmitter,
    /// Device acts as slave.
    /// Automatically acts as receiver or transmitter depending on R/W bit from master.
    Slave,
}

/// Configures the automatic glitch filter on the SDA and SCL lines
#[derive(Clone, Copy)]
pub enum GlitchFilter {
    ///Pulses of maximum 50-ns length are filtered.
    Max50ns,
    ///Pulses of maximum 25-ns length are filtered.
    Max25ns,
    ///Pulses of maximum 12.5-ns length are filtered.
    Max12_5ns,
    ///Pulses of maximum 6.25-ns length are filtered.
    Max6_25ns,
}

///Struct used to configure a I2C bus
pub struct I2CBusConfig<USCI: EUsciI2CBus> {
    usci: USCI,
    addressing_mode: AddressingMode,
    master_slave_mode: MasterSlaveMode,
    glitch_filter: GlitchFilter,

}

/// Marks a usci capable of I2C communication
pub trait EUsciI2CBus : EUsciI2C{
    /// I2C SCL
    type ClockPin;
    /// I2C SDA
    type DataPin;
}

impl EUsciI2CBus for pac::E_USCI_B0 {
    type ClockPin = UsciB0SCLPin;
    type DataPin = UsciB0SDAPin;
}

/// I2C SCL pin for eUSCI B0
pub struct UsciB0SCLPin;
impl<DIR> Into<UsciB0SCLPin> for Pin<P1, Pin3, Alternate1<DIR>> {
    #[inline(always)]
    fn into(self) -> UsciB0SCLPin {
        UsciB0SCLPin
    }
}

/// I2C SDA pin for eUSCI B0
pub struct UsciB0SDAPin;
impl<DIR> Into<UsciB0SDAPin> for Pin<P1, Pin2, Alternate1<DIR>> {
    #[inline(always)]
    fn into(self) -> UsciB0SDAPin {
        UsciB0SDAPin
    }
}


impl<USCI: EUsciI2CBus> I2CBusConfig<USCI>{

}
