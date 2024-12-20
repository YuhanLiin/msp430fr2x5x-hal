//! embedded_hal SPI implmentation
use core::marker::PhantomData;
use embedded_hal::spi::FullDuplex;
use msp430fr2355 as pac;
use crate::hal::spi::{Mode, Polarity, Phase};
use crate::{
    hw_traits::eusci::{EusciSPI, UcxSpiCtw0, Ucmode, Ucssel},
    gpio::{Alternate1, Pin, P1, P4, Pin0, Pin1, Pin2, Pin3, Pin4, Pin5, Pin6, Pin7},
    clock::{Smclk, Aclk},
    hal
};
use nb::Error::WouldBlock;

/// Marks a eUSCI capable of SPI communication (in this case, all euscis do)
pub trait EUsciSPIBus : EusciSPI{
    /// Master In Slave Out (refered to as SOMI in datasheet)
    type MISO;
    /// Master Out Slave In (refered to as SIMO in datasheet)
    type MOSI;
    /// Serial Clock
    type SCLK;
    /// Slave Transmit Enable (acts like CS)
    type STE;
}

impl EUsciSPIBus for pac::E_USCI_A0 {
    type MISO = UsciA0MISOPin;
    type MOSI = UsciA0MOSIPin;
    type SCLK = UsciA0SCLKPin;
    type STE  = UsciA0STEPin;
}

impl EUsciSPIBus for pac::E_USCI_A1 {
    type MISO = UsciA1MISOPin;
    type MOSI = UsciA1MOSIPin;
    type SCLK = UsciA1SCLKPin;
    type STE  = UsciA1STEPin;
}

impl EUsciSPIBus for pac::E_USCI_B0 {
    type MISO = UsciB0MISOPin;
    type MOSI = UsciB0MOSIPin;
    type SCLK = UsciB0SCLKPin;
    type STE  = UsciB0STEPin;
}

impl EUsciSPIBus for pac::E_USCI_B1 {
    type MISO = UsciB1MISOPin;
    type MOSI = UsciB1MOSIPin;
    type SCLK = UsciB1SCLKPin;
    type STE  = UsciB1STEPin;
}

// Allows a GPIO pin to be converted into an SPI object
macro_rules! impl_spi_pin{
    ($struct_name: ident, $port: ty, $pin: ty) => {
        impl<DIR> From<Pin<$port, $pin, Alternate1<DIR>>> for $struct_name {
            #[inline(always)]
            fn from(_val: Pin<$port, $pin, Alternate1<DIR>>) -> Self {
                $struct_name
            }
        }
    }
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

/// Struct used to configure a SPI bus
pub struct SPIBusConfig<USCI: EUsciSPIBus>{
    usci: USCI,
    prescaler: u16,

    // Register configs
    ctlw0: UcxSpiCtw0,
}

impl<USCI: EUsciSPIBus> SPIBusConfig<USCI>{
    /// Create a new configuration for setting up a EUSCI peripheral in SPI mode
    pub fn new(usci: USCI, mode:Mode, msb_first:bool)->Self{
        let ctlw0 = UcxSpiCtw0{
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
            ucmode:  Ucmode::FourPinSPI0,
            ucssel: Ucssel::Uclk,
        };


        SPIBusConfig{
            usci,
            prescaler: 0,
            ctlw0
        }
    }

    /// Configures this peripheral to use smclk
    #[inline]
    pub fn use_smclk(&mut self, _smclk:&Smclk, clk_divisor:u16){
        self.ctlw0.ucssel = Ucssel::Smclk;
        self.prescaler = clk_divisor;
    }

    /// Configures this peripheral to use aclk
    #[inline]
    pub fn use_aclk(&mut self, _aclk:&Aclk, clk_divisor:u16){
        self.ctlw0.ucssel = Ucssel::Aclk;
        self.prescaler = clk_divisor;
    }

    /// Performs hardware configuration and creates an SPI bus
    pub fn spi_pins
        <SO: Into<USCI::MISO>, SI: Into<USCI::MOSI>, CLK: Into<USCI::SCLK>, STE: Into<USCI::STE>>
        (&mut self, _miso: SO, _mosi: SI, _sclk: CLK, _cs : STE)
        -> SPIPins<USCI>{
        self.configure_hw();
        SPIPins(PhantomData)
    }

    #[inline]
    fn configure_hw(&self){
        self.usci.ctw0_set_rst();

        self.usci.ctw0_wr(&self.ctlw0);
        self.usci.brw_wr(self.prescaler);
        self.usci.uclisten_clear();

        self.usci.ctw0_clear_rst();

        self.usci.transmit_interrupt_set(false);
        self.usci.receive_interrupt_set(false);
    }

}

/// Represents a group of pins configured for SPI communication
pub struct SPIPins<USCI: EUsciSPIBus>(PhantomData<USCI>);


impl<USCI: EUsciSPIBus> SPIPins<USCI>{
    /// Enable or disable Rx interrupts, which fire when a byte is ready to be read
    #[inline(always)]
    pub fn rx_interrupt_set(&mut self, flag: bool) {
        let usci = unsafe { USCI::steal() };
        usci.receive_interrupt_set(flag);
    }

    /// Enable or disable Tx interrupts, which fire when the transmit buffer is empty
    #[inline(always)]
    pub fn tx_interrupt_set(&mut self, flag: bool) {
        let usci = unsafe { USCI::steal() };
        usci.transmit_interrupt_set(flag);
    }

    /// Writes raw value to Tx buffer with no checks for validity
    #[inline(always)]
    pub fn write_no_check(&mut self, val: u8){
        let usci = unsafe { USCI::steal() };
        usci.txbuf_wr(val)
    }
}

/// SPI transmit/receive errors
#[derive(Clone, Copy)]
pub enum SPIErr{
    /// Function not implemented
    Unimplemented = 0,
}

impl<USCI:EUsciSPIBus> FullDuplex<u8> for SPIPins<USCI>{
    type Error = SPIErr;
    fn read(&mut self) -> nb::Result<u8, Self::Error>{
        let usci = unsafe{USCI::steal()};
        if usci.receive_flag() {
            Ok(usci.rxbuf_rd())
        }else{
            Err(WouldBlock)
        }
    }

    fn send(&mut self, word: u8) -> nb::Result<(), Self::Error>{
        let usci = unsafe{USCI::steal()};
        if usci.transmit_flag() {
            usci.txbuf_wr(word);
            Ok(())
        }else{
            Err(WouldBlock)
        }
    }
}

// Implementing FullDuplex above gets us a blocking write and transfer implementation for free
impl<USCI: EUsciSPIBus> embedded_hal::blocking::spi::write::Default<u8> for SPIPins<USCI> {}
impl<USCI: EUsciSPIBus> embedded_hal::blocking::spi::transfer::Default<u8> for SPIPins<USCI> {}
