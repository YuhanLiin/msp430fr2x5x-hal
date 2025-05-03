//! SPI
//! 
//! Peripherals eUSCI_A0, eUSCI_A1, eUSCI_B0 and eUSCI_B1 can be used for SPI communication.
//! 
//! Begin by calling `SpiBusConfig::new()`. Once configured an `SpiBus` will be returned.
//! 
//! `SpiBus` implements the embedded_hal `FullDuplex` trait with non-blocking `.read()` and `.send()` methods, 
//! and the blocking embedded_hal `Transfer` and `Write` traits, with `.transfer()`  and `.write()` methods respectively.
//!
//! Pins used:
//!
//! eUSCI_A0: {MISO: `P1.7`, MOSI: `P1.6`, SCLK: `P1.5`}. `P1.4` can optionally used as a hardware-controlled chip select pin.
//!
//! eUSCI_A1: {MISO: `P4.3`, MOSI: `P4.2`, SCLK: `P4.1`}. `P4.0` can optionally used as a hardware-controlled chip select pin.
//!
//! eUSCI_B0: {MISO: `P1.3`, MOSI: `P1.2`, SCLK: `P1.1`}. `P1.0` can optionally used as a hardware-controlled chip select pin.
//!
//! eUSCI_B1: {MISO: `P4.7`, MOSI: `P4.6`, SCLK: `P4.5`}. `P4.4` can optionally used as a hardware-controlled chip select pin.
use crate::{
    clock::{Aclk, Smclk},
    gpio::{Alternate1, Pin, Pin0, Pin1, Pin2, Pin3, Pin4, Pin5, Pin6, Pin7, P1, P4},
    hw_traits::eusci::{EusciSPI, Ucmode, Ucssel, UcxSpiCtw0},
};
use core::marker::PhantomData;
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
impl_spi_pin!(UsciA0MISOPin, P1, Pin7);

/// SPI MOSI pin for eUSCI A0
pub struct UsciA0MOSIPin;
impl_spi_pin!(UsciA0MOSIPin, P1, Pin6);

/// SPI SCLK pin for eUSCI A0
pub struct UsciA0SCLKPin;
impl_spi_pin!(UsciA0SCLKPin, P1, Pin5);

/// SPI STE pin for eUSCI A0
pub struct UsciA0STEPin;
impl_spi_pin!(UsciA0STEPin, P1, Pin4);

/// SPI MISO pin for eUSCI A1
pub struct UsciA1MISOPin;
impl_spi_pin!(UsciA1MISOPin, P4, Pin3);

/// SPI MOSI pin for eUSCI A1
pub struct UsciA1MOSIPin;
impl_spi_pin!(UsciA1MOSIPin, P4, Pin2);

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
pub struct SpiBusConfig<USCI: SpiUsci, STATE> {
    usci: USCI,
    prescaler: u16,

    // Register configs
    ctlw0: UcxSpiCtw0,
    _phantom: PhantomData<STATE>,
}

impl<USCI: SpiUsci> SpiBusConfig<USCI, NoClockSet> {
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

        SpiBusConfig {
            usci,
            prescaler: 0,
            ctlw0,
            _phantom: PhantomData,
        }
    }

    /// Configures this peripheral to use smclk
    #[inline]
    pub fn use_smclk(mut self, _smclk: &Smclk, clk_divisor: u16) -> SpiBusConfig<USCI, ClockSet>{
        self.ctlw0.ucssel = Ucssel::Smclk;
        self.prescaler = clk_divisor;
        SpiBusConfig { usci: self.usci, prescaler: self.prescaler, ctlw0: self.ctlw0, _phantom: PhantomData }
    }

    /// Configures this peripheral to use aclk
    #[inline]
    pub fn use_aclk(mut self, _aclk: &Aclk, clk_divisor: u16) -> SpiBusConfig<USCI, ClockSet> {
        self.ctlw0.ucssel = Ucssel::Aclk;
        self.prescaler = clk_divisor;
        SpiBusConfig { usci: self.usci, prescaler: self.prescaler, ctlw0: self.ctlw0, _phantom: PhantomData }
    }
}
#[allow(private_bounds)]
impl<USCI: SpiUsci> SpiBusConfig<USCI, ClockSet> {
    /// Performs hardware configuration and creates an SPI bus. The STE pin is used as an automatically controlled chip select pin. Suitable for systems with only one slave device.
    #[inline(always)]
    pub fn configure_with_hardware_cs<
        SO: Into<USCI::MISO>,
        SI: Into<USCI::MOSI>,
        CLK: Into<USCI::SCLK>,
        STE: Into<USCI::STE>,
    >(
        &mut self,
        _miso: SO,
        _mosi: SI,
        _sclk: CLK,
        _cs: STE,
    ) -> SpiBus<USCI> {
        self.configure_hw();
        SpiBus(PhantomData)
    }

