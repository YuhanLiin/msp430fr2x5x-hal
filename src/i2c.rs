//! I2C
//!
//! Peripherals eUSCI_B0 and eUSCI_B1 can be used for I2C communication.
//!
//! Begin by calling [`I2cConfig::new()`]. Depending on configuration, one of [`I2cSlave`], [`I2cSingleMaster`], [`I2cMultiMaster`], 
//! or [`I2cMasterSlave`] will be returned.
//! 
//! [`I2cSlave`] acts as a slave device on the bus. If the MSP430 is to be the only master on the bus then [`I2cSingleMaster`] 
//! offers simplified error handling. If more than one master is on the bus then [`I2cMultiMaster`] should be used instead.
//! [`I2cMasterSlave`] offers a multi-role implementation that can act as a master but automatically downgrades to a slave 
//! upon being addressed by another device.
//!
//! In all modes interrupts can be set and cleared using the `set_interrupts()` and `clear_interrupts()` methods alongside
//! [`I2cInterruptFlags`], which provides a user-friendly way to set the register flags.
//! 
//! ## [`I2cSlave`]
//! In slave mode the peripheral responds to requests from master devices. The 'own address' is treated as 7-bit should a `u8`
//! be provided, and 10-bit if a `u16` is provided.
//! Both polling and interrupt-based methods are available, though interrupt-based is recommended for slave devices, as the slave
//! can 'fall behind' and lose information if polling is not done frequently enough.
//!
//! The interrupt-based interface relies on using [`interrupt_source()`](I2cRoleCommon::interrupt_source()) to determine which event
//! caused the interrupt. The polling-based implementation instead uses calls to [`poll()`](I2cRoleSlave::poll()) to listen for events.
//! In either case methods such as [`write_tx_buf()`](I2cSlave::write_tx_buf()) and
//! [`read_rx_buf()`](I2cSlave::read_rx_buf()) can be used to respond accordingly.
//!
//! ## [`I2cSingleMaster`]
//! Single master mode provides simplified error handling and ergonomics at the cost of being unsuitable for buses with more than one
//! master - single master mode does not handle bus arbitration, so even if the device is not expected to be addressed as a slave it is not
//! suitable for use on a multi-master bus.
//!
//! An easy-to-use blocking implementation is available through [`embedded_hal::i2c::I2c`], which provides methods for read, write,
//! write-read, and generic transactions. Additionally, slave detection is provided through [`is_slave_present()`](I2cRoleMaster::is_slave_present()).
//!
//! A non-blocking or interrupt-based implementation is possible using [`I2cSingleMaster::send_start()`],
//! [`write_tx_buf()`](I2cSingleMaster::write_tx_buf), [`read_rx_buf()`](I2cSingleMaster::read_rx_buf), and
//! [`schedule_stop()`](I2cRoleMaster::schedule_stop).
//! 
//! ## [`I2cMultiMaster`]
//! [`I2cMultiMaster`] acts similarly to [`I2cSingleMaster`], but with the addition of bus arbitration logic.
//! The MSP430 hardware automatically fails over from master to slave mode when arbitration is lost, so the methods check for this
//! before performing operations. After losing arbitration [`return_to_master()`](I2cRoleMulti::return_to_master) must be called.
//! 
//! ## [`I2cMasterSlave`]
//! [`I2cMasterSlave`] can act as either a master or slave device. It is multi-master capable by necessity.
//! It broadly combines the functionality of [`I2cSlave`] and [`I2cMultiMaster`], providing a blocking master implementation via
//! [`embedded_hal::i2c::I2c`], and a non-blocking or interrupt-based interface via methods similar to [`I2cMultiMaster`]:
//! [`I2cMasterSlave::send_start()`], [`write_tx_buf_as_master()`](I2cMasterSlave::write_tx_buf_as_master),
//! [`read_rx_buf_as_master()`](I2cMasterSlave::read_rx_buf_as_master), and [`schedule_stop()`](I2cRoleMaster::schedule_stop).
//!
//! The MSP430 hardware automatically fails over from master to slave mode when arbitration is lost or the device is addressed as a slave,
//! so the master-related methods check for this before attempting master-related operations, returning an error if so.
//! The device can be restored to master mode via [`return_to_master()`](I2cRoleMulti::return_to_master). If arbitration is lost this
//! method may be called immediately, however if the device is addressed as a slave then this slave transaction must be resolved
//! before the device can be returned to master mode.
//!
//! The slave interface is much the same as what is provided by [`I2cSlave`]: Bus events can be discovered using
//! [`interrupt_source()`](I2cRoleCommon::interrupt_source()) for an interrupt-based implementation, or [`poll()`](I2cRoleSlave::poll())
//! for a polling-based one. [`write_tx_buf_as_slave()`](I2cMasterSlave::write_tx_buf_as_slave) and
//! [`read_rx_buf_as_slave()`](I2cMasterSlave::read_rx_buf_as_slave) allow for writing to the Rx and Tx buffers. These methods don't have the
//! bus arbitration and slave addressing checks that the `_as_master` variants do, so these should only be called in slave mode.
//! 
//! Pins used:
//!
//! eUSCI_B0: {SCL: `P1.3`, SDA: `P1.2`}. `P1.1` can optionally be used as an external clock source in master modes.
//!
//! eUSCI_B1: {SCL: `P4.7`, SDA: `P4.6`}. `P4.5` can optionally be used as an external clock source in master modes.
//!

use core::convert::Infallible;

use crate::clock::{Aclk, Smclk};
use crate::hw_traits::eusci::{EUsciI2C, I2CUcbIfgOut, UcbCtlw0, UcbCtlw1, UcbI2coa, Ucmode, Ucssel};

use core::marker::PhantomData;
use embedded_hal::i2c::{AddressMode, SevenBitAddress, TenBitAddress};
use msp430::asm;
use nb::Error::{Other, WouldBlock};
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
    #[inline(always)]
    fn from(f: AddressingMode) -> bool {
        match f {
            AddressingMode::SevenBit => false,
            AddressingMode::TenBit => true,
        }
    }
}

/// I2C transmission modes
#[derive(Debug, Clone, Copy)]
pub enum TransmissionMode {
    /// Receiver mode
    Receive = 0,
    /// Transmitter mode
    Transmit = 1,
}
impl From<TransmissionMode> for bool {
    #[inline(always)]
    fn from(f: TransmissionMode) -> bool {
        match f {
            TransmissionMode::Receive => false,
            TransmissionMode::Transmit => true,
        }
    }
}

pub use crate::hw_traits::eusci::Ucglit as GlitchFilter;

///Struct used to configure a I2C bus
pub struct I2cConfig<USCI: I2cUsci, CLKSRC, ROLE> {
    usci: USCI,
    divisor: u16,

