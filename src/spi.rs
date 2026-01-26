//! SPI
//!
//! Peripherals eUSCI_A0, eUSCI_A1, eUSCI_B0 and eUSCI_B1 can be used for SPI communication as either a master or slave device.
//!
//! Begin by calling [`SpiConfig::new()`]. Once configured either an [`Spi`] or [`SpiSlave`] will be returned.
//!
//! Note that even if you are only using the legacy embedded-hal 0.2.7 trait implementations, configuration of the SPI bus
//! uses the embedded-hal 1.0 versions of types (e.g. [`Mode`]).
//!
//! # [`Spi`]
//! The SPI peripheral can be configured as a master device by calling one of the
//! [`to_master()`](SpiConfig::to_master_using_smclk) methods during configuration.
//!
//! [`Spi`] implements the embedded-hal [`SpiBus`](embedded_hal::spi::SpiBus) trait, which provides a simple blocking interface.
//! A non-blocking implementation is also available through [`embedded-hal-nb`](embedded_hal_nb)'s
//! [`FullDuplex`](embedded_hal_nb::spi::FullDuplex) trait.
//! Standalone methods are also provided for directly writing to the Tx and Rx buffers for interrupt-based implementations.
//!
//! # [`SpiSlave`]
//! The SPI peripheral can be configured as a slave device by calling [`to_slave()`](SpiConfig::to_slave) during configuration.
//!
//! [`SpiSlave`] supports sharing the bus with other slave devices by calling the [`shared_bus()`](SpiConfig::shared_bus) method
//! during configuration. In this mode the STE pin controls whether the MISO pin is an output or a high-impedance pin, allowing
//! other slaves to use the MISO bus when this device is not selected. The polarity of the STE pin is configurable to either
//! active high or active low.
//! If the bus is used exclusively by this device then the [`exclusive_bus()`](SpiConfig::exclusive_bus) configuration method
//! can be used, which allows the STE pin to be used for other purposes. In this mode the MISO pin will remain an output pin at
//! all times.
//!
//! [`SpiSlave`] provides non-blocking methods that can be used for polling or interrupt-based implementations.
//! It does not implement either of the embedded-hal traits.
//!
//! Pins used:
//!
//! |          |  MISO  |  MOSI  |  SCLK  |  STE  |
//! |:--------:|:------:|:------:|:------:|:-----:|
//! | eUSCI_A0 | `P1.6` | `P1.7` | `P1.5` | `P1.4`|
//! | eUSCI_A1 | `P4.2` | `P4.3` | `P4.1` | `P4.0`|
//! | eUSCI_B0 | `P1.3` | `P1.2` | `P1.1` | `P1.0`|
//! | eUSCI_B1 | `P4.7` | `P4.6` | `P4.5` | `P4.4`|
use crate::{
    clock::{Aclk, Smclk},
    hw_traits::eusci::{EusciSPI, Ucmode, Ucssel, UcxSpiCtw0},
};
use core::{convert::Infallible, marker::PhantomData};
use nb::Error::WouldBlock;

use embedded_hal::spi::{Mode, Phase, Polarity};

/// Marks a eUSCI capable of SPI communication (in this case, all euscis do)
pub trait SpiUsci: EusciSPI {
    /// Master In Slave Out (refered to as SOMI in datasheet)
    type MISO;
    /// Master Out Slave In (refered to as SIMO in datasheet)
    type MOSI;
    /// Serial Clock
    type SCLK;
    /// Slave Transmit Enable (acts like CS)
    type STE;
}

// Allows a GPIO pin to be converted into an SPI object
macro_rules! impl_spi_pin {
    ($struct_name: ident, $port: ty, $pin: ty) => {
        impl<DIR> From<Pin<$port, $pin, Alternate1<DIR>>> for $struct_name {
            #[inline(always)]
            fn from(_val: Pin<$port, $pin, Alternate1<DIR>>) -> Self {
                $struct_name
            }
        }
    };
}
pub(crate) use impl_spi_pin;

