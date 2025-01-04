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

use crate::clock::{Aclk, Smclk};
use crate::hw_traits::eusci::I2CUcbIfgOut;
use crate::{
    gpio::{Alternate1, Pin, Pin2, Pin3, Pin6, Pin7, P1, P4},
    hal::blocking::i2c::{
        Read, SevenBitAddress, TenBitAddress, Write, WriteRead,
    },
    hw_traits::eusci::{
        EUsciI2C, Ucastp, UcbCtlw0, UcbCtlw1, UcbI2coa, UcbIFG, UcbIe, Ucclto, Ucglit, Ucmode,
        Ucssel,
    },
    pac,
};
use core::marker::PhantomData;
use msp430::asm;

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
        match f {
            GlitchFilter::Max50ns => Ucglit::Max50ns,
            GlitchFilter::Max25ns => Ucglit::Max25ns,
            GlitchFilter::Max12_5ns => Ucglit::Max12_5ns,
            GlitchFilter::Max6_25ns => Ucglit::Max6_25ns,
        }
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
pub trait EUsciI2CBus: EUsciI2C {
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

impl<USCI: EUsciI2CBus> I2CBusConfig<USCI> {
    /// Create a new configuration for setting up a EUSCI peripheral in I2C master mode
    pub fn new(usci: USCI, deglitch_time: GlitchFilter) -> Self {
        let ctlw0 = UcbCtlw0 {
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

        let ctlw1 = UcbCtlw1 {
            ucetxint: false,
            ucstpnack: false,
            ucswack: false,
            ucclto: Ucclto::Ucclto00b,
            ucastp: Ucastp::Ucastp00b,
            ucglit: deglitch_time.into(),
        };

        let i2coa0 = UcbI2coa {
            ucgcen: false,
            ucoaen: false,
            i2coa0: 0,
        };

        let i2coa1 = UcbI2coa {
            ucgcen: false,
            ucoaen: false,
            i2coa0: 0,
        };

        let i2coa2 = UcbI2coa {
            ucgcen: false,
            ucoaen: false,
            i2coa0: 0,
        };

        let i2coa3 = UcbI2coa {
            ucgcen: false,
            ucoaen: false,
            i2coa0: 0,
        };

        let ie = UcbIe {
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

        let ifg = UcbIFG {
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

        I2CBusConfig {
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
    pub fn use_smclk(&mut self, _smclk: &Smclk, clk_divisor: u16) {
        self.ctlw0.ucssel = Ucssel::Smclk;
        self.divisor = clk_divisor;
    }

    /// Configures this peripheral to use aclk
    #[inline]
    pub fn use_aclk(&mut self, _aclk: &Aclk, clk_divisor: u16) {
        self.ctlw0.ucssel = Ucssel::Aclk;
        self.divisor = clk_divisor;
    }

    /// Performs hardware configuration and creates the SDL pin
    pub fn sdl<C: Into<USCI::ClockPin>, D: Into<USCI::DataPin>>(
        &self,
        _scl: C,
        _sdl: D,
    ) -> SDL<USCI> {
        self.configure();
        SDL(PhantomData)
    }

    /// Performs hardware configuration
    #[inline]
    fn configure(&self) {
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
pub enum I2CErr {
    /// Address was never acknolwedged by slave
    GotNACK,
    /// Device lost arbitration
    ArbitrationLost,
}

impl<USCI: EUsciI2CBus> SDL<USCI> {
    #[inline(always)]
    fn set_addressing_mode(&mut self, mode: AddressingMode) {
        let usci = unsafe { USCI::steal() };
        usci.set_ucsla10(mode.into())
    }

    #[inline(always)]
    fn set_transmission_mode(&mut self, mode: TransmissionMode) {
        let usci = unsafe { USCI::steal() };
        usci.set_uctr(mode.into())
    }

    /// Blocking read
    fn read(&mut self, address: u16, buffer: &mut [u8]) -> Result<(), I2CErr> {
        if buffer.is_empty() { return Ok(()) }

        let usci = unsafe { USCI::steal() };

        usci.i2csa_wr(address);
        usci.transmit_start();

        while usci.uctxstt_rd() {
            asm::nop();
        }

        let mut ifg = usci.ifg_rd();
        if ifg.ucnackifg() {
            usci.transmit_stop();
            while usci.uctxstp_rd() {
                asm::nop();
            }
            return Err::<(), I2CErr>(I2CErr::GotNACK);
        }

        for i in 0..buffer.len() - 1 {
            while !ifg.ucrxifg0() {
                ifg = usci.ifg_rd();
            }
            buffer[i] = usci.ucrxbuf_rd();
        }
        usci.transmit_stop();
        while !ifg.ucrxifg0() {
            ifg = usci.ifg_rd();
        }
        buffer[buffer.len() - 1] = usci.ucrxbuf_rd();

        while usci.uctxstp_rd() {
            asm::nop();
        }

        Ok(())
    }

    /// Blocking write
    fn write(&mut self, address: u16, bytes: &[u8]) -> Result<(), I2CErr> {
        if bytes.is_empty() { return Ok(()) }
        let usci = unsafe { USCI::steal() };

        usci.i2csa_wr(address);
        usci.transmit_start();

        let mut ifg = usci.ifg_rd();
        while !ifg.uctxifg0() {
            ifg = usci.ifg_rd();
        }

        while usci.uctxstt_rd() {
            asm::nop();
        }

        ifg = usci.ifg_rd();
        if ifg.ucnackifg() {
            usci.transmit_stop();
            while usci.uctxstp_rd() {
                asm::nop();
            }
            return Err::<(), I2CErr>(I2CErr::GotNACK);
        }

        for &byte in bytes {
            usci.uctxbuf_wr(byte);
            ifg = usci.ifg_rd();
            while !ifg.uctxifg0() {
                ifg = usci.ifg_rd();
            }
            if ifg.ucnackifg() {
                usci.transmit_stop();
                while usci.uctxstp_rd() {
                    asm::nop();
                }
                return Err::<(), I2CErr>(I2CErr::GotNACK);
            }
        }
        // usci.uctxbuf_wr(bytes[bytes.len()-1]);
        usci.transmit_stop();
        while usci.uctxstp_rd() {
            asm::nop();
        }

        Ok(())
    }

    /// blocking write then blocking read
    fn write_read(&mut self, address: u16, bytes: &[u8], buffer: &mut [u8]) -> Result<(), I2CErr> {
        self.set_transmission_mode(TransmissionMode::Transmit);
        self.read(address, buffer)?;
        self.set_transmission_mode(TransmissionMode::Receive);
        self.write(address, bytes)
    }
}

impl<USCI: EUsciI2CBus> Read<SevenBitAddress> for SDL<USCI> {
    type Error = I2CErr;
    fn read(&mut self, address: u8, buffer: &mut [u8]) -> Result<(), Self::Error> {
        self.set_addressing_mode(AddressingMode::SevenBit);
        self.set_transmission_mode(TransmissionMode::Receive);
        SDL::read(self, address as u16, buffer)
    }
}

impl<USCI: EUsciI2CBus> Read<TenBitAddress> for SDL<USCI> {
    type Error = I2CErr;
    fn read(&mut self, address: u16, buffer: &mut [u8]) -> Result<(), Self::Error> {
        self.set_addressing_mode(AddressingMode::TenBit);
        self.set_transmission_mode(TransmissionMode::Receive);
        SDL::read(self, address, buffer)
    }
}

impl<USCI: EUsciI2CBus> Write<SevenBitAddress> for SDL<USCI> {
    type Error = I2CErr;
    fn write(&mut self, address: u8, bytes: &[u8]) -> Result<(), Self::Error> {
        self.set_addressing_mode(AddressingMode::SevenBit);
        self.set_transmission_mode(TransmissionMode::Transmit);
        SDL::write(self, address as u16, bytes)
    }
}

impl<USCI: EUsciI2CBus> Write<TenBitAddress> for SDL<USCI> {
    type Error = I2CErr;
    fn write(&mut self, address: u16, bytes: &[u8]) -> Result<(), Self::Error> {
        self.set_addressing_mode(AddressingMode::TenBit);
        self.set_transmission_mode(TransmissionMode::Transmit);
        SDL::write(self, address, bytes)
    }
}

impl<USCI: EUsciI2CBus> WriteRead<SevenBitAddress> for SDL<USCI> {
    type Error = I2CErr;
    fn write_read(
        &mut self,
        address: u8,
        bytes: &[u8],
        buffer: &mut [u8],
    ) -> Result<(), Self::Error> {
        self.set_addressing_mode(AddressingMode::SevenBit);
        SDL::write_read(self, address as u16, bytes, buffer)
    }
}

impl<USCI: EUsciI2CBus> WriteRead<TenBitAddress> for SDL<USCI> {
    type Error = I2CErr;
    fn write_read(
        &mut self,
        address: u16,
        bytes: &[u8],
        buffer: &mut [u8],
    ) -> Result<(), Self::Error> {
        self.set_addressing_mode(AddressingMode::TenBit);
        SDL::write_read(self, address, bytes, buffer)
    }
}