    // Register configs
    ctlw0: UcbCtlw0,
    ctlw1: UcbCtlw1,
    i2coa0: UcbI2coa,
    i2coa1: UcbI2coa,
    i2coa2: UcbI2coa,
    i2coa3: UcbI2coa,
    clk_src: PhantomData<CLKSRC>,
    role: PhantomData<ROLE>,
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
pub(crate) use impl_i2c_pin;

/// Typestate for an I2C bus configuration with no clock source selected
pub struct NoClockSet;
/// Typestate for an I2C bus configuration with a clock source selected
pub struct ClockSet;

/// Typestate for an I2C bus that has not yet been assigned a role.
pub struct NoRoleSet;

/// Marker trait for typestates that correspond to I2C bus roles.
pub trait I2cMarker {}
/// Typestate for an I2C bus being configured as a master on a bus with no other master devices present.
pub struct SingleMaster;
impl I2cMarker for SingleMaster {}

/// Typestate for an I2C bus being configured as a slave.
pub struct Slave;
impl I2cMarker for Slave {}

/// Typestate for an I2C bus being configured as a master on a bus that has other master devices present.
pub struct MultiMaster;
impl I2cMarker for MultiMaster {}

/// Typestate for an I2C bus being configured as a master on a bus that has other master devices present that may address this device.
pub struct MasterSlave;
impl I2cMarker for MasterSlave {}

macro_rules! return_self_config {
    ($self: ident) => {
        I2cConfig {
            usci:    $self.usci,
            divisor: $self.divisor,
            ctlw0:   $self.ctlw0,
            ctlw1:   $self.ctlw1,
            i2coa0:  $self.i2coa0,
            i2coa1:  $self.i2coa1,
            i2coa2:  $self.i2coa2,
            i2coa3:  $self.i2coa3,
            clk_src: PhantomData,
            role: PhantomData,
        }
    };
}

impl<USCI: I2cUsci> I2cConfig<USCI, NoClockSet, NoRoleSet> {
    /// Begin configuration of an eUSCI peripheral as an I2C device.
    pub fn new(usci: USCI, deglitch_time: GlitchFilter) -> I2cConfig<USCI, NoClockSet, NoRoleSet> {
        let ctlw0 = UcbCtlw0 {
            ucsync: true,
            ucswrst: true,
            ucmode: Ucmode::I2CMode,
            ..Default::default()
        };

        let ctlw1 = UcbCtlw1 {
            ucglit: deglitch_time,
            ..Default::default()
        };

        let i2coa0 = UcbI2coa::default();
        let i2coa1 = UcbI2coa::default();
        let i2coa2 = UcbI2coa::default();
        let i2coa3 = UcbI2coa::default();

        I2cConfig {
            usci,
            divisor: 1,
            ctlw0,
            ctlw1,
            i2coa0,
            i2coa1,
            i2coa2,
            i2coa3,
            clk_src: PhantomData,
            role: PhantomData,
        }
    }
    /// Configure this eUSCI peripheral as an I2C master on a bus with no other master devices.
    pub fn as_single_master(mut self) -> I2cConfig<USCI, NoClockSet, SingleMaster> {
        self.ctlw0.ucmst = true;

        return_self_config!(self)
    }

    /// Configure this eUSCI peripheral as an I2C slave.
    pub fn as_slave<TenOrSevenBit>(mut self, own_address: TenOrSevenBit) -> I2cConfig<USCI, ClockSet, Slave>
    where TenOrSevenBit: AddressType {
        self.ctlw0.uca10 = TenOrSevenBit::addr_type().into();

        self.i2coa0 = UcbI2coa {
            ucgcen: false, // Not yet implemented
            ucoaen: true,
            i2coa0: own_address.into(),
        };

        return_self_config!(self)
    }

    /// Configure this eUSCI peripheral as an I2C master on a bus with other master devices.
    ///
    /// The address comparison unit is disabled so this device can't be addressed as a slave,
    /// though the other masters may still contest the bus.
    pub fn as_multi_master(mut self) -> I2cConfig<USCI, NoClockSet, MultiMaster> {
        self.ctlw0 = UcbCtlw0 {
            ucmst: true,
            ucmm: true,
            ..self.ctlw0
        };

        return_self_config!(self)
    }

    /// Configure this EUSCI peripheral as an I2C master-slave on a bus with other master devices.
    /// The other masters may contest the bus and/or address this device as a slave.
    pub fn as_master_slave<TenOrSevenBit>(mut self, own_address: TenOrSevenBit) -> I2cConfig<USCI, NoClockSet, MasterSlave>
    where TenOrSevenBit: AddressType {
        self.ctlw0 = UcbCtlw0 {
            uca10: TenOrSevenBit::addr_type().into(),
            ucmst: true,
            ucmm: true,
            ..self.ctlw0
        };

        // Note: If you add support for the other 3 own addresses (or the mask) you will also have to upgrade the logic for checking
        // that the peripheral isn't addressing itself, i.e. I2cMasterSlaveErr::TriedAddressingSelf
        self.i2coa0 = UcbI2coa {
            ucgcen: false, // Not yet implemented
            ucoaen: true,
            i2coa0: own_address.into(),
        };

        return_self_config!(self)
    }
}

#[allow(private_bounds)]
impl<USCI: I2cUsci, ROLE: I2cMarker> I2cConfig<USCI, NoClockSet, ROLE> {
    /// Configures this peripheral to use SMCLK
    #[inline]
    pub fn use_smclk(mut self, _smclk: &Smclk, clk_divisor: u16) -> I2cConfig<USCI, ClockSet, ROLE> {
        self.ctlw0.ucssel = Ucssel::Smclk;
        self.divisor = clk_divisor;
        return_self_config!(self)
    }
    /// Configures this peripheral to use ACLK
    #[inline]
    pub fn use_aclk(mut self, _aclk: &Aclk, clk_divisor: u16) -> I2cConfig<USCI, ClockSet, ROLE> {
        self.ctlw0.ucssel = Ucssel::Aclk;
        self.divisor = clk_divisor;
        return_self_config!(self)
    }
    /// Configures this peripheral to use UCLK
    #[inline]
    pub fn use_uclk<Pin: Into<USCI::ExternalClockPin> >(mut self, _uclk: Pin, clk_divisor: u16) -> I2cConfig<USCI, ClockSet, ROLE> {
        self.ctlw0.ucssel = Ucssel::Uclk;
        self.divisor = clk_divisor;
        return_self_config!(self)
    }
}

#[allow(private_bounds)]
impl<USCI: I2cUsci, RoleSet: I2cMarker> I2cConfig<USCI, ClockSet, RoleSet> {
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
        self.usci.ie_wr(0);
        self.usci.ifg_rst();

        self.usci.brw_wr(self.divisor);
        self.usci.tbcnt_wr(0);

        self.usci.ctw0_clear_rst();
    }
}

macro_rules! configure {
    ($role: ty, $out_type: path) => {
        impl<USCI: I2cUsci> I2cConfig<USCI, ClockSet, $role> {
            /// Performs hardware configuration and creates the I2C bus
            #[inline(always)]
            pub fn configure<SCL, SDA>(self, _scl: SCL, _sda: SDA) -> $out_type
            where
                SCL: Into<USCI::ClockPin>,
                SDA: Into<USCI::DataPin>,
            {
                self.configure_regs();
                $out_type { usci: self.usci }
            }
        }
    };
}

configure!(SingleMaster, I2cSingleMaster<USCI>);
configure!(MultiMaster,  I2cMultiMaster<USCI>);
configure!(Slave,        I2cSlave<USCI>);
configure!(MasterSlave,  I2cMasterSlave<USCI>);

mod sealed {
    use super::*;

    pub trait I2cRoleBase {
        type USCI: I2cUsci;
        fn usci(&self) -> &Self::USCI;
    }

    pub trait I2cError {
        fn nack(variant: NackType) -> Self;
        fn nack_type(&self) -> Option<NackType>;
    }

    /// Internal methods common to all I2C roles capable of master operations
    pub trait I2cRoleMasterPrivate: I2cRoleBase {
        type ErrorType: I2cError;
        fn set_addressing_mode(&mut self, mode: AddressingMode) {
            self.usci().set_ucsla10(mode.into())
        }