/// Typestate for an SPI bus whose role has not yet been chosen.
pub struct RoleNotSet;
/// Typestate for an SPI bus being configured as a master device.
pub struct Master;
/// Typestate for an SPI bus being configured as a slave device.
pub struct Slave;

/// Configuration object for an eUSCI peripheral being set up for SPI mode.
pub struct SpiConfig<USCI: SpiUsci, ROLE> {
    usci: USCI,
    ctlw0: UcxSpiCtw0,
    prescaler: u16,
    _phantom: PhantomData<ROLE>,
}

impl<USCI: SpiUsci> SpiConfig<USCI, RoleNotSet> {
    /// Begin configuring an EUSCI peripheral for SPI mode.
    pub fn new(usci: USCI, mode: Mode, msb_first: bool) -> Self {
        let ctlw0 = UcxSpiCtw0 {
            ucckph: match mode.phase {
                Phase::CaptureOnFirstTransition => true,
                Phase::CaptureOnSecondTransition => false,
            },
            ucckpl: match mode.polarity {
                Polarity::IdleLow => false,
                Polarity::IdleHigh => true,
            },
            ucmsb: msb_first,
            ucsync: true,
            ucswrst: true,
            // UCSTEM = 1 isn't useful for us, since the STE acts like a CS pin in this case, but
            // it asserts and de-asserts after each byte automatically, and unfortunately
            // ehal::SpiBus requires support for multi-byte transactions.
            ucstem: false,
            uc7bit: false, // Not supported
            ..Default::default()
        };

        Self { usci, ctlw0, prescaler: 0, _phantom: PhantomData }
    }
    /// This device will act as a slave on the SPI bus.
    pub fn to_slave(mut self) -> SpiConfig<USCI, Slave> {
        self.ctlw0.ucmst = false;
        // UCSSEL is 'don't care' in slave mode
        SpiConfig { usci: self.usci, prescaler: self.prescaler, ctlw0: self.ctlw0, _phantom: PhantomData }
    }
    /// This device will act as a master on the SPI bus, deriving SCLK from SMCLK.
    pub fn to_master_using_smclk(mut self, _smclk: &Smclk, clk_div: u16) -> SpiConfig<USCI, Master> {
        self.ctlw0.ucmst = true;
        self.ctlw0.ucssel = Ucssel::Smclk;
        self.prescaler = clk_div;
        SpiConfig { usci: self.usci, prescaler: self.prescaler, ctlw0: self.ctlw0, _phantom: PhantomData }
    }
    /// This device will act as a master on the SPI bus, deriving SCLK from ACLK.
    pub fn to_master_using_aclk(mut self, _aclk: &Aclk, clk_div: u16) -> SpiConfig<USCI, Master> {
        self.ctlw0.ucmst = true;
        self.ctlw0.ucssel = Ucssel::Aclk;
        self.prescaler = clk_div;
        SpiConfig { usci: self.usci, prescaler: self.prescaler, ctlw0: self.ctlw0, _phantom: PhantomData }
    }
}
impl<USCI: SpiUsci> SpiConfig<USCI, Master> {
    // Note: Errata USCI50 makes this mode a real pain to implement. Leave out for now.
    // /// For an SPI bus with more than one master.
    // /// The STE pin is used by the other master to turn SCLK and MOSI high impedance, so the other master can talk on the bus.
    // pub fn multi_master_bus<MOSI, MISO, SCLK, STE>(mut self, _miso: MISO, _mosi: MOSI, _sclk: SCLK, _ste: STE, ste_pol: StePolarity) -> Spi<USCI>
    // where MOSI: Into<USCI::MOSI>, MISO: Into<USCI::MISO>, SCLK: Into<USCI::SCLK>, STE: Into<USCI::STE> {
    //     // TODO: UCMODE
    //     self.configure_hw();
    //     Spi(PhantomData)
    // }
    /// For an SPI bus with a single master.
    /// SCLK and MOSI are always outputs. The STE pin is not required.
    pub fn single_master_bus<MOSI, MISO, SCLK>(mut self, _miso: MISO, _mosi: MOSI, _sclk: SCLK) -> Spi<USCI>
    where MOSI: Into<USCI::MOSI>, MISO: Into<USCI::MISO>, SCLK: Into<USCI::SCLK> {
        self.ctlw0.ucmode = Ucmode::ThreePinSPI;
        self.configure_hw();
        Spi { usci: self.usci }
    }
}
impl<USCI: SpiUsci> SpiConfig<USCI, Slave> {
    /// For an SPI bus with more than one slave.
    /// The STE pin is used to turn MISO high impedance, so other slaves can talk on the bus.
    pub fn shared_bus<MOSI, MISO, SCLK, STE>(mut self, _miso: MISO, _mosi: MOSI, _sclk: SCLK, _ste: STE, ste_pol: StePolarity) -> SpiSlave<USCI> 
    where MOSI: Into<USCI::MOSI>, MISO: Into<USCI::MISO>, SCLK: Into<USCI::SCLK>, STE: Into<USCI::STE> {
        self.ctlw0.ucmode = match ste_pol {
            StePolarity::EnabledWhenHigh => Ucmode::FourPinSPI1,
            StePolarity::EnabledWhenLow  => Ucmode::FourPinSPI0,
        };
        self.configure_hw();
        SpiSlave { usci: self.usci }
    }
    /// For an SPI bus where this device is the only slave.
    /// MOSI is always an output. 
    pub fn exclusive_bus<MOSI, MISO, SCLK>(mut self, _miso: MISO, _mosi: MOSI, _sclk: SCLK) -> SpiSlave<USCI> 
    where MOSI: Into<USCI::MOSI>, MISO: Into<USCI::MISO>, SCLK: Into<USCI::SCLK> {
        self.ctlw0.ucmode = Ucmode::ThreePinSPI;
        self.configure_hw();
        SpiSlave { usci: self.usci }
    }
}
impl<USCI: SpiUsci, ROLE> SpiConfig<USCI, ROLE> {
    #[inline]
    fn configure_hw(&self) {
        self.usci.ctw0_set_rst();

        self.usci.ctw0_wr(&self.ctlw0);
        self.usci.brw_wr(self.prescaler);
        self.usci.uclisten_clear();

        self.usci.ctw0_clear_rst();

        self.usci.clear_transmit_interrupt();
        self.usci.clear_receive_interrupt();
    }
}

