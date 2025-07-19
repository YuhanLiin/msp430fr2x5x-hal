//! SPI
//! 
//! Peripherals eUSCI_A0, eUSCI_A1, eUSCI_B0 and eUSCI_B1 can be used for SPI communication.
//! Currently there is only support for the MSP430 to act as the master.
//!
//! Begin by calling [`SpiConfig::new()`]. Once configured an [`Spi`] will be returned. 
//!
//! Note that even if you are only using the legacy embedded-hal 0.2.7 trait implementations, configuration of the SPI bus uses the embedded-hal 1.0 versions of types (e.g. [`Mode`]).
//! 
//! [`Spi`] implements the embedded-hal [`SpiBus`](embedded_hal::spi::SpiBus) trait. 
//! 
//! [`Spi`] also provides a non-blocking implementation through [`embedded-hal-nb`](embedded_hal_nb)'s 
//! [`FullDuplex`](embedded_hal_nb::spi::FullDuplex) trait.
//!
//! Pins used:
//!
//! eUSCI_A0: {MISO: `P1.7`, MOSI: `P1.6`, SCLK: `P1.5`}.
//!
//! eUSCI_A1: {MISO: `P4.3`, MOSI: `P4.2`, SCLK: `P4.1`}.
//!
//! eUSCI_B0: {MISO: `P1.3`, MOSI: `P1.2`, SCLK: `P1.1`}.
//!
//! eUSCI_B1: {MISO: `P4.7`, MOSI: `P4.6`, SCLK: `P4.5`}.
use crate::{
    clock::{Aclk, Smclk}, 
    delay::SysDelay, 
    gpio::{Alternate1, Pin, Pin0, Pin1, Pin2, Pin3, Pin4, Pin5, Pin6, Pin7, P1, P4}, 
    hw_traits::eusci::{EusciSPI, Ucmode, Ucssel, UcxSpiCtw0},
};
use core::{convert::Infallible, marker::PhantomData};
use msp430fr2355 as pac;
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

impl SpiUsci for pac::E_USCI_A0 {
    type MISO = UsciA0MISOPin;
    type MOSI = UsciA0MOSIPin;
    type SCLK = UsciA0SCLKPin;
    type STE = UsciA0STEPin;
}

impl SpiUsci for pac::E_USCI_A1 {
    type MISO = UsciA1MISOPin;
    type MOSI = UsciA1MOSIPin;
    type SCLK = UsciA1SCLKPin;
    type STE = UsciA1STEPin;
}

impl SpiUsci for pac::E_USCI_B0 {
    type MISO = UsciB0MISOPin;
    type MOSI = UsciB0MOSIPin;
    type SCLK = UsciB0SCLKPin;
    type STE = UsciB0STEPin;
}

impl SpiUsci for pac::E_USCI_B1 {
    type MISO = UsciB1MISOPin;
    type MOSI = UsciB1MOSIPin;
    type SCLK = UsciB1SCLKPin;
    type STE = UsciB1STEPin;
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

/// SPI MISO pin for eUSCI A0
pub struct UsciA0MISOPin;
impl_spi_pin!(UsciA0MISOPin, P1, Pin6);

/// SPI MOSI pin for eUSCI A0
pub struct UsciA0MOSIPin;
impl_spi_pin!(UsciA0MOSIPin, P1, Pin7);

/// SPI SCLK pin for eUSCI A0
pub struct UsciA0SCLKPin;
impl_spi_pin!(UsciA0SCLKPin, P1, Pin5);

/// SPI STE pin for eUSCI A0
pub struct UsciA0STEPin;
impl_spi_pin!(UsciA0STEPin, P1, Pin4);

/// SPI MISO pin for eUSCI A1
pub struct UsciA1MISOPin;
impl_spi_pin!(UsciA1MISOPin, P4, Pin2);

/// SPI MOSI pin for eUSCI A1
pub struct UsciA1MOSIPin;
impl_spi_pin!(UsciA1MOSIPin, P4, Pin3);

/// SPI SCLK pin for eUSCI A1
pub struct UsciA1SCLKPin;
impl_spi_pin!(UsciA1SCLKPin, P4, Pin1);
/// SPI STE pin for eUSCI A1
pub struct UsciA1STEPin;
impl_spi_pin!(UsciA1STEPin, P4, Pin0);

/// SPI MISO pin for eUSCI B0
pub struct UsciB0MISOPin;
impl_spi_pin!(UsciB0MISOPin, P1, Pin3);

/// SPI MOSI pin for eUSCI B0
pub struct UsciB0MOSIPin;
impl_spi_pin!(UsciB0MOSIPin, P1, Pin2);

/// SPI SCLK pin for eUSCI B0
pub struct UsciB0SCLKPin;
impl_spi_pin!(UsciB0SCLKPin, P1, Pin1);

/// SPI STE pin for eUSCI B0
pub struct UsciB0STEPin;
impl_spi_pin!(UsciB0STEPin, P1, Pin0);

/// SPI MISO pin for eUSCI B1
pub struct UsciB1MISOPin;
impl_spi_pin!(UsciB1MISOPin, P4, Pin7);

/// SPI MOSI pin for eUSCI B1
pub struct UsciB1MOSIPin;
impl_spi_pin!(UsciB1MOSIPin, P4, Pin6);

/// SPI SCLK pin for eUSCI B1
pub struct UsciB1SCLKPin;
impl_spi_pin!(UsciB1SCLKPin, P4, Pin5);

/// SPI STE pin for eUSCI B1
pub struct UsciB1STEPin;
impl_spi_pin!(UsciB1STEPin, P4, Pin4);

/// Typestate for an SPI bus configuration with no clock source selected
pub struct NoClockSet;
/// Typestate for an SPI bus configuration with a clock source selected
pub struct ClockSet;

/// Struct used to configure a SPI bus
pub struct SpiConfig<USCI: SpiUsci, STATE> {
    usci: USCI,
    prescaler: u16,