        fn blocking_read_unchecked(&mut self, address: u16, buffer: &mut [u8], send_start: bool, send_stop: bool) -> Result<(), Self::ErrorType> {
            // Hardware doesn't support zero byte reads.
            if buffer.is_empty() { return Ok(()) }

            // Clear any flags from previous transactions
            self.usci().ifg_rst();
            self.usci().i2csa_wr(address);
            self.usci().set_uctr(TransmissionMode::Receive.into());

            if send_start {
                self.usci().transmit_start();
                // Wait for initial address byte and (N)ACK to complete.
                while self.usci().uctxstt_rd() {
                    asm::nop();
                }
            }

            let len = buffer.len();
            for (idx, byte) in buffer.iter_mut().enumerate() {
                if send_stop && (idx == len - 1) {
                    self.usci().transmit_stop();
                }
                loop {
                    let ifg = self.usci().ifg_rd();
                    self.handle_errs(&ifg, idx)?;
                    if ifg.ucrxifg0() {
                        break;
                    }
                }
                *byte = self.usci().ucrxbuf_rd();
            }

            if send_stop {
                while self.usci().uctxstp_rd() {
                    asm::nop();
                }
            }

            Ok(())
        }

        fn blocking_write_unchecked(&mut self, address: u16, bytes: &[u8], send_start: bool, send_stop: bool) -> Result<(), Self::ErrorType> {
            // Clear any flags from previous transactions
            self.usci().ifg_rst();
            self.usci().i2csa_wr(address);
            self.usci().set_uctr(TransmissionMode::Transmit.into());

            if bytes.is_empty() {
                return self.zero_byte_write();
            }

            if send_start {
                self.usci().transmit_start();
            }

            for (idx, &byte) in bytes.iter().enumerate() {
                loop {
                    let ifg = self.usci().ifg_rd();
                    self.handle_errs(&ifg, idx.saturating_sub(1))?; // Subtract index because buffer fills before any NACKs come through
                    if ifg.uctxifg0() {
                        break;
                    }
                }
                self.usci().uctxbuf_wr(byte);
            }
            while !self.usci().ifg_rd().uctxifg0() {
                self.handle_errs(&self.usci().ifg_rd(), bytes.len().saturating_sub(1))?;
            }

            if send_stop {
                self.usci().transmit_stop();
                while self.usci().is_bus_busy() {
                    self.handle_errs(&self.usci().ifg_rd(), bytes.len())?;
                }
            }
            
            Ok(())
        }

        fn zero_byte_write(&mut self) -> Result<(), Self::ErrorType> {
            self.usci().transmit_start();
            self.usci().transmit_stop();
            self.usci().uctxbuf_wr(0); // Bus stalls if nothing in Tx, even if a stop is scheduled
            while self.usci().uctxstt_rd() || self.usci().uctxstp_rd() {
                self.handle_errs(&self.usci().ifg_rd(), 0)?;
            }
            self.handle_errs(&self.usci().ifg_rd(), 0)?;
            Ok(())
        }

        #[inline]
        fn send_start_unchecked<SevenOrTenBit: AddressType>(&mut self, address: SevenOrTenBit, mode: TransmissionMode) {
            self.set_addressing_mode(SevenOrTenBit::addr_type());
            self.usci().set_uctr(mode.into());
            self.usci().i2csa_wr(address.into());
            self.usci().transmit_start();
        }

        /// In multi-operation transactions update the NACK byte error count to match *total* bytes sent
        #[inline]
        fn add_nack_count(err: Self::ErrorType, bytes_already_sent: usize) -> Self::ErrorType {
            match err.nack_type() {
                None => err,
                Some(NackType::Address(n)) => Self::ErrorType::nack(NackType::Address(n + bytes_already_sent)),
                Some(NackType::Data(n))    => Self::ErrorType::nack(NackType::Data(n + bytes_already_sent)),
            }
        }

        #[inline]
        fn blocking_write(&mut self, address: u16, bytes: &[u8], send_start: bool, send_stop: bool) -> Result<(), Self::ErrorType> {
            self.can_proceed(address)?;
            let res = self.blocking_write_unchecked(address, bytes, send_start, send_stop);
            self.usci().ifg_rst();
            res
        }

        #[inline]
        fn blocking_read(&mut self, address: u16, buffer: &mut [u8], send_start: bool, send_stop: bool) -> Result<(), Self::ErrorType> {
            self.can_proceed(address)?;
            let res = self.blocking_read_unchecked(address, buffer, send_start, send_stop);
            self.usci().ifg_rst();
            res
        }

        /// blocking write then blocking read
        #[inline]
        fn blocking_write_read(&mut self, address: u16, bytes: &[u8], buffer: &mut [u8]) -> Result<(), Self::ErrorType> {
            self.blocking_write(address, bytes, true, false)?;
            self.blocking_read(address, buffer, true, true)
                .map_err(|e| Self::add_nack_count(e, bytes.len()))
        }

        fn mst_write_tx_buf(&mut self, byte: u8, ifg: &<Self::USCI as EUsciI2C>::IfgOut) -> nb::Result<(), Self::ErrorType> {
            if ifg.ucnackifg() {
                let nack_type = match self.usci().byte_count() {
                    0 => NackType::Address(0),
                    n => NackType::Data(n as usize),
                };

                return Err(Other(Self::ErrorType::nack(nack_type)));
            }
            if !ifg.uctxifg0() {
                return Err(WouldBlock);
            }
            self.usci().uctxbuf_wr(byte);
            Ok(())
        }
        
        fn mst_read_rx_buf(&mut self, ifg: &<Self::USCI as EUsciI2C>::IfgOut) -> nb::Result<u8, Self::ErrorType> {
            if ifg.ucnackifg() {
                let nack_type = match self.usci().byte_count() {
                    0 => NackType::Address(0),
                    n => NackType::Data(n as usize),
                };

                return Err(Other(Self::ErrorType::nack(nack_type)));
            }
            if !ifg.ucrxifg0() {
                return Err(WouldBlock);
            }
            Ok(self.usci().ucrxbuf_rd())
        }

        /// Error handling during blocking read/writes. NACKs, bus arbitration, demotion to slave device, etc.
        fn handle_errs(&mut self, ifg: &<Self::USCI as EUsciI2C>::IfgOut, idx: usize) -> Result<(), Self::ErrorType>;

        /// Whether a master operation can occur at the moment
        fn can_proceed(&mut self, _address: u16) -> Result<(), Self::ErrorType>;
    }

    /// Internal methods common to all I2C roles capable of slave operations
    pub trait I2cRoleSlavePrivate: I2cRoleBase {
        #[inline]
        fn sl_write_tx_buf(&mut self, byte: u8) -> nb::Result<(), Infallible> {
            if !self.usci().ifg_rd().uctxifg0() {
                return Err(WouldBlock);
            }
            self.usci().uctxbuf_wr(byte);
            Ok(())
        }
        #[inline]
        fn sl_read_rx_buf(&mut self) -> nb::Result<u8, Infallible> {
            if !self.usci().ifg_rd().ucrxifg0() {
                return Err(WouldBlock);
            }
            Ok(self.usci().ucrxbuf_rd())
        }
    }
}
use sealed::*;

