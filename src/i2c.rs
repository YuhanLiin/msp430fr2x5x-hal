//! I2C
//!
//! Peripherals eUSCI_B0 and eUSCI_B1 can be used for I2C communication.
//!
//! Begin by calling `I2cBusConfig::new()`. Once configured an `I2cBus` will be returned.
//! 
//! `I2cBus` implements the blocking embedded_hal `Read`, `Write` and `WriteRead` traits. 
//! Passing a `u8` address to these methods uses 7-bit addressing, passing a `u16` uses 10-bit addressing.
//! 
//! Pins used:
//!
//! eUSCI_B0: {SCL: `P1.3`, SDA: `P1.2`}. `P1.1` can optionally be used as an external clock source.
//!
//! eUSCI_B1: {SCL: `P4.7`, SDA: `P4.6`}. `P4.5` can optionally be used as an external clock source.
//!

use crate::clock::{Aclk, Smclk};
use crate::gpio::{Pin1, Pin5};
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
pub struct I2CBusConfig<USCI: I2cUsci, STATE> {
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
    _phantom: PhantomData<STATE>,
}

/// Marks a usci capable of I2C communication
pub trait I2cUsci: EUsciI2C {
    /// I2C SCL pin
    type ClockPin;
    /// I2C SDA pin
    type DataPin;
    /// I2C external clock source pin. Only necessary if UCLKI is selected as a clock source.
    type ExternalClockPin;
}

impl I2cUsci for pac::E_USCI_B0 {
    type ClockPin = UsciB0SCLPin;
    type DataPin = UsciB0SDAPin;
    type ExternalClockPin = UsciB0UCLKIPin;
}

impl I2cUsci for pac::E_USCI_B1 {
    type ClockPin = UsciB1SCLPin;
    type DataPin = UsciB1SDAPin;
    type ExternalClockPin = UsciB1UCLKIPin;
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

/// UCLKI pin for eUSCI B0. Used as an external clock source.
pub struct UsciB0UCLKIPin;
impl_i2c_pin!(UsciB0UCLKIPin, P1, Pin1);

/// I2C SCL pin for eUSCI B1
pub struct UsciB1SCLPin;
impl_i2c_pin!(UsciB1SCLPin, P4, Pin7);

/// I2C SDA pin for eUSCI B1
pub struct UsciB1SDAPin;
impl_i2c_pin!(UsciB1SDAPin, P4, Pin6);

/// UCLKI pin for eUSCI B1. Used as an external clock source.
pub struct UsciB1UCLKIPin;
impl_i2c_pin!(UsciB1UCLKIPin, P4, Pin5);

/// Typestate for an I2C bus configuration with no clock source selected
pub struct NoClockSet;
/// Typestate for an I2C bus configuration with a clock source selected
pub struct ClockSet;

impl<USCI: I2cUsci> I2CBusConfig<USCI, NoClockSet> {
    /// Create a new configuration for setting up a EUSCI peripheral in I2C master mode
    pub fn new(usci: USCI, deglitch_time: GlitchFilter) -> I2CBusConfig<USCI, NoClockSet> {
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
            ucssel: Ucssel::Smclk, // overwritten by `use_smclk/uclk/aclk()`
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
            _phantom: PhantomData,
        }
    }

    /// Configures this peripheral to use SMCLK
    #[inline]
    pub fn use_smclk(mut self, _smclk: &Smclk, clk_divisor: u16) -> I2CBusConfig<USCI, ClockSet> {
        self.ctlw0.ucssel = Ucssel::Smclk;
        self.divisor = clk_divisor;
        I2CBusConfig{ 
            usci: self.usci, 
            divisor: self.divisor, 
            ctlw0: self.ctlw0, 
            ctlw1: self.ctlw1, 
            i2coa0: self.i2coa0, 
            i2coa1: self.i2coa1, 
            i2coa2: self.i2coa2, 
            i2coa3: self.i2coa3, 
            ie: self.ie, 
            ifg: self.ifg, 
            _phantom: PhantomData }
    }

    /// Configures this peripheral to use ACLK
    #[inline]
    pub fn use_aclk(mut self, _aclk: &Aclk, clk_divisor: u16) -> I2CBusConfig<USCI, ClockSet> {
        self.ctlw0.ucssel = Ucssel::Aclk;
        self.divisor = clk_divisor;
        I2CBusConfig{ 
            usci: self.usci, 
            divisor: self.divisor, 
            ctlw0: self.ctlw0, 
            ctlw1: self.ctlw1, 
            i2coa0: self.i2coa0, 
            i2coa1: self.i2coa1, 
            i2coa2: self.i2coa2, 
            i2coa3: self.i2coa3, 
            ie: self.ie, 
            ifg: self.ifg, 
            _phantom: PhantomData }
    }
    /// Configures this peripheral to use UCLK
    #[inline]
    pub fn use_uclk<Pin: Into<USCI::ExternalClockPin> >(mut self, _uclk: Pin, clk_divisor: u16) -> I2CBusConfig<USCI, ClockSet> {
        self.ctlw0.ucssel = Ucssel::Uclk;
        self.divisor = clk_divisor;
        I2CBusConfig{ 
            usci: self.usci, 
            divisor: self.divisor, 
            ctlw0: self.ctlw0, 
            ctlw1: self.ctlw1, 
            i2coa0: self.i2coa0, 
            i2coa1: self.i2coa1, 
            i2coa2: self.i2coa2, 
            i2coa3: self.i2coa3, 
            ie: self.ie, 
            ifg: self.ifg, 
            _phantom: PhantomData }
    }
}

