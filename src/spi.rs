//! embedded_hal SPI implmentation

use core::marker::PhantomData;
use msp430fr2355 as pac;
use crate::hal::{
    spi as spi_nb,
    blocking::spi as spi_blocking,
};
use crate::{
    hw_traits::eusci::{EusciSPI, UcxSpiCtw0, Ucmode, Ucssel},
    gpio::{Alternate1, Pin, P1, P4, Pin0, Pin1, Pin2, Pin3, Pin4, Pin5, Pin6, Pin7},
    clock::{Smclk, Aclk}
};

/// Marks a eUSCI capable of SPI communication (in this case, all euscis do)
pub trait EUsciSPIBus : EusciSPI{
    /// Master In Slave Out (refered to as SOMI in datasheet)
    type MISO;
    /// Master Out Slave In (refered to as SIMO in datasheet)
    type MOSI;
    /// Serial Clock
    type SCLK;
    /// Slave Transmit Enable (mostly equivalent to Chip Select for single-master systems)
    type STE;
}

impl EUsciSPIBus for pac::E_USCI_A0 {
    type MISO = UsciA0MISOPin;
    type MOSI = UsciA0MOSIPin;
    type SCLK = UsciA0SCLKPin;
    type STE  = UsciA0STEPin;
}

impl EUsciSPIBus for pac::E_USCI_A1 {
    type MISO = UsciA0MISOPin;
    type MOSI = UsciA0MOSIPin;
    type SCLK = UsciA0SCLKPin;
    type STE  = UsciA0STEPin;
}

impl EUsciSPIBus for pac::E_USCI_B0 {
    type MISO = UsciA0MISOPin;
    type MOSI = UsciA0MOSIPin;
    type SCLK = UsciA0SCLKPin;
    type STE  = UsciA0STEPin;
}

impl EUsciSPIBus for pac::E_USCI_B1 {
    type MISO = UsciA0MISOPin;
    type MOSI = UsciA0MOSIPin;
    type SCLK = UsciA0SCLKPin;
    type STE  = UsciA0STEPin;
}

/// SPI MISO pin for eUSCI A0
pub struct UsciA0MISOPin;
impl<DIR> Into<UsciA0MISOPin> for Pin<P1, Pin7, Alternate1<DIR>> {
    #[inline(always)]
    fn into(self) -> UsciA0MISOPin {
        UsciA0MISOPin
    }
}

/// SPI MOSI pin for eUSCI A0
pub struct UsciA0MOSIPin;
impl<DIR> Into<UsciA0MOSIPin> for Pin<P1, Pin6, Alternate1<DIR>> {
    #[inline(always)]
    fn into(self) -> UsciA0MOSIPin {
        UsciA0MOSIPin
    }
}

/// SPI SCLK pin for eUSCI A0
pub struct UsciA0SCLKPin;
impl<DIR> Into<UsciA0SCLKPin> for Pin<P1, Pin5, Alternate1<DIR>> {
    #[inline(always)]
    fn into(self) -> UsciA0SCLKPin {
        UsciA0SCLKPin
    }
}

/// SPI STE pin for eUSCI A0
pub struct UsciA0STEPin;
impl<DIR> Into<UsciA0STEPin> for Pin<P1, Pin4, Alternate1<DIR>> {
    #[inline(always)]
    fn into(self) -> UsciA0STEPin {
        UsciA0STEPin
    }
}

/// SPI MISO pin for eUSCI A1
pub struct UsciA1MISOPin;
impl<DIR> Into<UsciA1MISOPin> for Pin<P4, Pin3, Alternate1<DIR>> {
    #[inline(always)]
    fn into(self) -> UsciA1MISOPin {
        UsciA1MISOPin
    }
}

/// SPI MOSI pin for eUSCI A1
pub struct UsciA1MOSIPin;
impl<DIR> Into<UsciA1MOSIPin> for Pin<P4, Pin2, Alternate1<DIR>> {
    #[inline(always)]
    fn into(self) -> UsciA1MOSIPin {
        UsciA1MOSIPin
    }
}

/// SPI SCLK pin for eUSCI A1
pub struct UsciA1SCLKPin;
impl<DIR> Into<UsciA1SCLKPin> for Pin<P4, Pin1, Alternate1<DIR>> {
    #[inline(always)]
    fn into(self) -> UsciA1SCLKPin {
        UsciA1SCLKPin
    }
}

/// SPI STE pin for eUSCI A1
pub struct UsciA1STEPin;
impl<DIR> Into<UsciA1STEPin> for Pin<P4, Pin0, Alternate1<DIR>> {
    #[inline(always)]
    fn into(self) -> UsciA1STEPin {
        UsciA1STEPin
    }
}

/// SPI MISO pin for eUSCI B0
pub struct UsciB0MISOPin;
impl<DIR> Into<UsciB0MISOPin> for Pin<P1, Pin3, Alternate1<DIR>> {
    #[inline(always)]
    fn into(self) -> UsciB0MISOPin {
        UsciB0MISOPin
    }
}