/// Common methods available to all I2C roles.
pub trait I2cRoleCommon: I2cRoleBase {
    /// Queue a NACK to be sent on the I2C bus. If this is called in response to a packet being received the NACK will be sent on the following byte.
    /// 
    /// Used as part of the non-blocking / interrupt-based interface. Only use during a receive operation. 
    #[inline(always)]
    fn send_nack(&mut self) {
        self.usci().transmit_nack();
    }

    /// Get the number of bytes received/transmitted since the last Start or Repeated Start condition.
    #[inline(always)]
    fn byte_count(&mut self) -> u8 {
        self.usci().byte_count()
    }

    /// Get the event that triggered the current interrupt. Used as part of the interrupt-based interface.
    fn interrupt_source(&mut self) -> I2cVector {
        use I2cVector::*;
        match self.usci().iv_rd() {
            0x00 => None,
            0x02 => ArbitrationLost,
            0x04 => NackReceived,
            0x06 => StartReceived,
            0x08 => StopReceived,
            0x0A => Slave3RxBufFull,
            0x0C => Slave3TxBufEmpty,
            0x0E => Slave2RxBufFull,
            0x10 => Slave2TxBufEmpty,
            0x12 => Slave1RxBufFull,
            0x14 => Slave1TxBufEmpty,
            0x16 => RxBufFull,
            0x18 => TxBufEmpty,
            0x1A => ByteCounterZero,
            0x1C => ClockLowTimeout,
            0x1E => NinthBitReceived,
            _ => unsafe { core::hint::unreachable_unchecked() },
        }
    }

    /// Set the bits in the interrupt enable register that correspond to the bits set in `intrs`.
    #[inline(always)]
    fn set_interrupts(&mut self, intrs: I2cInterruptFlags) {
        self.usci().ie_set(intrs.bits())
    }
    /// Clear the bits in the interrupt enable register that correspond to the bits *set* in `intrs`.
    #[inline(always)]
    fn clear_interrupts(&mut self, intrs: I2cInterruptFlags) {
        self.usci().ie_clr(!(intrs.bits()))
    }
}

/// Common methods available to all I2C roles that can perform master operations.
pub trait I2cRoleMaster: I2cRoleMasterPrivate {
    /// Manually schedule a stop condition to be sent. Used as part of the non-blocking interface.
    ///
    /// The stop will be sent after the current byte operation. If the bus stalls waiting for the Rx or Tx buffer then the stop won't be sent until that condition is dealt with.
    #[inline(always)]
    fn schedule_stop(&mut self) {
        self.usci().transmit_stop();
        self.usci().ifg_rst(); // For some reason the TXIFG flag needs to be cleared between transactions
    }

    /// Checks whether a slave with the specified address is present on the I2C bus.
    /// Sends a zero-byte write and records whether the slave sends an ACK or not.
    ///
    /// A `u8` address will use the 7-bit addressing mode, a `u16` address uses 10-bit addressing.
    #[inline]
    fn is_slave_present<TenOrSevenBit>(&mut self, address: TenOrSevenBit) -> Result<bool, Self::ErrorType>
    where TenOrSevenBit: AddressType {
        self.set_addressing_mode(TenOrSevenBit::addr_type());
        match self.blocking_write(address.into(), &[], true, true) {
            Ok(_) => Ok(true),
            Err(e) if e.nack_type().is_some() => Ok(false),
            Err(e) => Err(e),
        }
    }
}

/// Common methods available to all I2C roles that can perform slave operations.
pub trait I2cRoleSlave: I2cRoleSlavePrivate {
    /// Returns whether the device is currently in receive mode or transmit mode.
    #[inline(always)]
    fn transmission_mode(&mut self) -> TransmissionMode {
        match self.usci().is_transmitter() {
            true  => TransmissionMode::Transmit,
            false => TransmissionMode::Receive,
        }
    }
    /// Check the I2C bus flags for any events that should be dealt with. Returns `Err(WouldBlock)` if no events have occurred yet, otherwise `Ok(I2cEvent)`.
    fn poll(&mut self) -> nb::Result<I2cEvent, Infallible> {
        if self.usci().stop_received() {
            self.usci().clear_start_stop_flags();
            return Ok(I2cEvent::Stop);
        }

        match (self.usci().start_received(), self.usci().rxifg0_rd(), self.usci().is_transmitter() & self.usci().txifg0_rd()) {
            (true,  true,  false) => {
                self.usci().clear_start_flag();
                Ok(I2cEvent::WriteStart)
            },
            (true,  false, true ) => {
                self.usci().clear_start_flag();
                Ok(I2cEvent::ReadStart)
            },
            (false, true,  false) => Ok(I2cEvent::Write),
            (false, false, true ) => Ok(I2cEvent::Read),
            // Rx buffer filled, then repeated start then Tx buffer empty. (Can't be reverse because empty Tx buf stalls the bus).
            (true,  true,  true ) => Ok(I2cEvent::OverrunWrite), // Don't clear the start flag yet.
            // Start flag but no Rx / Tx events yet. Don't clear the flag yet.
            (_,     false, false) => Err(WouldBlock),
            // I don't believe this is ever reachable.
            (false, true,  true ) => unreachable!(), // TODO: Test and replace with unchecked
        }
    }

    /// Check whether the device is currently being addressed as a slave.
    #[inline(always)]
    fn is_being_addressed(&mut self) -> bool {
        !self.usci().is_master() && self.usci().ifg_rd().ucsttifg()
    }
}

/// Common methods available to all multi-master-aware I2C roles.
pub trait I2cRoleMulti: I2cRoleMaster {
    /// Manually send a start condition and address byte. Used as part of the non-blocking interface.
    /// Passing a `u8` address uses 7-bit addressing, a `u16` address uses 10-bit addressing.
    #[inline]
    fn send_start<SevenOrTenBit: AddressType>(&mut self, address: SevenOrTenBit, mode: TransmissionMode) -> Result<(), Self::ErrorType>{
        self.can_proceed(address.into())?;
        self.send_start_unchecked(address, mode);
        Ok(())
    }

    /// After losing arbitration (or after being addressed as a slave) call this method to return the peripheral to master mode.
    #[inline(always)]
    fn return_to_master(&mut self) {
        self.usci().set_master();
    }

    /// Check whether the device is currently in master mode.
    #[inline(always)]
    fn is_master(&mut self) -> bool {
        self.usci().is_master()
    }
}

/// An eUSCI peripheral that has been configured as an I2C master.
/// This variant offers simplified error handling and ease of use, but is not suitable for use on a multi-master bus.
pub struct I2cSingleMaster<USCI> {
    usci: USCI,
}
impl<USCI: I2cUsci> I2cRoleBase for I2cSingleMaster<USCI> {
    type USCI = USCI;

    fn usci(&self) -> &Self::USCI {
        &self.usci
    }
}
impl<USCI: I2cUsci> I2cRoleCommon for I2cSingleMaster<USCI> {}
impl<USCI: I2cUsci> I2cRoleMasterPrivate for I2cSingleMaster<USCI> {
    type ErrorType = I2cSingleMasterErr;
    fn handle_errs(&mut self, ifg: &<Self::USCI as EUsciI2C>::IfgOut, idx: usize) -> Result<(), Self::ErrorType> {
        if ifg.ucnackifg() {
            self.usci.transmit_stop();
            let nack = if idx == 0 {
                NackType::Address(idx)
            } else {
                NackType::Data(idx)
            };
            while self.usci.uctxstp_rd() {
                asm::nop();
            }
            return Err(I2cSingleMasterErr::GotNACK(nack));
        }
        Ok(())
    }