#[allow(private_bounds)]
impl<USCI: I2cUsci> I2CBusConfig<USCI, ClockSet> {
    /// Performs hardware configuration and creates the I2C bus
    pub fn configure<C: Into<USCI::ClockPin>, D: Into<USCI::DataPin>>(
        &self,
        _scl: C,
        _sda: D,
    ) -> I2cBus<USCI> {
        self.configure_regs();
        I2cBus(PhantomData)
    }

    /// Performs hardware configuration
    #[inline]
    fn configure_regs(&self) {
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

/// I2C data bus
pub struct I2cBus<USCI: I2cUsci>(PhantomData<USCI>);

/// I2C transmit/receive errors
#[derive(Clone, Copy, Debug)]
#[non_exhaustive]
pub enum I2CErr {
    /// Address was never acknolwedged by slave
    GotNACK,
    /// Device lost arbitration
    ArbitrationLost,
    // Other errors such as the 'clock low timeout' UCCLTOIFG may appear here in future.
}

impl<USCI: I2cUsci> I2cBus<USCI> {
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

        let len = buffer.len();
        for (idx, byte) in buffer.iter_mut().enumerate() {
            if idx == len - 1 {
                usci.transmit_stop();
            }
            while !ifg.ucrxifg0() {
                ifg = usci.ifg_rd();
            }
            *byte = usci.ucrxbuf_rd();
        }

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
        self.write(address, bytes)?;
        self.set_transmission_mode(TransmissionMode::Receive);
        self.read(address, buffer)
    }
}

impl<USCI: I2cUsci> Read<SevenBitAddress> for I2cBus<USCI> {
    type Error = I2CErr;
    fn read(&mut self, address: u8, buffer: &mut [u8]) -> Result<(), Self::Error> {
        self.set_addressing_mode(AddressingMode::SevenBit);
        self.set_transmission_mode(TransmissionMode::Receive);
        I2cBus::read(self, address as u16, buffer)
    }
}

impl<USCI: I2cUsci> Read<TenBitAddress> for I2cBus<USCI> {
    type Error = I2CErr;
    fn read(&mut self, address: u16, buffer: &mut [u8]) -> Result<(), Self::Error> {
        self.set_addressing_mode(AddressingMode::TenBit);
        self.set_transmission_mode(TransmissionMode::Receive);
        I2cBus::read(self, address, buffer)
    }
}

impl<USCI: I2cUsci> Write<SevenBitAddress> for I2cBus<USCI> {
    type Error = I2CErr;
    fn write(&mut self, address: u8, bytes: &[u8]) -> Result<(), Self::Error> {
        self.set_addressing_mode(AddressingMode::SevenBit);
        self.set_transmission_mode(TransmissionMode::Transmit);
        I2cBus::write(self, address as u16, bytes)
    }
}

impl<USCI: I2cUsci> Write<TenBitAddress> for I2cBus<USCI> {
    type Error = I2CErr;
    fn write(&mut self, address: u16, bytes: &[u8]) -> Result<(), Self::Error> {
        self.set_addressing_mode(AddressingMode::TenBit);
        self.set_transmission_mode(TransmissionMode::Transmit);
        I2cBus::write(self, address, bytes)
    }
}

impl<USCI: I2cUsci> WriteRead<SevenBitAddress> for I2cBus<USCI> {
    type Error = I2CErr;
    fn write_read(
        &mut self,
        address: u8,
        bytes: &[u8],
        buffer: &mut [u8],
    ) -> Result<(), Self::Error> {
        self.set_addressing_mode(AddressingMode::SevenBit);
        I2cBus::write_read(self, address as u16, bytes, buffer)
    }
}

impl<USCI: I2cUsci> WriteRead<TenBitAddress> for I2cBus<USCI> {
    type Error = I2CErr;
    fn write_read(
        &mut self,
        address: u16,
        bytes: &[u8],
        buffer: &mut [u8],
    ) -> Result<(), Self::Error> {
        self.set_addressing_mode(AddressingMode::TenBit);
        I2cBus::write_read(self, address, bytes, buffer)
    }
}