    // Register configs
    ctlw0: UcxSpiCtw0,
    _phantom: PhantomData<STATE>,
}

impl<USCI: SpiUsci> SpiConfig<USCI, NoClockSet> {
    /// Create a new configuration for setting up a EUSCI peripheral in SPI mode
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
            uc7bit: false,
            ucmst: true,
            ucsync: true,
            ucstem: true,
            ucswrst: true,
            ucmode: Ucmode::FourPinSPI0, // overwritten by `configure_with_software_cs()`
            ucssel: Ucssel::Smclk, // overwritten by `use_smclk/aclk()`
        };

        SpiConfig {
            usci,
            prescaler: 0,
            ctlw0,
            _phantom: PhantomData,
        }
    }

    /// Configures this peripheral to use smclk
    #[inline]
    pub fn use_smclk(mut self, _smclk: &Smclk, clk_divisor: u16) -> SpiConfig<USCI, ClockSet>{
        self.ctlw0.ucssel = Ucssel::Smclk;
        self.prescaler = clk_divisor;
        SpiConfig { usci: self.usci, prescaler: self.prescaler, ctlw0: self.ctlw0, _phantom: PhantomData }
    }

    /// Configures this peripheral to use aclk
    #[inline]
    pub fn use_aclk(mut self, _aclk: &Aclk, clk_divisor: u16) -> SpiConfig<USCI, ClockSet> {
        self.ctlw0.ucssel = Ucssel::Aclk;
        self.prescaler = clk_divisor;
        SpiConfig { usci: self.usci, prescaler: self.prescaler, ctlw0: self.ctlw0, _phantom: PhantomData }
    }
}
#[allow(private_bounds)]
impl<USCI: SpiUsci> SpiConfig<USCI, ClockSet> {
    /// Performs hardware configuration and creates an [SPI *bus*](embedded_hal::spi::SpiBus).
    /// You must configure and control any chip select pins yourself. 
    #[inline(always)]
    pub fn configure<
        SO: Into<USCI::MISO>,
        SI: Into<USCI::MOSI>,
        CLK: Into<USCI::SCLK>,
    >(
        mut self,
        _miso: SO,
        _mosi: SI,
        _sclk: CLK
    ) -> Spi<USCI> {
        self.ctlw0.ucmode = Ucmode::ThreePinSPI;
        self.configure_hw();
        Spi{ usci: self.usci }
    }

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

/// Represents a group of pins configured for SPI communication
pub struct Spi<USCI: SpiUsci>{usci: USCI}

impl<USCI: SpiUsci> Spi<USCI> {
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

    /// Writes raw value to Tx buffer with no checks for validity
    /// # Safety
    /// May clobber unsent data still in the buffer
    #[inline(always)]
    pub unsafe fn write_no_check(&mut self, val: u8) {
        self.usci.txbuf_wr(val)
    }

    #[inline(always)]
    /// Reads a raw value from the Rx buffer with no checks for validity
    /// # Safety
    /// May read duplicate data
    pub unsafe fn read_no_check(&mut self) -> u8 {
        self.usci.rxbuf_rd()
    }

    #[inline(always)]
    /// Change the SPI mode
    pub fn change_mode(&mut self, mode: Mode) {
        self.usci.ctw0_set_rst();
        self.usci.set_spi_mode(mode);
        self.usci.ctw0_clear_rst();
    }

    fn recv_byte(&mut self) -> nb::Result<u8, SpiErr> {
        if self.usci.receive_flag() {
            if self.usci.overrun_flag() {
                Err(nb::Error::Other(SpiErr::Overrun(self.usci.rxbuf_rd())))
            }
            else {
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
    use embedded_hal::spi::{Error, ErrorType, SpiBus};
    use nb::block;
    use super::*;

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
    use embedded_hal_nb::{nb, spi::FullDuplex};
    use super::*;

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
    use embedded_hal_02::spi::FullDuplex;
    use super::*;

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
        nb::Error::Other(e) => match e {},
    }
}
