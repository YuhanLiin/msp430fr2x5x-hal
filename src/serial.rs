use crate::clock::{Aclk, Clock, Smclk};
use crate::hw_traits::eusci::{EUsci, EUsciUart, Ucssel};
use msp430fr2355 as pac;

/// Bit order of transmit and receive
pub enum BitOrder {
    /// LSB first (typically the default)
    LsbFirst,
    /// MSB first
    MsbFirst,
}

/// Number of bits per transaction
pub enum BitCount {
    /// 8 bits
    EightBits,
    /// 7 bits
    SevenBits,
}

/// Number of stop bits at end of each byte
pub enum StopBits {
    /// 1 stop bit
    OneStopBit,
    /// 2 stop bits
    TwoStopBits,
}

/// Parity bit for error checking
pub enum Parity {
    /// No parity
    NoParity,
    /// Odd parity
    OddParity,
    /// Even parity
    EvenParity,
}

/// Configuration object for serial UART without clock select info, which is required before a
/// Serial object can be created.
pub struct SerialConfigNoClock<USCI: EUsciUart> {
    usci: USCI,
    order: BitOrder,
    cnt: BitCount,
    stopbits: StopBits,
    parity: Parity,
}

/// Configuration object for serial UART with clock select info.
pub struct SerialConfig<USCI: EUsciUart> {
    config: SerialConfigNoClock<USCI>,
    clksel: Ucssel,
    freq: u32,
}

/// Extension trait for converting the proper PAC E_USCI peripherals into Serial objects
pub trait SerialExt: EUsciUart + Sized {
    /// Begin configuring the peripheral into a Serial object
    fn to_serial(self) -> SerialConfigNoClock<Self>;
}

impl<USCI: EUsciUart> SerialConfigNoClock<USCI> {
    /// Configure serial UART to use external UCLK, passing in the appropriately configured pin
    /// used as the clock signal as well as the frequency of the clock.
    pub fn use_uclk(self, freq: u32) -> SerialConfig<USCI> {
        // TODO pass in appropriate pin type
        SerialConfig {
            config: self,
            clksel: Ucssel::Uclk,
            freq,
        }
    }

    /// Configure serial UART to use ACLK.
    pub fn use_aclk(self, aclk: &Aclk) -> SerialConfig<USCI> {
        SerialConfig {
            config: self,
            clksel: Ucssel::Aclk,
            freq: aclk.freq() as u32,
        }
    }

    /// Configure serial UART to use SMCLK.
    pub fn use_smclk(self, smclk: &Smclk) -> SerialConfig<USCI> {
        SerialConfig {
            config: self,
            clksel: Ucssel::Smclk,
            freq: smclk.freq(),
        }
    }
}
