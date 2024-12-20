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

use core::marker::PhantomData;
use msp430::asm;
use crate::{
    gpio::{Alternate1, Pin, P1, P4, Pin2, Pin3, Pin6, Pin7},
    hal::blocking::i2c::{Read, Write, WriteRead, WriteIter, Operation, TransactionalIter,
                         SevenBitAddress, TenBitAddress, Transactional},
    hw_traits::eusci::{EUsciI2C, UcbCtlw0, UcbCtlw1, UcbI2coa, UcbIe, UcbIFG, Ucssel,
                       Ucmode, Ucglit, Ucclto, Ucastp},
    pac
};
use crate::clock::{Aclk, Smclk};
use crate::hw_traits::eusci::I2CUcbIfgOut;

/// Configure bus to use 7bit or 10bit I2C slave addressing mode
#[derive(Clone, Copy)]
enum AddressingMode {
    /// 7 Bit addressing mode
    SevenBit = 0,
    /// 10 bit addressing mode
    TenBit = 1,
}

impl From<AddressingMode> for bool {
    fn from(f: AddressingMode) -> bool {
        match f {
            AddressingMode::SevenBit => false,
            AddressingMode::TenBit => true,
        }
    }
}

/// Configure between master receiver and master transmitter modes
#[derive(Clone, Copy)]
enum TransmissionMode {
    /// master receiver mode
    Receive = 0,
    /// master transmitter mode
    Transmit = 1,
}

impl From<TransmissionMode> for bool {
    fn from(f: TransmissionMode) -> bool {
        match f {
            TransmissionMode::Receive => false,
            TransmissionMode::Transmit => true,
        }
    }
}

/// Configure the automatic glitch filter on the SDA and SCL lines
#[derive(Clone, Copy)]
pub enum GlitchFilter {
    ///Pulses of maximum 50-ns length are filtered.
    Max50ns = 0,
    ///Pulses of maximum 25-ns length are filtered.
    Max25ns = 1,
    ///Pulses of maximum 12.5-ns length are filtered.
    Max12_5ns = 2,
    ///Pulses of maximum 6.25-ns length are filtered.
    Max6_25ns = 3,
}

impl From<GlitchFilter> for Ucglit {
    fn from(f: GlitchFilter) -> Ucglit {
        Ucglit::from(f as u8)
    }
}

///Struct used to configure a I2C bus
pub struct I2CBusConfig<USCI: EUsciI2CBus> {
    usci: USCI,
    divisor: u16,