/// The polarity of the STE pin.
pub enum StePolarity {
    /// This device is enabled when STE is high.
    EnabledWhenHigh = 0b01,
    /// This device is enabled when STE is low.
    EnabledWhenLow = 0b10,
}

macro_rules! spi_common {
    () => {
        /// Enable Rx interrupts, which fire when a byte is ready to be read
        #[inline(always)]
        pub fn set_rx_interrupt(&mut self) {
            self.usci.set_receive_interrupt();
        }

        /// Disable Rx interrupts, which fire when a byte is ready to be read
        #[inline(always)]
        pub fn clear_rx_interrupt(&mut self) {
            self.usci.clear_receive_interrupt();
        }

        /// Enable Tx interrupts, which fire when the transmit buffer is empty
        #[inline(always)]
        pub fn set_tx_interrupt(&mut self) {
            self.usci.set_transmit_interrupt();
        }

        /// Disable Tx interrupts, which fire when the transmit buffer is empty
        #[inline(always)]
        pub fn clear_tx_interrupt(&mut self) {
            self.usci.clear_transmit_interrupt();
        }

        /// Write a byte into the Tx buffer, without checking if the Tx buffer is empty. Returns immediately.
        /// Useful if you already know the buffer is empty (e.g. a Tx interrupt was triggered)
        /// # Safety
        /// May clobber previous unsent data if the TXIFG bit is not set.
        #[inline(always)]
        pub unsafe fn write_unchecked(&mut self, val: u8) {
            self.usci.txbuf_wr(val)
        }

        /// Read the byte in the Rx buffer, without checking if the Rx buffer is ready.
        /// Useful when you already know the buffer is ready (e.g. an Rx interrupt was triggered).
        /// # Safety
        /// May read invalid data if RXIFG bit is not ready.
        #[inline]
        pub unsafe fn read_unchecked(&mut self) -> Result<u8, SpiErr> {
            if self.usci.overrun_flag() {
                return Err(SpiErr::Overrun(self.usci.rxbuf_rd()));
            }
            Ok(self.usci.rxbuf_rd())
        }

        fn recv_byte(&mut self) -> nb::Result<u8, SpiErr> {
            if self.usci.receive_flag() {
                if self.usci.overrun_flag() {
                    Err(nb::Error::Other(SpiErr::Overrun(self.usci.rxbuf_rd())))
                } else {
                    Ok(self.usci.rxbuf_rd())
                }
            } else {
                Err(WouldBlock)
            }
        }

        fn send_byte(&mut self, word: u8) -> nb::Result<(), Infallible> {
            if self.usci.transmit_flag() {
                self.usci.txbuf_wr(word);
                Ok(())
            } else {
                Err(WouldBlock)
            }
        }

        /// Get the source of the interrupt currently being serviced.
        #[inline]
        pub fn interrupt_source(&mut self) -> SpiVector {
            match self.usci.iv_rd() {
                0 => SpiVector::None,
                2 => SpiVector::RxBufferFull,
                4 => SpiVector::TxBufferEmpty,
                _ => unsafe { core::hint::unreachable_unchecked() },
            }
        }
    };
}