    fn can_proceed(&mut self, _address: u16) -> Result<(), Self::ErrorType> {
        Ok(())
    }
}
impl<USCI: I2cUsci> I2cRoleMaster for I2cSingleMaster<USCI> {}
impl<USCI: I2cUsci> I2cSingleMaster<USCI> {
    /// Manually send a start condition and address byte. Used as part of the non-blocking interface.
    /// Passing a `u8` address uses 7-bit addressing, a `u16` address uses 10-bit addressing.
    #[inline(always)]
    pub fn send_start<SevenOrTenBit: AddressType>(&mut self, address: SevenOrTenBit, mode: TransmissionMode) {
        self.send_start_unchecked(address, mode);
    }

    /// Check if the Rx buffer is full, if so read it. Used as part of the non-blocking / interrupt-based interface.
    ///
    /// Returns `Err(WouldBlock)` if the Rx buffer is empty, `Err(GotNACK(n))` if a NACK was received from a previous byte
    /// (will prevent the Rx buffer from filling), where `n` is the number of
    /// bytes since the latest Start or Repeated Start condition. otherwise `Ok(n)`.
    #[inline(always)]
    pub fn read_rx_buf(&mut self) -> nb::Result<u8, I2cSingleMasterErr> {
        self.mst_read_rx_buf(&self.usci.ifg_rd())
    }

    /// Check if the Tx buffer is empty, if so write to it. Used as part of the non-blocking / interrupt-based interface.
    ///
    /// Returns `Err(WouldBlock)` if the Tx buffer is still full, `Err(GotNACK(n))` if a NACK was received from a previous byte
    /// (will prevent the Tx buffer from emptying), where `n` is the number of
    /// bytes since the latest Start or Repeated Start condition. Otherwise returns `Ok(())`.
    #[inline(always)]
    pub fn write_tx_buf(&mut self, byte: u8) -> nb::Result<(), I2cSingleMasterErr> {
        self.mst_write_tx_buf(byte, &self.usci.ifg_rd())
    }
}

/// An eUSCI peripheral that has been configured as an I2C multi-master.
/// Multi-masters are capable of sharing an I2C bus with other multi-masters, and may also optionally act as a slave device (depending on configuration).
pub struct I2cMultiMaster<USCI> {
    usci: USCI,
}
impl<USCI: I2cUsci> I2cRoleBase for I2cMultiMaster<USCI> {
    type USCI = USCI;

    fn usci(&self) -> &Self::USCI {
        &self.usci
    }
}
impl<USCI: I2cUsci> I2cRoleCommon for I2cMultiMaster<USCI> {}
impl<USCI: I2cUsci> I2cRoleMasterPrivate for I2cMultiMaster<USCI> {
    type ErrorType = I2cMultiMasterErr;
    fn can_proceed(&mut self, _address: u16) -> Result<(), I2cMultiMasterErr> {
        // Multimaster doesn't need to check anything with the address, but it keeps the interface the same so we can abstract it
        if !self.usci.is_master() {
            return Err(I2cMultiMasterErr::ArbitrationLost);
        }
        Ok(())
    }

    fn handle_errs(&mut self, ifg: &USCI::IfgOut, idx: usize) -> Result<(), I2cMultiMasterErr> {
        if ifg.ucnackifg() {
            self.usci.transmit_stop();
            let nack = if idx == 0 {
                NackType::Address(idx)
            } else {
                NackType::Data(idx)
            };
            while self.usci.uctxstp_rd() {
                asm::nop();
            }
            return Err(I2cMultiMasterErr::GotNACK(nack));
        }
        if ifg.ucalifg() {
            return Err(I2cMultiMasterErr::ArbitrationLost);
        }
        Ok(())
    }
}
impl<USCI: I2cUsci> I2cRoleMaster for I2cMultiMaster<USCI> {}
impl<USCI: I2cUsci> I2cRoleMulti for I2cMultiMaster<USCI> {}
impl<USCI: I2cUsci> I2cMultiMaster<USCI> {
    /// Check if the Rx buffer is full, if so read it. Used as part of the non-blocking / interrupt-based interface.
    ///
    /// Returns `Err(WouldBlock)` if the buffer is empty,
    /// `Err(Other(I2cMultiMasterErr))` if any bus conditions occur that would impede regular operation, or
    /// `Ok(n)` if data was successfully retreived from the Rx buffer.
    #[inline]
    pub fn read_rx_buf(&mut self) -> nb::Result<u8, I2cMultiMasterErr> {
        let ifg = self.usci.ifg_rd();
        if ifg.ucalifg() {
            return Err(Other(I2cMultiMasterErr::ArbitrationLost));
        }
        self.mst_read_rx_buf(&ifg)
    }

    /// Check if the Tx buffer is empty, if so write to it. Used as part of the non-blocking / interrupt-based interface.
    /// First checks if the peripheral is still in master mode, if not returns an error.
    ///
    /// Returns `Err(WouldBlock)` if the buffer is still full,
    /// `Err(Other(I2cMultiMasterErr))` if any bus conditions occur that would impede regular operation, or
    /// `Ok(())` if data was successfully loaded into the Tx buffer.
    #[inline]
    pub fn write_tx_buf(&mut self, byte: u8) -> nb::Result<(), I2cMultiMasterErr> {
        let ifg = self.usci.ifg_rd();
        if ifg.ucalifg() {
            return Err(Other(I2cMultiMasterErr::ArbitrationLost));
        }
        self.mst_write_tx_buf(byte, &ifg)
    }
}

/// An eUSCI peripheral that has been configured as an I2C slave.
pub struct I2cSlave<USCI> {
    usci: USCI,
}
impl<USCI: I2cUsci> I2cRoleBase for I2cSlave<USCI> {
    type USCI = USCI;

    fn usci(&self) -> &Self::USCI {
        &self.usci
    }
}
impl<USCI: I2cUsci> I2cRoleCommon for I2cSlave<USCI> {}
impl<USCI: I2cUsci> I2cRoleSlavePrivate for I2cSlave<USCI> {}
impl<USCI: I2cUsci> I2cRoleSlave for I2cSlave<USCI> {}
impl<USCI: I2cUsci> I2cSlave<USCI> {
    /// Read the Rx buffer without checking if it's ready.
    /// Useful in cases where you already know the Rx buffer is ready (e.g. an Rx interrupt occurred).
    /// Used as part of the non-blocking / interrupt-based interface.
    /// # Safety
    /// If the buffer is not ready then the data will be invalid.
    #[inline(always)]
    pub unsafe fn read_rx_buf_unchecked(&mut self) -> u8 {
        self.usci.ucrxbuf_rd()
    }

    /// Write to the Tx buffer without checking if it's ready.
    /// Useful in cases where you already know the Tx buffer is ready (e.g. a Tx interrupt occurred).
    /// Used as part of the non-blocking / interrupt-based interface.
    /// # Safety
    /// If the buffer is not ready then previous data may be clobbered.
    #[inline(always)]
    pub unsafe fn write_tx_buf_unchecked(&mut self, byte: u8) {
        self.usci.uctxbuf_wr(byte);
    }

    /// Check if the Rx buffer is full, if so read it. Used as part of the non-blocking / interrupt-based interface.
    /// Returns `Err(WouldBlock)` if the Rx buffer is empty, otherwise `Ok(n)`.
    #[inline(always)]
    pub fn read_rx_buf(&mut self) -> nb::Result<u8, Infallible> {
        self.sl_read_rx_buf()
    }