    // Register configs
    ctlw0: UcbCtlw0,
    ctlw1: UcbCtlw1,
    i2coa0: UcbI2coa,
    i2coa1: UcbI2coa,
    i2coa2: UcbI2coa,
    i2coa3: UcbI2coa,
    ie: UcbIe,
    ifg: UcbIFG,
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

impl EUsciI2CBus for pac::E_USCI_B1 {
    type ClockPin = UsciB1SCLPin;
    type DataPin = UsciB1SDAPin;
}

// Allows a GPIO pin to be converted into an I2C object
macro_rules! impl_i2c_pin {
    ($struct_name: ident, $port: ty, $pin: ty) => {
        impl<DIR> From<Pin<$port, $pin, Alternate1<DIR>>> for $struct_name {
            #[inline(always)]
            fn from(_val: Pin<$port, $pin, Alternate1<DIR>>) -> Self {
                $struct_name
            }
        }
    };
}

/// I2C SCL pin for eUSCI B0
pub struct UsciB0SCLPin;
impl_i2c_pin!(UsciB0SCLPin, P1, Pin3);

/// I2C SDA pin for eUSCI B0
pub struct UsciB0SDAPin;
impl_i2c_pin!(UsciB0SDAPin, P1, Pin2);

/// I2C SCL pin for eUSCI B1
pub struct UsciB1SCLPin;
impl_i2c_pin!(UsciB1SCLPin, P4, Pin7);

/// I2C SDA pin for eUSCI B1
pub struct UsciB1SDAPin;
impl_i2c_pin!(UsciB1SDAPin, P4, Pin6);

impl<USCI: EUsciI2CBus> I2CBusConfig<USCI>{
    /// Create a new configuration for setting up a EUSCI peripheral in I2C master mode
    pub fn new(
        usci: USCI,
        )->Self{

        let ctlw0 = UcbCtlw0{
            uca10: false,
            ucsla10: false,
            ucmm: false,
            ucmst: true,
            ucsync: true,
            uctxack: false,
            uctr: false,
            uctxnack: false,
            uctxstp: false,
            uctxstt: false,
            ucswrst: true,
            ucmode: Ucmode::I2CMode,
            ucssel: Ucssel::Smclk,
        };

        let ctlw1 = UcbCtlw1{
            ucetxint: false,
            ucstpnack: false,
            ucswack: false,
            ucclto: Ucclto::Ucclto00b,
            ucastp: Ucastp::Ucastp00b,
            ucglit: Ucglit::Max6_25ns,
        };

        let i2coa0 = UcbI2coa{
            ucgcen: false,
            ucoaen: false,
            i2coa0: 0,
        };

        let i2coa1 = UcbI2coa{
            ucgcen: false,
            ucoaen: false,
            i2coa0: 0,
        };

        let i2coa2 = UcbI2coa{
            ucgcen: false,
            ucoaen: false,
            i2coa0: 0,
        };

        let i2coa3 = UcbI2coa{
            ucgcen: false,
            ucoaen: false,
            i2coa0: 0,
        };

        let ie = UcbIe{
            ucbit9ie: false,
            uctxie3: false,
            ucrxie3: false,
            uctxie2: false,
            ucrxie2: false,
            uctxie1: false,
            ucrxie1: false,
            uccltoie: false,
            ucbcntie: false,
            ucnackie: false,
            ucalie: false,
            ucstpie: false,
            ucsttie: false,
            uctxie0: false,
            ucrxie0: false,
        };

        let ifg = UcbIFG{
            ucbit9ifg: false,
            uctxifg3: false,
            ucrxifg3: false,
            uctxifg2: false,
            ucrxifg2: false,
            uctxifg1: false,
            ucrxifg1: false,
            uccltoifg: false,
            ucbcntifg: false,
            ucnackifg: false,
            ucalifg: false,
            ucstpifg: false,
            ucsttifg: false,
            uctxifg0: false,
            ucrxifg0: false,
        };

        I2CBusConfig{
            usci,
            divisor: 1,
            ctlw0,
            ctlw1,
            i2coa0,
            i2coa1,
            i2coa2,
            i2coa3,
            ie,
            ifg,
        }
    }

    /// Configures this peripheral to use smclk
    #[inline]
    pub fn use_smclk(&mut self, _smclk:&Smclk, clk_divisor:u16){
        self.ctlw0.ucssel = Ucssel::Smclk;
        self.divisor = clk_divisor;
    }

    /// Configures this peripheral to use aclk
    #[inline]
    pub fn use_aclk(&mut self, _aclk:&Aclk, clk_divisor:u16){
        self.ctlw0.ucssel = Ucssel::Aclk;
        self.divisor = clk_divisor;
    }

    /// Configures the glitch filter length for the SDA and SCL lines
    #[inline(always)]
    pub fn set_deglitch_time(&mut self, deglitch_time:GlitchFilter){
        self.ctlw1.ucglit = deglitch_time.into();
    }

    /// Performs hardware configuration and creates the SDL pin
    pub fn sdl<C: Into<USCI::ClockPin>, D: Into<USCI::DataPin>>(&self, _scl: C, _sdl: D) -> SDL<USCI>{
        self.configure();
        SDL(PhantomData)
    }

    /// Performs hardware configuration
    #[inline]
    fn configure(&self){
        self.usci.ctw0_set_rst();

        self.usci.ctw0_wr(&self.ctlw0);
        self.usci.ctw1_wr(&self.ctlw1);
        self.usci.i2coa_wr(0, &self.i2coa0);
        self.usci.i2coa_wr(1, &self.i2coa1);
        self.usci.i2coa_wr(2, &self.i2coa2);
        self.usci.i2coa_wr(3, &self.i2coa3);
        self.usci.ie_wr(&self.ie);
        self.usci.ifg_wr(&self.ifg);

        self.usci.brw_wr(self.divisor);
        self.usci.tbcnt_wr(0);

        self.usci.ctw0_clear_rst();
    }

}

/// I2C data pin
pub struct SDL<USCI: EUsciI2CBus>(PhantomData<USCI>);

/// I2C transmit/receive errors
#[derive(Clone, Copy)]
pub enum I2CErr{
    /// Function not implemented
    Unimplemented = 0,
    /// Address was never acknolwedged by slave
    GotNACK,
    /// Device lost arbitration
    ArbitrationLost,
}

impl<USCI:EUsciI2CBus> SDL<USCI>{

