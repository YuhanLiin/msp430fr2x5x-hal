//! I2C
//!
//! Peripherals eUSCI_B0 and eUSCI_B1 can be used for I2C communication.
//!
//! Begin by calling [`I2cConfig::new()`]. Once configured an [`I2cPeriph`] will be returned.
//! 
//! [`I2cPeriph`] implements the blocking embedded_hal [`I2c`](embedded_hal::i2c::I2c) trait. 
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
    hw_traits::eusci::{
        EUsciI2C, Ucastp, UcbCtlw0, UcbCtlw1, UcbI2coa, UcbIFG, UcbIe, Ucclto, Ucglit, Ucmode,
        Ucssel,
    },
    pac,
};
use core::marker::PhantomData;
use embedded_hal::i2c::{AddressMode, SevenBitAddress, TenBitAddress};
use msp430::asm;

/// Enumerates the two I2C addressing modes: 7-bit and 10-bit.
/// 
/// Used internally by the HAL.
#[derive(Clone, Copy)]
pub enum AddressingMode {
    /// 7-bit addressing mode
    SevenBit = 0,
    /// 10-bit addressing mode
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
pub struct I2cConfig<USCI: I2cUsci, STATE> {
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

impl<USCI: I2cUsci> I2cConfig<USCI, NoClockSet> {
    /// Create a new configuration for setting up a EUSCI peripheral in I2C master mode
    pub fn new(usci: USCI, deglitch_time: GlitchFilter) -> I2cConfig<USCI, NoClockSet> {
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

        I2cConfig {
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
    pub fn use_smclk(mut self, _smclk: &Smclk, clk_divisor: u16) -> I2cConfig<USCI, ClockSet> {
        self.ctlw0.ucssel = Ucssel::Smclk;
        self.divisor = clk_divisor;
        I2cConfig{ 
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
    pub fn use_aclk(mut self, _aclk: &Aclk, clk_divisor: u16) -> I2cConfig<USCI, ClockSet> {
        self.ctlw0.ucssel = Ucssel::Aclk;
        self.divisor = clk_divisor;
        I2cConfig{ 
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
    pub fn use_uclk<Pin: Into<USCI::ExternalClockPin> >(mut self, _uclk: Pin, clk_divisor: u16) -> I2cConfig<USCI, ClockSet> {
        self.ctlw0.ucssel = Ucssel::Uclk;
        self.divisor = clk_divisor;
        I2cConfig{ 
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
impl<USCI: I2cUsci> I2cConfig<USCI, ClockSet> {
    /// Performs hardware configuration and creates the I2C bus
    pub fn configure<C: Into<USCI::ClockPin>, D: Into<USCI::DataPin>>(
        self,
        _scl: C,
        _sda: D,
    ) -> I2cPeriph<USCI> {
        self.configure_regs();
        I2cPeriph{ usci: self.usci }
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
pub struct I2cPeriph<USCI: I2cUsci>{usci: USCI}

/// I2C transmit/receive errors
#[derive(Clone, Copy, Debug)]
#[non_exhaustive]
pub enum I2CErr {
    /// Address was never acknolwedged by slave
    GotNACK,
    // Other errors such as 'arbitration lost' and the 'clock low timeout' UCCLTOIFG may appear here in future.
}

impl<USCI: I2cUsci> I2cPeriph<USCI> {
    #[inline(always)]
    fn set_addressing_mode(&mut self, mode: AddressingMode) {
        self.usci.set_ucsla10(mode.into())
    }

    #[inline(always)]
    fn set_transmission_mode(&mut self, mode: TransmissionMode) {
        self.usci.set_uctr(mode.into())
    }

    /// Blocking read
    // TODO: Check for arbitration loss
    fn read(&mut self, address: u16, buffer: &mut [u8], send_start: bool, send_stop: bool) -> Result<(), I2CErr> {
        // Hardware doesn't support zero byte reads.
        if buffer.is_empty() { return Ok(()) }

        let usci = &mut self.usci;

        // Clear any flags from previous transactions
        usci.ifg_rst();
        
        usci.i2csa_wr(address);

        if send_start {
            usci.transmit_start();
            // Wait for initial address byte and (N)ACK to complete.
            while usci.uctxstt_rd() {
                asm::nop();
            }
        }

        let len = buffer.len();
        for (idx, byte) in buffer.iter_mut().enumerate() {
            if send_stop && (idx == len - 1) {
                usci.transmit_stop();
            }
            loop {
                let ifg = usci.ifg_rd();
                // If NACK (from initial address packet), send STOP and abort
                if ifg.ucnackifg() {
                    usci.transmit_stop();
                    while usci.uctxstp_rd() {
                        asm::nop();
                    }
                    return Err::<(), I2CErr>(I2CErr::GotNACK);
                }
                // If byte recieved
                if ifg.ucrxifg0() {
                    break;
                }
            }
            *byte = usci.ucrxbuf_rd();
        }

        if send_stop {
            while usci.uctxstp_rd() {
                asm::nop();
            }
        }

        Ok(())
    }

    /// Blocking write
    // TODO: Check for arbitration loss
    fn write(&mut self, address: u16, bytes: &[u8], mut send_start: bool, mut send_stop: bool) -> Result<(), I2CErr> {
        // The only way to perform a zero byte write is with a start + stop
        if bytes.is_empty() {
            send_start = true;
            send_stop = true;
        }

        let usci = &mut self.usci;

        // Clear any flags from previous transactions
        usci.ifg_rst();

        usci.i2csa_wr(address);

        if send_start {
            usci.transmit_start();
        }

        while !usci.ifg_rd().uctxifg0() {
            asm::nop();
        }

        for &byte in bytes {
            usci.uctxbuf_wr(byte);
            loop {
                if usci.ifg_rd().ucnackifg() {
                    usci.transmit_stop();
                    while usci.uctxstp_rd() {
                        asm::nop();
                    }
                    return Err(I2CErr::GotNACK);
                }
                if usci.ifg_rd().uctxifg0() {
                    break;
                }
            }
        } 

        if send_stop {
            usci.transmit_stop();
            while usci.uctxstp_rd() {
                // This is mainly for catching NACKs in a zero-byte write
                if usci.ifg_rd().ucnackifg() {
                    return Err(I2CErr::GotNACK);
                }
            }
        }

        Ok(())
    }

    /// Checks whether a slave with the specified address is present on the I2C bus.
    /// Sends a zero-byte write and records whether the slave sends an ACK or not.
    /// 
    /// A u8 address will use the 7-bit addressing mode, a u16 address uses 10-bit addressing.
    // If we add more I2C error variants this fn should be changed to return a Result<bool, I2cErr>
    pub fn is_slave_present<TenOrSevenBit>(&mut self, address: TenOrSevenBit) -> bool 
    where TenOrSevenBit: AddressType {
        self.set_addressing_mode(TenOrSevenBit::addr_type());
        self.set_transmission_mode(TransmissionMode::Transmit);
        match self.write(address.into(), &[], true, true) {
            Ok(_) => true,
            Err(I2CErr::GotNACK) => false,
            //Err(e) => Err(e),
        }
    }

    /// blocking write then blocking read
    fn write_read(&mut self, address: u16, bytes: &[u8], buffer: &mut [u8]) -> Result<(), I2CErr> {
        self.set_transmission_mode(TransmissionMode::Transmit);
        self.write(address, bytes, true, false)?;
        self.set_transmission_mode(TransmissionMode::Receive);
        self.read(address, buffer, true, true)
    }
}

// Trait to link embedded-hal types to our addressing mode enum.
// Since SevenBitAddress and TenBitAddress are just aliases for u8 and u16 in both ehal 1.0 and 0.2.7, this works for both!
/// A trait marking types that can be used as I2C addresses. Namely `u8` for 7-bit addresses and `u16` for 10-bit addresses.
/// 
/// Used internally by the HAL.
pub trait AddressType: AddressMode + Into<u16> + Copy {
    /// Return the `AddressingMode` that relates to this type: `SevenBit` for `u8`, `TenBit` for `u16`.
    fn addr_type() -> AddressingMode;
}
impl AddressType for SevenBitAddress {
    fn addr_type() -> AddressingMode {
        AddressingMode::SevenBit
    }
}
impl AddressType for TenBitAddress {
    fn addr_type() -> AddressingMode {
        AddressingMode::TenBit
    }
}

mod ehal1 {
    use embedded_hal::i2c::{Error, ErrorKind, ErrorType, I2c, Operation, NoAcknowledgeSource};
    use super::*;

    impl Error for I2CErr {
        fn kind(&self) -> ErrorKind {
            match self {
                I2CErr::GotNACK => ErrorKind::NoAcknowledge(NoAcknowledgeSource::Unknown),
            }
        }
    }

    impl<USCI: I2cUsci> ErrorType for I2cPeriph<USCI> {
        type Error = I2CErr;
    }

    impl<USCI: I2cUsci, TenOrSevenBit> I2c<TenOrSevenBit> for I2cPeriph<USCI> 
    where TenOrSevenBit: AddressType {
        fn transaction(&mut self, address: TenOrSevenBit, ops: &mut [Operation<'_>]) -> Result<(), Self::Error> {
            self.set_addressing_mode(TenOrSevenBit::addr_type());
            
            // Ideally this would be done with sliding windows rather than manual indexing, 
            // but we need a &mut for reads, and `windows_mut` doesn't exist
            for i in 0..ops.len() {
                // Send a start if this is the first operation, 
                // or if the previous operation was a different type (e.g. Read and Write)
                let send_start = if i == 0 { true } else {
                    let op1 = &ops[i];
                    let op2 = &ops[i-1];
                    core::mem::discriminant(op1) != core::mem::discriminant(op2)
                };
                // Send a stop only if this is the last operation
                let send_stop = i == (ops.len() - 1);
                
                match ops[i] {
                    Operation::Read(ref mut items) => {
                        self.set_transmission_mode(TransmissionMode::Receive);
                        self.read(address.into(), items, send_start, send_stop)?;
                    }
                    Operation::Write(items) => {
                        self.set_transmission_mode(TransmissionMode::Transmit);
                        self.write(address.into(), items, send_start, send_stop)?;
                    }
                }
            }
            Ok(())
        }
    }
}

#[cfg(feature = "embedded-hal-02")]
mod ehal02 {
    use embedded_hal_02::blocking::i2c::{AddressMode as ehal02AddressMode, Read, Write, WriteRead};
    use super::*;

    impl<USCI: I2cUsci, SevenOrTenBit> Read<SevenOrTenBit> for I2cPeriph<USCI>
    where SevenOrTenBit: ehal02AddressMode + AddressType {
        type Error = I2CErr;
        fn read(&mut self, address: SevenOrTenBit, buffer: &mut [u8]) -> Result<(), Self::Error> {
            self.set_addressing_mode(SevenOrTenBit::addr_type());
            self.set_transmission_mode(TransmissionMode::Receive);
            self.read(address.into(), buffer, true, true)
        }
    }

    impl<USCI: I2cUsci, SevenOrTenBit> Write<SevenOrTenBit> for I2cPeriph<USCI> 
    where SevenOrTenBit: ehal02AddressMode + AddressType {
        type Error = I2CErr;
        fn write(&mut self, address: SevenOrTenBit, bytes: &[u8]) -> Result<(), Self::Error> {
            self.set_addressing_mode(SevenOrTenBit::addr_type());
            self.set_transmission_mode(TransmissionMode::Transmit);
            self.write(address.into(), bytes, true, true)
        }
    }

    impl<USCI: I2cUsci, SevenOrTenBit> WriteRead<SevenOrTenBit> for I2cPeriph<USCI>  
    where SevenOrTenBit: ehal02AddressMode + AddressType {
        type Error = I2CErr;
        fn write_read(
            &mut self,
            address: SevenOrTenBit,
            bytes: &[u8],
            buffer: &mut [u8],
        ) -> Result<(), Self::Error> {
            self.set_addressing_mode(SevenOrTenBit::addr_type());
            self.write_read(address.into(), bytes, buffer)
        }
    }
}