    /// Check if the Tx buffer is empty, if so write to it. Used as part of the non-blocking / interrupt-based interface.
    /// Returns `Err(WouldBlock)` if the Tx buffer is still full, otherwise `Ok(())`.
    #[inline(always)]
    pub fn write_tx_buf(&mut self, byte: u8) -> nb::Result<(), Infallible> {
        self.sl_write_tx_buf(byte)
    }
}

/// An eUSCI peripheral that has been configured as an I2C multi-master.
/// Multi-masters are capable of sharing an I2C bus with other multi-masters, and may also optionally act as a slave device (depending on configuration).
pub struct I2cMasterSlave<USCI> {
    usci: USCI,
}
impl<USCI: I2cUsci> I2cRoleBase for I2cMasterSlave<USCI> {
    type USCI = USCI;

    fn usci(&self) -> &Self::USCI {
        &self.usci
    }
}
impl<USCI: I2cUsci> I2cRoleCommon for I2cMasterSlave<USCI> {}
impl<USCI: I2cUsci> I2cRoleMasterPrivate for I2cMasterSlave<USCI> {
    type ErrorType = I2cMasterSlaveErr;
    fn can_proceed(&mut self, address: u16) -> Result<(), I2cMasterSlaveErr> {
        // Are we a master? If not, why?
        if !self.usci.is_master() {
            return match self.usci.ifg_rd().ucsttifg() {
                false => Err(I2cMasterSlaveErr::ArbitrationLost),
                true  => Err(I2cMasterSlaveErr::AddressedAsSlave),
            };
        }
        // Check if the eUSCI is addressing itself. The hardware isn't capable of this.
        let own_addr_reg = self.usci.i2coa_rd(0);
        if own_addr_reg.ucoaen && own_addr_reg.i2coa0 == address {
            return Err(I2cMasterSlaveErr::TriedAddressingSelf);
        }
        Ok(())
    }

    fn handle_errs(&mut self, ifg: &USCI::IfgOut, idx: usize) -> Result<(), I2cMasterSlaveErr> {
        if ifg.ucnackifg() {
            self.usci.transmit_stop();
            let nack = if idx == 0 {
                NackType::Address(idx)
            } else {
                NackType::Data(idx)
            };
            while self.usci.uctxstp_rd() {
                asm::nop();
            }
            return Err(I2cMasterSlaveErr::GotNACK(nack));
        }
        if ifg.ucalifg() {
            return match ifg.ucsttifg() {
                false => Err(I2cMasterSlaveErr::ArbitrationLost),  // Lost arbitration
                true  => Err(I2cMasterSlaveErr::AddressedAsSlave), // Lost arbitration and the slave address was us
            };
        }
        Ok(())
    }
}
impl<USCI: I2cUsci> I2cRoleMaster for I2cMasterSlave<USCI> {}
impl<USCI: I2cUsci> I2cRoleSlavePrivate for I2cMasterSlave<USCI> {}
impl<USCI: I2cUsci> I2cRoleSlave for I2cMasterSlave<USCI> {}
impl<USCI: I2cUsci> I2cRoleMulti for I2cMasterSlave<USCI> {}

impl<USCI: I2cUsci> I2cMasterSlave<USCI> {
    /// Check if the Rx buffer is full, if so read it. Used as part of the non-blocking / interrupt-based interface.
    ///
    /// Returns `Err(WouldBlock)` if the buffer is empty,
    /// `Err(Other(I2cMasterSlaveErr))` if any bus conditions occur that would impede regular operation, or
    /// `Ok(n)` if data was successfully retreived from the Rx buffer.
    #[inline]
    pub fn read_rx_buf_as_master(&mut self) -> nb::Result<u8, I2cMasterSlaveErr> {
        let ifg = self.usci.ifg_rd();
        if ifg.ucalifg() {
            return match ifg.ucsttifg() {
                false => Err(Other(I2cMasterSlaveErr::ArbitrationLost)),
                true  => Err(Other(I2cMasterSlaveErr::AddressedAsSlave)),
            };
        }
        self.mst_read_rx_buf(&ifg)
    }

    /// Check if the Rx buffer is full, if so read it. Used as part of the non-blocking / interrupt-based interface.
    ///
    /// Returns `Err(WouldBlock)` if the buffer is empty, or
    /// `Ok(n)` if data was successfully retreived from the Rx buffer.
    #[inline(always)]
    pub fn read_rx_buf_as_slave(&mut self) -> nb::Result<u8, Infallible> {
        self.sl_read_rx_buf()
    }

    /// Read the Rx buffer without checking if it's ready. Should only be used if the peripheral is in slave mode.
    ///
    /// Useful in cases where you already know the Rx buffer is ready (e.g. an Rx interrupt occurred).
    /// Used as part of the non-blocking / interrupt-based interface.
    /// # Safety
    /// If the buffer is not ready then the data will be invalid.
    #[inline(always)]
    pub unsafe fn read_rx_buf_as_slave_unchecked(&mut self) -> u8 {
        self.usci.ucrxbuf_rd()
    }

    /// Check if the Tx buffer is empty, if so write to it. Used as part of the non-blocking / interrupt-based interface.
    /// First checks if the peripheral is still in master mode, if not returns an error.
    ///
    /// Returns `Err(WouldBlock)` if the buffer is still full,
    /// `Err(Other(I2cMasterSlaveErr))` if any bus conditions occur that would impede regular operation, or
    /// `Ok(())` if data was successfully loaded into the Tx buffer.
    #[inline]
    pub fn write_tx_buf_as_master(&mut self, byte: u8) -> nb::Result<(), I2cMasterSlaveErr> {
        let ifg = self.usci.ifg_rd();
        if ifg.ucalifg() {
            return match ifg.ucsttifg() {
                false => Err(Other(I2cMasterSlaveErr::ArbitrationLost)),  // Lost arbitration
                true  => Err(Other(I2cMasterSlaveErr::AddressedAsSlave)), // Lost arbitration and the slave address was us
            };
        }
        self.mst_write_tx_buf(byte, &ifg)
    }

    /// Check if the Tx buffer is empty, if so write to it. Used as part of the non-blocking / interrupt-based interface.
    /// Does not check if the peripheral is in master mode.
    ///
    /// Returns `Err(WouldBlock)` if the buffer is still full, or
    /// `Ok(())` if data was successfully loaded into the Tx buffer.
    #[inline(always)]
    pub fn write_tx_buf_as_slave(&mut self, byte: u8) -> nb::Result<(), Infallible> {
        self.sl_write_tx_buf(byte)
    }

    /// Write to the Tx buffer without checking if it's ready. Should only be used if the peripheral is in slave mode.
    /// Useful in cases where you already know the Tx buffer is ready (e.g. a Tx interrupt occurred).
    ///
    /// Used as part of the non-blocking / interrupt-based interface.
    /// # Safety
    /// If the buffer is not ready then previous data may be clobbered.
    #[inline(always)]
    pub unsafe fn write_tx_buf_as_slave_unchecked(&mut self, byte: u8) {
        self.usci.uctxbuf_wr(byte);
    }
}

macro_rules! impl_i2c_error {
    ($err_type: ty) => {
        impl I2cError for $err_type {
            fn nack(variant: NackType) -> Self {
                Self::GotNACK(variant)
            }

            fn nack_type(&self) -> Option<NackType> {
                match self {
                    Self::GotNACK(nack_type) => Some(*nack_type),
                    #[allow(unreachable_patterns)] // I2cSingleMasterErr has only one variant
                    _ => None,
                }
            }
        }
    };
}