/// Possible sources for an eUSCI SPI interrupt
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum SpiVector {
    /// No interrupt is currently being serviced.
    None = 0,
    /// The interrupt was caused by the Rx buffer being full.
    RxBufferFull = 2,
    /// The interrupt was caused by the Tx buffer being empty.
    TxBufferEmpty = 4,
}

/// Represents a group of pins configured for SPI communication
pub struct Spi<USCI: SpiUsci>{usci: USCI}
impl<USCI: SpiUsci> Spi<USCI> {
    spi_common!();

    #[inline(always)]
    /// Change the SPI mode. This requires resetting the peripheral, which also sets TXIFG and clears RXIFG, UCOE, and UCFE.
    pub fn change_mode(&mut self, mode: Mode) {
        let intrs = self.usci.ie_rd();
        self.usci.ctw0_set_rst();
        self.usci.set_spi_mode(mode);
        self.usci.ie_wr(intrs);
        self.usci.ctw0_clear_rst();
    }
}

/// An eUSCI peripheral that has been configured into an SPI slave.
pub struct SpiSlave<USCI: SpiUsci>{usci: USCI}
impl<USCI: SpiUsci> SpiSlave<USCI> {
    spi_common!();

    /// Try to read from the Rx buffer. Returns `nb::WouldBlock` if the buffer is empty.
    #[inline(always)]
    pub fn read(&mut self) -> nb::Result<u8, SpiErr> {
        self.recv_byte()
    }

    /// Try to write a byte into the Tx buffer. Returns `nb::WouldBlock` if the buffer is still full. Returns immediately.
    #[inline(always)]
    pub fn write(&mut self, byte: u8) -> nb::Result<(), Infallible> {
        self.send_byte(byte)
    }
}

/// SPI transmit/receive errors
#[derive(Clone, Copy, Debug)]
pub enum SpiErr {
    /// Data in the recieve buffer was overwritten before it was read. The contained data is the new contents of the recieve buffer.
    Overrun(u8),
}
impl From<Infallible> for SpiErr {
    fn from(value: Infallible) -> Self {
        match value {}
    }
}

mod ehal1 {
    use super::*;
    use embedded_hal::spi::{Error, ErrorType, SpiBus};
    use nb::block;

    impl Error for SpiErr {
        fn kind(&self) -> embedded_hal::spi::ErrorKind {
            match self {
                SpiErr::Overrun(_) => embedded_hal::spi::ErrorKind::Overrun,
            }
        }
    }

    impl<USCI: SpiUsci> ErrorType for Spi<USCI> {
        type Error = SpiErr;
    }