    #[inline(always)]
    fn set_addressing_mode(&mut self, mode:AddressingMode){
        let usci = unsafe { USCI::steal() };
        usci.set_ucsla10(mode.into())
    }

    #[inline(always)]
    fn set_transmission_mode(&mut self, mode:TransmissionMode){
        let usci = unsafe { USCI::steal() };
        usci.set_uctr(mode.into())
    }

    /// Blocking read
    fn read(&mut self, address: u16, buffer: &mut [u8]) -> Result<(), I2CErr>{
        let usci = unsafe { USCI::steal() };

        usci.i2csa_wr(address);
        usci.transmit_start();

        while usci.uctxstt_rd() {asm::nop();}

        let mut ifg = usci.ifg_rd();
        if ifg.ucnackifg() {
            usci.transmit_stop();
            while usci.uctxstp_rd() {asm::nop();}
            return Err::<(), I2CErr>(I2CErr::GotNACK);
        }

        for i in 0 .. buffer.len()-1 {
            while !ifg.ucrxifg0() {
                ifg =  usci.ifg_rd();
            }
            buffer[i] = usci.ucrxbuf_rd();
        }
        usci.transmit_stop();
        while !ifg.ucrxifg0() {ifg =  usci.ifg_rd();}
        buffer[buffer.len()-1] = usci.ucrxbuf_rd();

        while usci.uctxstp_rd() {asm::nop();}

        Ok(())
    }

    /// Blocking write
    fn write(&mut self, address: u16, bytes: &[u8]) -> Result<(), I2CErr>{
        let usci = unsafe { USCI::steal() };

        usci.i2csa_wr(address);
        usci.transmit_start();

        let mut ifg = usci.ifg_rd();
        while !ifg.uctxifg0() {ifg = usci.ifg_rd();}

        while usci.uctxstt_rd() {asm::nop();}

        ifg = usci.ifg_rd();
        if ifg.ucnackifg() {
            usci.transmit_stop();
            while usci.uctxstp_rd() {asm::nop();}
            return Err::<(), I2CErr>(I2CErr::GotNACK);
        }

        for i in 0 .. bytes.len() {
            usci.uctxbuf_wr(bytes[i]);
            ifg = usci.ifg_rd();
            while !ifg.uctxifg0() {
                ifg = usci.ifg_rd();
            }
            if ifg.ucnackifg() {
                usci.transmit_stop();
                while usci.uctxstp_rd() {asm::nop();}
                return Err::<(), I2CErr>(I2CErr::GotNACK);
            }
        }
        // usci.uctxbuf_wr(bytes[bytes.len()-1]);
        usci.transmit_stop();
        while usci.uctxstp_rd() {asm::nop();}

        Ok(())
    }

    fn write_iter<B>(&mut self, _address: u16, _bytes: B) -> Result<(), I2CErr>
        where
            B: IntoIterator<Item = u8>{

        Err(I2CErr::Unimplemented)
    }

    /// blocking write then blocking read
    fn write_read(
        &mut self,
        address: u16,
        bytes: &[u8],
        buffer: &mut [u8],
    ) -> Result<(), I2CErr>{
        self.set_transmission_mode(TransmissionMode::Transmit);
        let result = self.read(address, buffer);
        if result.is_err() {
            return result;
        }
        self.set_transmission_mode(TransmissionMode::Receive);
        self.write(address, bytes)
    }

    fn exec<'a>(&mut self, _address: u16, _operations: &mut [Operation<'a>])
                -> Result<(), I2CErr>{
        Err(I2CErr::Unimplemented)
    }

    fn exec_iter<'a, O>(&mut self, _address: u16, _operations: O) -> Result<(), I2CErr>
        where
            O: IntoIterator<Item = Operation<'a>>{
        Err(I2CErr::Unimplemented)
    }
}


impl<USCI:EUsciI2CBus> Read<SevenBitAddress> for SDL<USCI>{
    type Error = I2CErr;
    fn read(&mut self, address: u8, buffer: &mut [u8]) -> Result<(), Self::Error>{
        self.set_addressing_mode(AddressingMode::SevenBit);
        self.set_transmission_mode(TransmissionMode::Receive);
        SDL::read(self, address as u16, buffer)
    }
}

impl<USCI:EUsciI2CBus> Read<TenBitAddress> for SDL<USCI>{
    type Error = I2CErr;
    fn read(&mut self, address: u16, buffer: &mut [u8]) -> Result<(), Self::Error>{
        self.set_addressing_mode(AddressingMode::TenBit);
        self.set_transmission_mode(TransmissionMode::Receive);
        SDL::read(self, address, buffer)
    }
}