/// NACK information enum. The contained value is the byte number when the error occurred. 
///
/// If this originated from a blocking method the byte number counts up from the beginning of the transaction 
/// (i.e. the initial start condition) where byte 0 is the address byte, byte 1 is the first data byte, etc.. 
/// If it originated from a non-blocking method it counts up from the most recent Start or Repeated Start condition.
#[derive(Clone, Copy, Debug)]
pub enum NackType {
    /// Received a NACK during an address byte. No device with the specified address is on the bus.
    Address(usize),
    /// Received a NACK during a data byte. This could be caused by a number of reasons - 
    /// the receiver is not ready, it received invalid data or commands, it cannot receive any more data, etc.
    Data(usize),
}

/// I2C transmit/receive errors on a single master I2C bus.
#[derive(Clone, Copy, Debug)]
#[non_exhaustive]
pub enum I2cSingleMasterErr {
    /// Received a NACK. The contained value denotes the byte where the NACK occurred.
    GotNACK(NackType),
    // Other errors like the 'clock low timeout' UCCLTOIFG may appear here in future.
}
impl_i2c_error!(I2cSingleMasterErr);

/// I2C transmit/receive errors on a multi-master I2C bus.
#[derive(Clone, Copy, Debug)]
#[non_exhaustive]
pub enum I2cMultiMasterErr {
    /// Received a NACK. The contained value denotes the byte where the NACK occurred.
    GotNACK(NackType),
    /// Another master on the bus talked over us, so the transaction was aborted.
    /// The peripheral has been forced into slave mode.
    /// Call [`return_to_master()`](I2cRoleMulti::return_to_master) to resume the master role.
    ArbitrationLost,
    // Other errors like the 'clock low timeout' UCCLTOIFG may appear here in future.
}
impl_i2c_error!(I2cMultiMasterErr);

/// I2C transmit/receive errors on a master-slave I2C device.
#[derive(Clone, Copy, Debug)]
#[non_exhaustive]
pub enum I2cMasterSlaveErr {
    /// Received a NACK. The contained value denotes the byte where the NACK occurred.
    GotNACK(NackType),
    /// Another master on the bus talked over us, so the transaction was aborted.
    /// The peripheral has been forced into slave mode.
    /// Call [`return_to_master()`](I2cRoleMulti::return_to_master) to resume the master role.
    ArbitrationLost,
    /// Another master on the bus addressed us as a slave device. The peripheral has been forced into slave mode.
    /// The slave transaction *must* be completed before master operations can be resumed with
    /// [`return_to_master()`](I2cRoleMulti::return_to_master).
    AddressedAsSlave,
    /// The eUSCI peripheral attempted to address itself. The hardware does not support this operation.
    TriedAddressingSelf,
    // Other errors like the 'clock low timeout' UCCLTOIFG may appear here in future.
}
impl_i2c_error!(I2cMasterSlaveErr);

/// A list of events that may occur on the I2C bus.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum I2cEvent {
    /// The master sent a (repeated) start and wants to read from us. Write to the Tx buffer to clear this event.
    ReadStart,
    /// The master continues to read from us. Write to the Tx buffer to clear this event.
    Read,
    /// The master sent a (repeated) start and wants to write to us. Read from the Rx buffer to clear this event.
    WriteStart,
    /// The master continues to write to us. Read from the Rx buffer to clear this event.
    Write,
    /// We have fallen behind. The master sent a write (filled the Rx buffer), then a repeated start and a read (currently stalled).
    ///
    /// The repeated start after the write means we can no longer tell if the initial write was a `WriteStart` or a `Write`.
    ///
    /// If you have more information about the expected format of the transaction you may be able to deduce which of the two it was.
    ///
    /// Read from the Rx buffer to clear this event.
    OverrunWrite,
    /// The master has ended the transaction. This event is automatically cleared.
    Stop,
}

/// List of possible I2C interrupt sources. 
/// 
/// Used when reading from the I2C interrupt vector register via [`interrupt_source()`](I2cRoleCommon::interrupt_source())
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum I2cVector {
    /// No interrupt.
    None                = 0x00,
    /// Arbitration was lost during an attempted transmission.
    ArbitrationLost     = 0x02,
    /// Received a NACK.
    NackReceived        = 0x04,
    /// Received a Start condition on the I2C bus along with one of our own addresses.
    StartReceived       = 0x06,
    /// Received a Stop condition on the I2C bus.
    /// This is usually set when acting as an I2C slave, but that this can occur as an I2C master during a zero byte write.
    StopReceived        = 0x08,
    /// Slave address 3 received a data byte.
    Slave3RxBufFull     = 0x0A,
    /// The Tx buffer is empty and slave address 3 was on the I2C bus when this occurred.
    Slave3TxBufEmpty    = 0x0C,
    /// Slave address 2 received a data byte.
    Slave2RxBufFull     = 0x0E,
    /// The Tx buffer is empty and slave address 2 was on the I2C bus when this occurred.
    Slave2TxBufEmpty    = 0x10,
    /// Slave address 1 received a data byte.
    Slave1RxBufFull     = 0x12,
    /// The Tx buffer is empty and slave address 1 was on the I2C bus when this occurred.
    Slave1TxBufEmpty    = 0x14,
    /// Data is waiting in the Rx buffer. In slave mode slave address 0 was on the I2C bus when this occurred.
    RxBufFull           = 0x16,
    /// The Tx buffer is empty. In slave mode slave address 0 was on the I2C bus when this occurred.
    TxBufEmpty          = 0x18,
    /// The target byte count has been reached.
    ByteCounterZero     = 0x1A,
    /// The SCL line has been held low longer than the Clock Low Timeout value.
    ClockLowTimeout     = 0x1C,
    /// The 9th bit of an I2C data packet has been completed.
    NinthBitReceived    = 0x1E,
}