/// SPI MOSI pin for eUSCI B0
pub struct UsciB0MOSIPin;
impl<DIR> Into<UsciB0MOSIPin> for Pin<P1, Pin2, Alternate1<DIR>> {
    #[inline(always)]
    fn into(self) -> UsciB0MOSIPin {
        UsciB0MOSIPin
    }
}

/// SPI SCLK pin for eUSCI B0
pub struct UsciB0SCLKPin;
impl<DIR> Into<UsciB0SCLKPin> for Pin<P1, Pin1, Alternate1<DIR>> {
    #[inline(always)]
    fn into(self) -> UsciB0SCLKPin {
        UsciB0SCLKPin
    }
}

/// SPI STE pin for eUSCI B0
pub struct UsciB0STEPin;
impl<DIR> Into<UsciB0STEPin> for Pin<P1, Pin0, Alternate1<DIR>> {
    #[inline(always)]
    fn into(self) -> UsciB0STEPin {
        UsciB0STEPin
    }
}

/// SPI MISO pin for eUSCI B1
pub struct UsciB1MISOPin;
impl<DIR> Into<UsciB1MISOPin> for Pin<P4, Pin7, Alternate1<DIR>> {
    #[inline(always)]
    fn into(self) -> UsciB1MISOPin {
        UsciB1MISOPin
    }
}

/// SPI MOSI pin for eUSCI B1
pub struct UsciB1MOSIPin;
impl<DIR> Into<UsciB1MOSIPin> for Pin<P4, Pin6, Alternate1<DIR>> {
    #[inline(always)]
    fn into(self) -> UsciB1MOSIPin {
        UsciB1MOSIPin
    }
}

/// SPI SCLK pin for eUSCI B1
pub struct UsciB1SCLKPin;
impl<DIR> Into<UsciB1SCLKPin> for Pin<P4, Pin5, Alternate1<DIR>> {
    #[inline(always)]
    fn into(self) -> UsciB1SCLKPin {
        UsciB1SCLKPin
    }
}

/// SPI STE pin for eUSCI B1
pub struct UsciB1STEPin;
impl<DIR> Into<UsciB1STEPin> for Pin<P4, Pin4, Alternate1<DIR>> {
    #[inline(always)]
    fn into(self) -> UsciB1STEPin {
        UsciB1STEPin
    }
}

/// Struct used to configure a SPI bus
pub struct SPIBusConfig<USCI: EUsciSPIBus>{
    usci: USCI,
    prescaler: u16,

    // Register configs
    ctlw0: UcxSpiCtw0,
}

impl<USCI: EUsciSPIBus> SPIBusConfig<USCI>{
    /// Create a new configuration for setting up a EUSCI peripheral in SPI mode
    pub fn new(usci: USCI, fourPin:bool)->Self{
        let ctlw0 = UcxSpiCtw0{
            ucckph: false,
            ucckpl: false,
            ucmsb: false,
            uc7bit: false,
            ucmst: false,
            ucsync: false,
            ucstem: false,
            ucswrst: false,
            ucmode:  Ucmode::FourPinSPI0,
            ucssel: Ucssel::Uclk,
        };

        SPIBusConfig{
            usci: usci,
            prescaler: 0,
            ctlw0: ctlw0
        }
    }

    /// Configures this peripheral to use smclk
    #[inline]
    pub fn use_smclk(&mut self, smclk:&Smclk, clk_divisor:u16){
        self.ctlw0.ucssel = Ucssel::Smclk;
        self.prescaler = clk_divisor;
    }

    /// Configures this peripheral to use aclk
    #[inline]
    pub fn use_aclk(&mut self, aclk:&Aclk, clk_divisor:u16){
        self.ctlw0.ucssel = Ucssel::Aclk;
        self.prescaler = clk_divisor;
    }

    /// Performs hardware configuration and creates a 4 wire SPI bus
    pub fn four_wire
        <SO: Into<USCI::MISO>, SI: Into<USCI::MOSI>, CLK: Into<USCI::SCLK>, CS: Into<USCI::STE>>
        (&mut self, cs_active_high: bool, _miso: SO, _mosi: SI, _sclk: CLK, _cs:CS)
        -> SPIPins<USCI>{
        match cs_active_high {
            true => {self.ctlw0.ucmode = Ucmode::FourPinSPI1},
            false => {self.ctlw0.ucmode = Ucmode::FourPinSPI0},
        }
        self.configure_hw();
        SPIPins(PhantomData)
    }

    #[inline]
    fn configure_hw(&self){
        self.usci.ctw0_wr_rst(true);

        self.usci.ctw0_wr(&self.ctlw0);
        self.usci.brw_wr(self.prescaler);
        self.usci.uclisten_set(false);
        self.usci.transmit_interrupt_set(false);
        self.usci.receive_interrupt_set(false);

        self.usci.ctw0_wr_rst(false);
    }

}

/// Represents a group of pins configured for SPI communication
pub struct SPIPins<USCI: EUsciSPIBus>(PhantomData<USCI>);

impl<USCI:EUsciSPIBus> SPIPins<USCI>{

}