    /// Performs hardware configuration and creates an SPI bus. You must configure and control any chip select pins yourself. Suitable for systems with multiple slave devices. 
    #[inline(always)]
    pub fn configure_with_software_cs<
        SO: Into<USCI::MISO>,
        SI: Into<USCI::MOSI>,
        CLK: Into<USCI::SCLK>,
    >(
        &mut self,
        _miso: SO,
        _mosi: SI,
        _sclk: CLK
    ) -> SpiBus<USCI> {
        self.ctlw0.ucmode = Ucmode::ThreePinSPI;
        self.configure_hw();
        SpiBus(PhantomData)
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
pub struct SpiBus<USCI: SpiUsci>(PhantomData<USCI>);

impl<USCI: SpiUsci> SpiBus<USCI> {
    /// Enable Rx interrupts, which fire when a byte is ready to be read
    #[inline(always)]
    pub fn set_rx_interrupt(&mut self) {
        let usci = unsafe { USCI::steal() };
        usci.set_receive_interrupt();
    }

    /// Disable Rx interrupts, which fire when a byte is ready to be read
    #[inline(always)]
    pub fn clear_rx_interrupt(&mut self) {
        let usci = unsafe { USCI::steal() };
        usci.clear_receive_interrupt();
    }

    /// Enable Tx interrupts, which fire when the transmit buffer is empty
    #[inline(always)]
    pub fn set_tx_interrupt(&mut self) {
        let usci = unsafe { USCI::steal() };
        usci.set_transmit_interrupt();
    }

    /// Disable Tx interrupts, which fire when the transmit buffer is empty
    #[inline(always)]
    pub fn clear_tx_interrupt(&mut self) {
        let usci = unsafe { USCI::steal() };
        usci.clear_transmit_interrupt();
    }

    /// Writes raw value to Tx buffer with no checks for validity
    /// # Safety
    /// May clobber unsent data still in the buffer
    #[inline(always)]
    pub unsafe fn write_no_check(&mut self, val: u8) {
        let usci = unsafe { USCI::steal() };
        usci.txbuf_wr(val)
    }

    #[inline(always)]
    /// Reads a raw value from the Rx buffer with no checks for validity
    /// # Safety
    /// May read duplicate data
    pub unsafe fn read_no_check(&mut self) -> u8 {
        let usci = unsafe { USCI::steal() };
        usci.rxbuf_rd()
    }

    #[inline(always)]
    /// Change the SPI mode
    pub fn change_mode(&mut self, mode: Mode) {
        let usci = unsafe { USCI::steal() };
        usci.ctw0_set_rst();
        usci.set_spi_mode(mode);
        usci.ctw0_clear_rst();
    }
}

/// SPI transmit/receive errors
#[derive(Clone, Copy, Debug)]
#[non_exhaustive]
pub enum SPIErr {
    /// Data in the recieve buffer was overwritten before it was read. The contained data is the new contents of the recieve buffer.
    OverrunError(u8),
    // In future the framing error bit UCFE may appear here. Right now it's unimplemented.
}

#[cfg(feature = "embedded-hal-02")]
mod ehal02 {
    use embedded_hal_02::spi::FullDuplex;
    use super::*;

    impl<USCI: SpiUsci> FullDuplex<u8> for SpiBus<USCI> {
        type Error = SPIErr;
        fn read(&mut self) -> nb::Result<u8, Self::Error> {
            let usci = unsafe { USCI::steal() };
            
            if usci.receive_flag() {
                if usci.overrun_flag() {
                    Err(nb::Error::Other(SPIErr::OverrunError(usci.rxbuf_rd())))
                }
                else {
                    Ok(usci.rxbuf_rd())
                }
            } else {
                Err(WouldBlock)
            }
        }

        fn send(&mut self, word: u8) -> nb::Result<(), Self::Error> {
            let usci = unsafe { USCI::steal() };
            if usci.transmit_flag() {
                usci.txbuf_wr(word);
                Ok(())
            } else {
                Err(WouldBlock)
            }
        }
    }

    // Implementing FullDuplex above gets us a blocking write and transfer implementation for free
    impl<USCI: SpiUsci> embedded_hal_02::blocking::spi::write::Default<u8> for SpiBus<USCI> {}
    impl<USCI: SpiUsci> embedded_hal_02::blocking::spi::transfer::Default<u8> for SpiBus<USCI> {}
}