bitflags::bitflags! {
    /// Human-friendly list of possible I2C interrupt source flags.
    /// 
    /// Used for writing to the I2C interrupt enable register e.g. via the [`set_interrupts()`](I2cSingleMaster::set_interrupts()) method.
    pub struct I2cInterruptFlags: u16 {
        /// UCRXIE0. Trigger an interrupt when data is waiting in the Rx buffer. In slave mode slave address 0 must be on the I2C bus when this occurred.
        const RxBufFull           = 1 << 0;
        /// UCTXIE0. Trigger an interrupt when the Tx buffer is empty. In slave mode slave address 0 must be on the I2C bus when this occurred.
        const TxBufEmpty          = 1 << 1;
        /// UCSTTIE. Trigger an interrupt when a Start condition is received on the I2C bus along with one of our own addresses.
        const StartReceived       = 1 << 2;
        /// UCSTPIE. Trigger an interrupt when a Stop condition is received on the I2C bus in a transaction we are a part of.
        /// Typically this triggers when acting as an I2C slave, but this also triggers as an I2C master during a zero byte write.
        const StopReceived        = 1 << 3;
        /// UCALIE. Trigger an interrupt when arbitration was lost during an attempted transmission.
        const ArbitrationLost     = 1 << 4;
        /// UCNACKIE. Trigger an interrupt a NACK is received.
        const NackReceived        = 1 << 5;
        /// UCBCNTIE. Trigger an interrupt when the target byte count has been reached.
        const ByteCounterZero     = 1 << 6;
        /// UCCLTOIE. Trigger an interrupt when the SCL line has been held low longer than the Clock Low Timeout value.
        const ClockLowTimeout     = 1 << 7;
        /// UCRXIE1. Trigger an interrupt when slave address 1 receives a data byte.
        const Slave1RxBufFull     = 1 << 8;
        /// UCTXIE1. Trigger an interrupt when the Tx buffer is empty and slave address 1 was on the I2C bus when this occurred.
        const Slave1TxBufEmpty    = 1 << 9;
        /// UCRXIE2. Trigger an interrupt when slave address 2 receives a data byte.
        const Slave2RxBufFull     = 1 << 10;
        /// UCTXIE2. Trigger an interrupt when the Tx buffer is empty and slave address 2 was on the I2C bus when this occurred.
        const Slave2TxBufEmpty    = 1 << 11;
        /// UCRXIE3. Trigger an interrupt when slave address 3 receives a data byte.
        const Slave3RxBufFull     = 1 << 12;
        /// UCTXIE3. Trigger an interrupt when the Tx buffer is empty and slave address 3 was on the I2C bus when this occurred.
        const Slave3TxBufEmpty    = 1 << 13;
        /// UCBIT9IE. Trigger an interrupt when the 9th bit of an I2C data packet we are involved in has been completed.
        const NinthBitReceived    = 1 << 14;
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
    use super::*;
    use embedded_hal::i2c::{Error, ErrorKind, ErrorType, I2c, NoAcknowledgeSource, Operation};

    /// Implement embedded-hal's [`I2c`](embedded_hal::i2c::I2c) trait
    macro_rules! impl_ehal_i2c {
        ($type: ty, $err_type: ty) => {
            impl<USCI: I2cUsci, TenOrSevenBit> I2c<TenOrSevenBit> for $type
            where TenOrSevenBit: AddressType {
                fn transaction(&mut self, address: TenOrSevenBit, ops: &mut [Operation<'_>]) -> Result<(), Self::Error> {
                    self.set_addressing_mode(TenOrSevenBit::addr_type());

                    let mut prev_discr = None;
                    let mut bytes_sent = 0;
                    let len = ops.len();
                    for (i, op) in ops.iter_mut().enumerate() {
                        // Send a start if this is the first operation,
                        // or if the previous operation was a different type (e.g. Read and Write)
                        let send_start = match prev_discr {
                            None => true,
                            Some(prev) => prev != core::mem::discriminant(op),
                        };
                        // Send a stop only if this is the last operation
                        let send_stop = i == (len - 1);

                        match op {
                            Operation::Read(ref mut items) => {
                                self.blocking_read(address.into(), items, send_start, send_stop)
                                    .map_err(|e| Self::add_nack_count(e, bytes_sent))?;
                                bytes_sent += items.len();
                            }
                            Operation::Write(items) => {
                                self.blocking_write(address.into(), items, send_start, send_stop)
                                    .map_err(|e| Self::add_nack_count(e, bytes_sent))?;
                                bytes_sent += items.len();
                            }
                        }
                        prev_discr = Some(core::mem::discriminant(op));
                    }
                    Ok(())
                }
            }
            impl<USCI: I2cUsci> ErrorType for $type {
                type Error = $err_type;
            }
        };
    }

    use NackType::*;
    impl_ehal_i2c!(I2cSingleMaster<USCI>, I2cSingleMasterErr);
    impl Error for I2cSingleMasterErr {
        fn kind(&self) -> ErrorKind {
            match self {
                I2cSingleMasterErr::GotNACK(Address(_))  => ErrorKind::NoAcknowledge(NoAcknowledgeSource::Address),
                I2cSingleMasterErr::GotNACK(Data(_))     => ErrorKind::NoAcknowledge(NoAcknowledgeSource::Data),
            }
        }
    }

    impl_ehal_i2c!(I2cMultiMaster<USCI>, I2cMultiMasterErr);
    impl Error for I2cMultiMasterErr {
        fn kind(&self) -> ErrorKind {
            match self {
                I2cMultiMasterErr::GotNACK(Address(_))  => ErrorKind::NoAcknowledge(NoAcknowledgeSource::Address),
                I2cMultiMasterErr::GotNACK(Data(_))     => ErrorKind::NoAcknowledge(NoAcknowledgeSource::Data),
                I2cMultiMasterErr::ArbitrationLost      => ErrorKind::ArbitrationLoss,
            }
        }
    }

    impl_ehal_i2c!(I2cMasterSlave<USCI>, I2cMasterSlaveErr);
    impl Error for I2cMasterSlaveErr {
        fn kind(&self) -> ErrorKind {
            match self {
                I2cMasterSlaveErr::GotNACK(Address(_))  => ErrorKind::NoAcknowledge(NoAcknowledgeSource::Address),
                I2cMasterSlaveErr::GotNACK(Data(_))     => ErrorKind::NoAcknowledge(NoAcknowledgeSource::Data),
                I2cMasterSlaveErr::ArbitrationLost      => ErrorKind::ArbitrationLoss,
                I2cMasterSlaveErr::AddressedAsSlave     => ErrorKind::ArbitrationLoss,
                I2cMasterSlaveErr::TriedAddressingSelf  => ErrorKind::Other,
            }
        }
    }
}

#[cfg(feature = "embedded-hal-02")]
mod ehal02 {
    use super::*;
    use embedded_hal_02::blocking::i2c::{AddressMode, Read, Write, WriteRead};

    macro_rules! impl_ehal02_i2c {
        ($type: ty, $err_type: ty) => {
            impl<USCI: I2cUsci, SevenOrTenBit> Read<SevenOrTenBit> for $type
            where SevenOrTenBit: AddressMode + AddressType {
                type Error = $err_type;
                #[inline]
                fn read(&mut self, address: SevenOrTenBit, buffer: &mut [u8]) -> Result<(), Self::Error> {
                    self.set_addressing_mode(SevenOrTenBit::addr_type());
                    self.blocking_read(address.into(), buffer, true, true)
                }
            }
            impl<USCI: I2cUsci, SevenOrTenBit> Write<SevenOrTenBit> for $type
            where SevenOrTenBit: AddressMode + AddressType {
                type Error = $err_type;
                #[inline]
                fn write(&mut self, address: SevenOrTenBit, bytes: &[u8]) -> Result<(), Self::Error> {
                    self.set_addressing_mode(SevenOrTenBit::addr_type());
                    self.blocking_write(address.into(), bytes, true, true)
                }
            }
            impl<USCI: I2cUsci, SevenOrTenBit> WriteRead<SevenOrTenBit> for $type
            where SevenOrTenBit: AddressMode + AddressType {
                type Error = $err_type;
                #[inline]
                fn write_read(
                    &mut self,
                    address: SevenOrTenBit,
                    bytes: &[u8],
                    buffer: &mut [u8],
                ) -> Result<(), Self::Error> {
                    self.set_addressing_mode(SevenOrTenBit::addr_type());
                    self.blocking_write_read(address.into(), bytes, buffer)
                }
            }
        };
    }

    impl_ehal02_i2c!(I2cSingleMaster<USCI>, I2cSingleMasterErr);
    impl_ehal02_i2c!(I2cMultiMaster<USCI>,  I2cMultiMasterErr);
    impl_ehal02_i2c!(I2cMasterSlave<USCI>,  I2cMasterSlaveErr);
}