impl<USCI:EUsciI2CBus> Write<SevenBitAddress> for SDL<USCI>{
    type Error = I2CErr;
    fn write(&mut self, address: u8, bytes: &[u8]) -> Result<(), Self::Error>{
        self.set_addressing_mode(AddressingMode::SevenBit);
        self.set_transmission_mode(TransmissionMode::Transmit);
        SDL::write(self, address as u16, bytes)
    }
}

impl<USCI:EUsciI2CBus> Write<TenBitAddress> for SDL<USCI>{
    type Error = I2CErr;
    fn write(&mut self, address: u16, bytes: &[u8]) -> Result<(), Self::Error>{
        self.set_addressing_mode(AddressingMode::TenBit);
        self.set_transmission_mode(TransmissionMode::Transmit);
        SDL::write(self, address, bytes)
    }
}

impl<USCI:EUsciI2CBus> WriteIter<SevenBitAddress> for SDL<USCI>{
    type Error = I2CErr;
    fn write<B>(&mut self, address: u8, bytes: B) -> Result<(), Self::Error>
        where
            B: IntoIterator<Item = u8>{
        self.set_addressing_mode(AddressingMode::SevenBit);
        self.set_transmission_mode(TransmissionMode::Transmit);
        SDL::write_iter(self, address as u16, bytes)
    }
}

impl<USCI:EUsciI2CBus> WriteIter<TenBitAddress> for SDL<USCI>{
    type Error = I2CErr;
    fn write<B>(&mut self, address: u16, bytes: B) -> Result<(), Self::Error>
        where
            B: IntoIterator<Item = u8>{
        self.set_addressing_mode(AddressingMode::TenBit);
        self.set_transmission_mode(TransmissionMode::Transmit);
        SDL::write_iter(self, address, bytes)
    }
}

impl<USCI:EUsciI2CBus> WriteRead<SevenBitAddress> for SDL<USCI>{
    type Error = I2CErr;
    fn write_read(
        &mut self,
        address: u8,
        bytes: &[u8],
        buffer: &mut [u8],
    ) -> Result<(), Self::Error>{
        self.set_addressing_mode(AddressingMode::SevenBit);
        SDL::write_read(self, address as u16, bytes, buffer)
    }
}

impl<USCI:EUsciI2CBus> WriteRead<TenBitAddress> for SDL<USCI>{
    type Error = I2CErr;
    fn write_read(
        &mut self,
        address: u16,
        bytes: &[u8],
        buffer: &mut [u8],
    ) -> Result<(), Self::Error>{
        self.set_addressing_mode(AddressingMode::TenBit);
        SDL::write_read(self, address, bytes, buffer)
    }
}

impl<USCI:EUsciI2CBus> Transactional<SevenBitAddress> for SDL<USCI>{
    type Error = I2CErr;
    fn exec<'a>(&mut self, address: u8, operations: &mut [Operation<'a>])
                -> Result<(), Self::Error>{
        self.set_addressing_mode(AddressingMode::SevenBit);
        SDL::exec(self, address as u16, operations)
    }
}

impl<USCI:EUsciI2CBus> Transactional<TenBitAddress> for SDL<USCI>{
    type Error = I2CErr;
    fn exec<'a>(&mut self, address: u16, operations: &mut [Operation<'a>])
                -> Result<(), Self::Error>{
        self.set_addressing_mode(AddressingMode::TenBit);
        SDL::exec(self, address, operations)
    }
}

impl<USCI:EUsciI2CBus> TransactionalIter<SevenBitAddress> for SDL<USCI> {
    type Error = I2CErr;
    fn exec_iter<'a, O>(&mut self, address: u8, operations: O) -> Result<(), Self::Error>
        where
            O: IntoIterator<Item = Operation<'a>>{
        self.set_addressing_mode(AddressingMode::SevenBit);
        SDL::exec_iter(self, address as u16, operations)
    }
}

impl<USCI:EUsciI2CBus> TransactionalIter<TenBitAddress> for SDL<USCI> {
    type Error = I2CErr;
    fn exec_iter<'a, O>(&mut self, address: u16, operations: O) -> Result<(), Self::Error>
        where
            O: IntoIterator<Item = Operation<'a>>{
        self.set_addressing_mode(AddressingMode::TenBit);
        SDL::exec_iter(self, address, operations)
    }
}