    impl<USCI: SpiUsci> SpiBus for Spi<USCI> {
        /// Send dummy packets (`0x00`) on MOSI so the slave can respond on MISO. Store the response in `words`.
        fn read(&mut self, words: &mut [u8]) -> Result<(), Self::Error> {
            for word in words {
                block!(self.send_byte(0x00))?;
                *word = block!(self.recv_byte())?;
            }
            Ok(())
        }

        /// Write `words` to the slave, ignoring all the incoming words.
        ///
        /// Returns as soon as the last word is placed in the hardware buffer.
        fn write(&mut self, words: &[u8]) -> Result<(), Self::Error> {
            for word in words {
                block!(self.send_byte(*word))?;
                let _ = block!(self.recv_byte());
            }
            Ok(())
        }

        /// Write and read simultaneously. `write` is written to the slave on MOSI and
        /// words received on MISO are stored in `read`.
        ///
        /// If `write` is longer than `read`, then after `read` is full any subsequent incoming words will be discarded.
        /// If `read` is longer than `write`, then dummy packets of `0x00` are sent until `read` is full.
        fn transfer(&mut self, read: &mut [u8], write: &[u8]) -> Result<(), Self::Error> {
            let mut read_bytes = read.iter_mut();
            let mut write_bytes = write.iter();
            const DUMMY_WRITE: u8 = 0x00;
            let mut dummy_read = 0;

            // Pair up read and write bytes (inserting dummy values as necessary) until everything's sent
            loop {
                let (rd, wr) = match (read_bytes.next(), write_bytes.next()) {
                    (Some(rd), Some(wr)) => (rd, wr),
                    (Some(rd), None    ) => (rd, &DUMMY_WRITE),
                    (None,     Some(wr)) => (&mut dummy_read, wr),
                    (None,     None    ) => break,
                };

                block!(self.send_byte(*wr))?;
                *rd = block!(self.recv_byte())?;
            }
            Ok(())
        }

        /// Write and read simultaneously. The contents of `words` are
        /// written to the slave, and the received words are stored into the same
        /// `words` buffer, overwriting it.
        fn transfer_in_place(&mut self, words: &mut [u8]) -> Result<(), Self::Error> {
            for word in words {
                block!(self.send_byte(*word))?;
                *word = block!(self.recv_byte())?;
            }
            Ok(())
        }

        fn flush(&mut self) -> Result<(), Self::Error> {
            while self.usci.is_busy() {}
            Ok(())
        }
    }
}

mod ehal_nb1 {
    use super::*;
    use embedded_hal_nb::{nb, spi::FullDuplex};

    impl<USCI: SpiUsci> FullDuplex<u8> for Spi<USCI> {
        fn read(&mut self) -> nb::Result<u8, Self::Error> {
            self.recv_byte()
        }

        fn write(&mut self, word: u8) -> nb::Result<(), Self::Error> {
            self.send_byte(word).map_err(map_infallible)
        }
    }
}

#[cfg(feature = "embedded-hal-02")]
mod ehal02 {
    use super::*;
    use embedded_hal_02::spi::FullDuplex;

    impl<USCI: SpiUsci> FullDuplex<u8> for Spi<USCI> {
        type Error = SpiErr;
        fn read(&mut self) -> nb::Result<u8, Self::Error> {
            self.recv_byte()
        }

        fn send(&mut self, word: u8) -> nb::Result<(), Self::Error> {
            self.send_byte(word).map_err(map_infallible)
        }
    }

    // Implementing FullDuplex above gets us a blocking write and transfer implementation for free
    impl<USCI: SpiUsci> embedded_hal_02::blocking::spi::write::Default<u8> for Spi<USCI> {}
    impl<USCI: SpiUsci> embedded_hal_02::blocking::spi::transfer::Default<u8> for Spi<USCI> {}
}

// Unfortunately the compiler can't always automatically infer this, even though we already have From<Infallible> for SpiErr
fn map_infallible<E>(err: nb::Error<Infallible>) -> nb::Error<E> {
    match err {
        WouldBlock => WouldBlock,
    }
}
