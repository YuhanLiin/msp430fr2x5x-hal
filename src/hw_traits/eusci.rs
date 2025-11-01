use super::Steal;
use crate::pac;

use embedded_hal::spi::{Mode, Phase, Polarity};

/// Defines macros for a register associated struct to make reading/writing to this struct's
/// register a lot less tedious.
///
/// The $macro_rd and $macro_wr inputs are needed due to a limitation in Rust's macro parsing
/// where it isn't able to create new tokens.
macro_rules! reg_struct {
    (
        $(#[$attr:meta])*
        pub struct $struct_name : ident, $macro_rd : ident, $macro_wr : ident {
            $(flags{
                $(pub $bool_name : ident : bool, $(#[$f_attr:meta])*)*
            })?
            $(enums{
                $(pub $val_name : ident : $val_type : ty : $size : ty, $(#[$e_attr:meta])*)*
            })?
            $(ints{
                $(pub $int_name : ident : $int_size : ty, $(#[$i_attr:meta])*)*
            })?
        }
    ) => {
        $(#[$attr:meta])*
        #[derive(Copy, Clone, Default)]
        pub struct $struct_name {
            $($(pub $bool_name : bool, $(#[$f_attr])*)*)?
            $($(pub $val_name : $val_type , $(#[$e_attr])*)*)?
            $($(pub $int_name : $int_size , $(#[$i_attr])*)*)?
        }

        #[allow(unused_macros)]
        macro_rules! $macro_wr {
            ($reg : expr) => { |w|
                w$($(.$bool_name().bit($reg.$bool_name))*)?
                 $($(.$val_name().bits($reg.$val_name as $size))*)?
                 $($(.$int_name().bits($reg.$int_name as $int_size))*)?
            };
        }
    };
}

#[derive(Copy, Clone, Default)]
pub enum Ucssel {
    Uclk = 0,
    Aclk = 1,
    #[default]
    Smclk = 2,
}

#[derive(Copy, Clone, Default)]
pub enum Ucmode {
    #[default]
    ThreePinSPI = 0,
    /// 0b01
    FourPinSPI1 = 1,
    /// 0b10
    FourPinSPI0 = 2,
    I2CMode = 3,
}

/// Configure the automatic glitch filter on the SDA and SCL lines
#[derive(Copy, Clone, Default)]
pub enum Ucglit {
    /// Pulses of maximum 50-ns length are filtered.
    #[default]
    Max50ns = 0,
    /// Pulses of maximum 25-ns length are filtered.
    Max25ns = 1,
    /// Pulses of maximum 12.5-ns length are filtered.
    Max12_5ns = 2,
    /// Pulses of maximum 6.25-ns length are filtered.
    Max6_25ns = 3,
}

/// Clock low timeout select
#[derive(Copy, Clone, Default)]
pub enum Ucclto {
    /// Disable clock low time-out counter
    #[default]
    Ucclto00b = 0,
    /// 135000 MODCLK cycles (approximately 28 ms)
    Ucclto01b = 1,
    /// 150000 MODCLK cycles (approximately 31 ms)
    Ucclto10b = 2,
    /// = 165000 MODCLK cycles (approximately 34 ms)
    Ucclto11b = 3,
}

/// Automatic STOP condition generation. In slave mode, only settings 00b and 01b
/// are available.
#[derive(Copy, Clone, Default)]
pub enum Ucastp {
    /// No automatic STOP generation. The STOP condition is generated after
    /// the user sets the UCTXSTP bit. The value in UCBxTBCNT is a don't care.
    #[default]
    Ucastp00b = 0,
    /// UCBCNTIFG is set when the byte counter reaches the threshold defined in
    /// UCBxTBCNT
    Ucastp01b = 1,
    /// A STOP condition is generated automatically after the byte counter value
    /// reached UCBxTBCNT. UCBCNTIFG is set with the byte counter reaching the
    /// threshold.
    Ucastp10b = 2,
}

pub struct UcaCtlw0 {
    pub ucpen: bool,
    pub ucpar: bool,
    pub ucmsb: bool,
    pub uc7bit: bool,
    pub ucspb: bool,
    pub ucssel: Ucssel,
    pub ucrxeie: bool,
}

reg_struct! {
pub struct UcbCtlw0, UcbCtlw0_rd, UcbCtlw0_wr {
    flags{
        pub uca10: bool,
        pub ucsla10: bool,
        pub ucmm: bool,
        pub ucmst: bool,
        pub ucsync: bool,
        pub uctxack: bool,
        pub uctr: bool,
        pub uctxnack: bool,
        pub uctxstp: bool,
        pub uctxstt: bool,
        pub ucswrst: bool,
    }
    enums{
        pub ucmode: Ucmode : u8,
        pub ucssel: Ucssel : u8,
    }
}
}

reg_struct! {
pub struct UcbCtlw1, UcbCtlw1_rd, UcbCtlw1_wr {
    flags{
        pub ucetxint: bool,
        pub ucstpnack: bool,
        pub ucswack: bool,
    }
    enums{
        pub ucclto: Ucclto : u8,
        pub ucastp: Ucastp : u8,
        pub ucglit: Ucglit : u8,
    }
}
}

// in order to avoid 4 separate structs, I manually implemented the macro for these registers
#[derive(Debug, Default)]
pub struct UcbI2coa {
    pub ucgcen: bool,
    pub ucoaen: bool,
    pub i2coa0: u16,
}

reg_struct! {
pub struct UcxSpiCtw0, UcxSpiCtw0_rd, UcxSpiCtw0_wr{
    flags{
        pub ucckph: bool,
        pub ucckpl: bool,
        pub ucmsb: bool,
        pub uc7bit: bool,
        pub ucmst: bool,
        pub ucsync: bool,
        pub ucstem: bool,
        pub ucswrst: bool,
    }
    enums{
        pub ucmode: Ucmode : u8,
        pub ucssel: Ucssel : u8,
    }
}
}

pub trait EUsciUart: Steal {
    type Statw: UartUcxStatw;

    fn ctl0_reset(&self);

    // only call while in reset state
    fn brw_settings(&self, ucbr: u16);

    // only call while in reset state
    fn loopback(&self, loopback: bool);

    fn txifg_rd(&self) -> bool;
    fn rxifg_rd(&self) -> bool;

    fn rx_rd(&self) -> u8;
    fn tx_wr(&self, val: u8);

    fn iv_rd(&self) -> u16;

    // only call while in reset state
    fn ctl0_settings(&self, reg: UcaCtlw0);

    fn mctlw_settings(&self, ucos16: bool, ucbrs: u8, ucbrf: u8);

    fn statw_rd(&self) -> <Self as EUsciUart>::Statw;

    fn txie_set(&self);
    fn txie_clear(&self);
    fn rxie_set(&self);
    fn rxie_clear(&self);
}

pub trait EUsciI2C: Steal {
    type IfgOut: I2CUcbIfgOut;

    fn transmit_ack(&self);
    fn transmit_nack(&self);
    fn transmit_start(&self);
    fn transmit_stop(&self);

    fn uctxstt_rd(&self) -> bool;
    fn uctxstp_rd(&self) -> bool;

    fn start_received(&self) -> bool;
    fn stop_received(&self) -> bool;
    fn clear_start_flag(&self);
    fn clear_start_stop_flags(&self);

    fn set_ucsla10(&self, bit: bool);
    fn set_uctr(&self, bit: bool);
    fn set_master(&self);

    fn txifg0_rd(&self) -> bool;
    fn rxifg0_rd(&self) -> bool;

    //Register read/write functions

    // Read or write to UCSWRST
    fn ctw0_rd_rst(&self) -> bool;
    fn ctw0_set_rst(&self);
    fn ctw0_clear_rst(&self);

    // Modify only when UCSWRST = 1
    fn ctw0_wr(&self, reg: &UcbCtlw0);

    fn is_master(&self) -> bool;
    fn is_bus_busy(&self) -> bool;
    fn is_transmitter(&self) -> bool;

    // Modify only when UCSWRST = 1
    fn ctw1_wr(&self, reg: &UcbCtlw1);

    // Modify only when UCSWRST = 1
    fn brw_rd(&self) -> u16;
    fn brw_wr(&self, val: u16);

    fn byte_count(&self) -> u8; 

    // Modify only when UCSWRST = 1
    fn tbcnt_rd(&self) -> u16;
    fn tbcnt_wr(&self, val: u16);

    fn ucrxbuf_rd(&self) -> u8;
    fn uctxbuf_wr(&self, val: u8);

    // Modify only when UCSWRST = 1
    // the which parameter is used to select one of the 4 registers
    fn i2coa_rd(&self, which: u8) -> UcbI2coa;
    fn i2coa_wr(&self, which: u8, reg: &UcbI2coa);

    fn addrx_rd(&self) -> u16;

    // Modify only when UCSWRST = 1
    fn addmask_rd(&self) -> u16;
    fn addmask_wr(&self, val: u16);

    fn i2csa_rd(&self) -> u16;
    fn i2csa_wr(&self, val: u16);

    fn ie_wr(&self, reg: u16);
    fn ie_set(&self, mask: u16);
    fn ie_clr(&self, mask: u16);

    fn ifg_rd(&self) -> Self::IfgOut;
    fn ifg_wr(&self, reg: u16);
    fn ifg_rst(&self);

    fn iv_rd(&self) -> u16;
}

pub trait EusciSPI: Steal {
    type Statw: SpiStatw;

    fn ctw0_set_rst(&self);
    fn ctw0_clear_rst(&self);

    fn ctw0_wr(&self, reg: &UcxSpiCtw0);

    fn set_spi_mode(&self, mode: Mode);

    fn brw_wr(&self, val: u16);

    fn uclisten_set(&self);
    fn uclisten_clear(&self);

    fn rxbuf_rd(&self) -> u8;

    fn txbuf_wr(&self, val: u8);

    fn ie_rd(&self) -> u16;
    fn ie_wr(&self, reg: u16);

    fn set_transmit_interrupt(&self);

    fn clear_transmit_interrupt(&self);

    fn set_receive_interrupt(&self);

    fn clear_receive_interrupt(&self);

    fn transmit_flag(&self) -> bool;

    fn receive_flag(&self) -> bool;

    fn overrun_flag(&self) -> bool;

    fn iv_rd(&self) -> u16;

    fn is_busy(&self) -> bool;
}

pub trait UartUcxStatw {
    fn ucfe(&self) -> bool;
    fn ucoe(&self) -> bool;
    fn ucpe(&self) -> bool;
    fn ucbrk(&self) -> bool;
    fn ucbusy(&self) -> bool;
}

pub trait SpiStatw {
    fn uclisten(&self) -> bool;
    fn ucfe(&self) -> bool;
    fn ucoe(&self) -> bool;
    fn ucbusy(&self) -> bool;
}

pub trait I2CUcbIfgOut {
    /// Byte counter interrupt flag
    fn ucbcntifg(&self) -> bool;
    /// Not-acknowledge received interrupt flag
    fn ucnackifg(&self) -> bool;
    /// Arbitration lost interrupt flag
    fn ucalifg(&self) -> bool;
    /// STOP condition interrupt flag
    fn ucstpifg(&self) -> bool;
    /// START condition interrupt flag
    fn ucsttifg(&self) -> bool;
    /// eUSCI_B transmit interrupt flag 0. (UCBxTXBUF is empty)
    fn uctxifg0(&self) -> bool;
    /// eUSCI_B receive interrupt flag 0. (complete character present in UCBxRXBUF)
    fn ucrxifg0(&self) -> bool;
}

macro_rules! eusci_impl {
    ($intr_vec:ident, $EUsci:ident, $eusci:ident, $ucxctlw0:ident, $ucxctlw1:ident, $ucxbrw:ident,
     $ucxstatw:ident, $ucxrxbuf:ident, $ucxtxbuf:ident, $ucxie:ident, $ucxifg:ident,
     $ucxiv:ident, $StatwSpi:ty) => {
        impl Steal for pac::$EUsci {
            #[inline(always)]
            unsafe fn steal() -> Self {
                pac::Peripherals::conjure().$EUsci
            }
        }

        impl EusciSPI for pac::$EUsci {
            type Statw = $StatwSpi;

            #[inline(always)]
            fn ctw0_set_rst(&self) {
                unsafe { self.$ucxctlw0().set_bits(|w| w.ucswrst().set_bit()) }
            }

            #[inline(always)]
            fn ctw0_clear_rst(&self) {
                unsafe { self.$ucxctlw0().clear_bits(|w| w.ucswrst().clear_bit()) }
            }

            #[inline(always)]
            fn ctw0_wr(&self, reg: &UcxSpiCtw0) {
                self.$ucxctlw0().write(UcxSpiCtw0_wr! {reg});
            }

            #[inline(always)]
            fn brw_wr(&self, val: u16) {
                self.$ucxbrw().write(|w| unsafe { w.bits(val) });
            }

            #[inline(always)]
            fn uclisten_set(&self) {
                unsafe { self.$ucxstatw().set_bits(|w| w.uclisten().set_bit()) }
            }
            #[inline(always)]
            fn uclisten_clear(&self) {
                unsafe { self.$ucxstatw().clear_bits(|w| w.uclisten().clear_bit()) }
            }

            #[inline(always)]
            fn rxbuf_rd(&self) -> u8 {
                self.$ucxrxbuf().read().ucrxbuf().bits()
            }

            #[inline(always)]
            fn txbuf_wr(&self, val: u8) {
                self.$ucxtxbuf().write(|w| unsafe { w.uctxbuf().bits(val) });
            }

            #[inline(always)]
            fn ie_rd(&self) -> u16 {
                self.$ucxie().read().bits()
            }

            #[inline(always)]
            fn ie_wr(&self, reg: u16) {
                self.$ucxie().write(|w| unsafe { w.bits(reg) })
            }

            #[inline(always)]
            fn set_transmit_interrupt(&self) {
                unsafe { self.$ucxie().set_bits(|w| w.uctxie().set_bit()) }
            }

            #[inline(always)]
            fn clear_transmit_interrupt(&self) {
                unsafe { self.$ucxie().clear_bits(|w| w.uctxie().clear_bit()) }
            }

            #[inline(always)]
            fn set_receive_interrupt(&self) {
                unsafe { self.$ucxie().set_bits(|w| w.ucrxie().set_bit()) }
            }

            #[inline(always)]
            fn clear_receive_interrupt(&self) {
                unsafe { self.$ucxie().clear_bits(|w| w.ucrxie().clear_bit()) }
            }

            #[inline(always)]
            // Set the SPI mode without disturbing the rest of the register.
            fn set_spi_mode(&self, mode: Mode) {
                let ucckph = match mode.phase {
                    Phase::CaptureOnFirstTransition => true,
                    Phase::CaptureOnSecondTransition => false,
                };
                let ucckpl = match mode.polarity {
                    Polarity::IdleLow => false,
                    Polarity::IdleHigh => true,
                };
                self.$ucxctlw0().modify(|_, w| w
                    .ucckph().bit(ucckph)
                    .ucckpl().bit(ucckpl));
            }

            #[inline(always)]
            fn transmit_flag(&self) -> bool {
                self.$ucxifg().read().uctxifg().bit()
            }

            #[inline(always)]
            fn receive_flag(&self) -> bool {
                self.$ucxifg().read().ucrxifg().bit()
            }

            #[inline(always)]
            fn overrun_flag(&self) -> bool {
                self.$ucxstatw().read().ucoe().bit()
            }

            #[inline(always)]
            fn iv_rd(&self) -> u16 {
                self.$ucxiv().read().uciv().bits()
            }

            fn is_busy(&self) -> bool {
                self.$ucxstatw().read().ucbusy()
            }
        }

        impl SpiStatw for $StatwSpi {
            #[inline(always)]
            fn uclisten(&self) -> bool {
                self.uclisten().bit()
            }

            #[inline(always)]
            fn ucfe(&self) -> bool {
                self.ucfe().bit()
            }

            #[inline(always)]
            fn ucoe(&self) -> bool {
                self.ucoe().bit()
            }

            #[inline(always)]
            fn ucbusy(&self) -> bool {
                // TODO: Update the PAC
                // The PAC is currently missing the UCBUSY bit in the SPI version of this register
                const UCBUSY_MASK: u16 = (1 << 0);
                (self.bits() & UCBUSY_MASK) > 0
            }
        }
    };
}

macro_rules! eusci_a_impl {
    ($intr_vec:ident,$EUsci:ident, $eusci:ident, $ucaxctlw0:ident, $ucaxctlw1:ident, $ucaxbrw:ident,
     $ucaxmctlw:ident, $ucaxstatw:ident, $ucaxrxbuf:ident, $ucaxtxbuf:ident, $ucaxie:ident,
     $ucaxifg:ident, $ucaxiv:ident, $Statw:ty,
     $StatwSpi:ty,
     $ucaxctlw0spi:ident, $ucaxstatwspi:ident, $ucaxiespi:ident, $ucaxifgspi:ident) => {
        eusci_impl!(
            $intr_vec,
            $EUsci,
            $eusci,
            $ucaxctlw0spi,
            $ucaxctlw1,
            $ucaxbrw,
            $ucaxstatwspi,
            $ucaxrxbuf,
            $ucaxtxbuf,
            $ucaxiespi,
            $ucaxifgspi,
            $ucaxiv,
            $StatwSpi
        );

        impl EUsciUart for pac::$EUsci {
            type Statw = $Statw;

            #[inline(always)]
            fn ctl0_settings(&self, reg: UcaCtlw0) {
                self.$ucaxctlw0().write(|w| {
                    w.ucpen()
                        .bit(reg.ucpen)
                        .ucpar()
                        .bit(reg.ucpar)
                        .ucmsb()
                        .bit(reg.ucmsb)
                        .uc7bit()
                        .bit(reg.uc7bit)
                        .ucspb()
                        .bit(reg.ucspb)
                        .ucssel()
                        .bits(reg.ucssel as u8)
                        .ucrxeie()
                        .bit(reg.ucrxeie)
                });
            }

            #[inline(always)]
            fn mctlw_settings(&self, ucos16: bool, ucbrs: u8, ucbrf: u8) {
                self.$ucaxmctlw.write(|w| unsafe {
                    w.ucos16()
                        .bit(ucos16)
                        .ucbrs()
                        .bits(ucbrs)
                        .ucbrf()
                        .bits(ucbrf)
                });
            }

            #[inline(always)]
            fn statw_rd(&self) -> <Self as EUsciUart>::Statw {
                self.$ucaxstatw().read()
            }

            #[inline(always)]
            fn txie_set(&self) {
                unsafe { self.$ucaxie().set_bits(|w| w.uctxie().set_bit()) };
            }

            #[inline(always)]
            fn txie_clear(&self) {
                unsafe { self.$ucaxie().clear_bits(|w| w.uctxie().clear_bit()) };
            }

            #[inline(always)]
            fn rxie_set(&self) {
                unsafe { self.$ucaxie().set_bits(|w| w.ucrxie().set_bit()) };
            }

            #[inline(always)]
            fn rxie_clear(&self) {
                unsafe { self.$ucaxie().clear_bits(|w| w.ucrxie().clear_bit()) };
            }

            #[inline(always)]
            fn ctl0_reset(&self) {
                self.$ucaxctlw0().write(|w| w.ucswrst().set_bit());
            }

            #[inline(always)]
            fn brw_settings(&self, ucbr: u16) {
                self.$ucaxbrw().write(|w| unsafe { w.bits(ucbr) });
            }

            #[inline(always)]
            fn loopback(&self, loopback: bool) {
                self.$ucaxstatw().write(|w| w.uclisten().bit(loopback));
            }

            #[inline(always)]
            fn rx_rd(&self) -> u8 {
                self.$ucaxrxbuf().read().ucrxbuf().bits()
            }

            #[inline(always)]
            fn tx_wr(&self, bits: u8) {
                self.$ucaxtxbuf()
                    .write(|w| unsafe { w.uctxbuf().bits(bits) });
            }

            #[inline(always)]
            fn txifg_rd(&self) -> bool {
                self.$ucaxifg().read().uctxifg().bit()
            }

            #[inline(always)]
            fn rxifg_rd(&self) -> bool {
                self.$ucaxifg().read().ucrxifg().bit()
            }

            #[inline(always)]
            fn iv_rd(&self) -> u16 {
                self.$ucaxiv().read().bits()
            }
        }

        impl UartUcxStatw for $Statw {
            #[inline(always)]
            fn ucfe(&self) -> bool {
                self.ucfe().bit()
            }

            #[inline(always)]
            fn ucoe(&self) -> bool {
                self.ucoe().bit()
            }

            #[inline(always)]
            fn ucpe(&self) -> bool {
                self.ucpe().bit()
            }

            #[inline(always)]
            fn ucbrk(&self) -> bool {
                self.ucbrk().bit()
            }

            #[inline(always)]
            fn ucbusy(&self) -> bool {
                self.ucbusy().bit()
            }
        }
    };
}

macro_rules! eusci_b_impl {
    ($intr_vec:ident, $EUsci:ident, $eusci:ident, $ucbxctlw0:ident, $ucbxctlw1:ident, $ucbxbrw:ident,
     $ucbxstatw:ident, $ucbxtbcnt:ident, $ucbxrxbuf:ident, $ucbxtxbuf:ident, $ucbxi2coa0:ident,
     $ucbxi2coa1:ident, $ucbxi2coa2:ident, $ucbxi2coa3:ident, $ucbxaddrx:ident, $ucbxaddmask:ident,
     $ucbxi2csa:ident, $ucbxie:ident,
     $ucbxifg:ident, $ucbxiv:ident, $Statw:ty, $Ifg:ty,
     $StatwSpi:ty,
     $ucbxctlw0spi:ident, $ucbxstatwspi:ident, $ucbxiespi:ident, $ucbxifgspi:ident) => {
        eusci_impl!(
            $intr_vec,
            $EUsci,
            $eusci,
            $ucbxctlw0spi,
            $ucbxctlw1,
            $ucbxbrw,
            $ucbxstatwspi,
            $ucbxrxbuf,
            $ucbxtxbuf,
            $ucbxiespi,
            $ucbxifgspi,
            $ucbxiv,
            $StatwSpi
        );

        impl EUsciI2C for pac::$EUsci {
            type IfgOut = $Ifg;

            #[inline(always)]
            fn ctw0_rd_rst(&self) -> bool {
                self.$ucbxctlw0().read().ucswrst().bit()
            }

            #[inline(always)]
            fn ctw0_set_rst(&self) {
                unsafe { self.$ucbxctlw0().set_bits(|w| w.ucswrst().set_bit()) }
            }

            #[inline(always)]
            fn ctw0_clear_rst(&self) {
                unsafe { self.$ucbxctlw0().clear_bits(|w| w.ucswrst().clear_bit()) }
            }

            #[inline(always)]
            fn transmit_ack(&self) {
                unsafe { self.$ucbxctlw0().set_bits(|w| w.uctxack().set_bit()) }
            }

            #[inline(always)]
            fn transmit_nack(&self) {
                unsafe { self.$ucbxctlw0().set_bits(|w| w.uctxnack().set_bit()) }
            }

            #[inline(always)]
            fn transmit_start(&self) {
                unsafe { self.$ucbxctlw0().set_bits(|w| w.uctxstt().set_bit()) }
            }

            #[inline(always)]
            fn transmit_stop(&self) {
                unsafe { self.$ucbxctlw0().set_bits(|w| w.uctxstp().set_bit()) }
            }

            #[inline(always)]
            fn uctxstt_rd(&self) -> bool {
                self.$ucbxctlw0().read().uctxstt().bit()
            }

            #[inline(always)]
            fn uctxstp_rd(&self) -> bool {
                self.$ucbxctlw0().read().uctxstp().bit()
            }

            #[inline(always)]
            fn start_received(&self) -> bool {
                self.$ucbxifg().read().ucsttifg().bit()
            }

            #[inline(always)]
            fn stop_received(&self) -> bool {
                self.$ucbxifg().read().ucstpifg().bit()
            }

            #[inline(always)]
            fn clear_start_flag(&self) {
                unsafe{ self.$ucbxifg().clear_bits(|w| w.ucsttifg().clear_bit()) }
            }

            #[inline(always)]
            fn clear_start_stop_flags(&self) {
                unsafe{ self.$ucbxifg().clear_bits(|w| w.ucstpifg().clear_bit().ucsttifg().clear_bit()) }
            }

            #[inline(always)]
            fn set_master(&self) {
                unsafe { self.$ucbxctlw0().set_bits(|w| w.ucmst().set_bit()) }
            }

            #[inline(always)]
            fn set_ucsla10(&self, bit: bool) {
                match bit {
                    true => unsafe { self.$ucbxctlw0().set_bits(|w| w.ucsla10().set_bit()) },
                    false => unsafe { self.$ucbxctlw0().clear_bits(|w| w.ucsla10().clear_bit()) },
                }
            }

            #[inline(always)]
            fn set_uctr(&self, bit: bool) {
                match bit {
                    true => unsafe { self.$ucbxctlw0().set_bits(|w| w.uctr().set_bit()) },
                    false => unsafe { self.$ucbxctlw0().clear_bits(|w| w.uctr().clear_bit()) },
                }
            }

            #[inline(always)]
            fn txifg0_rd(&self) -> bool {
                self.$ucbxifg().read().uctxifg0().bit()
            }

            #[inline(always)]
            fn rxifg0_rd(&self) -> bool {
                self.$ucbxifg().read().ucrxifg0().bit()
            }

            #[inline(always)]
            fn ctw0_wr(&self, reg: &UcbCtlw0) {
                self.$ucbxctlw0().write(UcbCtlw0_wr! {reg});
            }
            
            #[inline(always)]
            fn is_master(&self) -> bool {
                self.$ucbxctlw0().read().ucmst().bit_is_set()
            }

            #[inline(always)]
            fn is_bus_busy(&self) -> bool {
                self.$ucbxstatw().read().ucbbusy().bit_is_set()
            }

            #[inline(always)]
            fn is_transmitter(&self) -> bool {
                self.$ucbxctlw0().read().uctr().bit_is_set()
            }

            #[inline(always)]
            fn ctw1_wr(&self, reg: &UcbCtlw1) {
                self.$ucbxctlw1.write(UcbCtlw1_wr! {reg});
            }

            #[inline(always)]
            fn brw_rd(&self) -> u16 {
                self.$ucbxbrw().read().bits()
            }
            #[inline(always)]
            fn brw_wr(&self, val: u16) {
                self.$ucbxbrw().write(|w| unsafe { w.bits(val) });
            }

            #[inline(always)]
            fn byte_count(&self) -> u8 {
                self.$ucbxstatw().read().ucbcnt().bits()
            }

            #[inline(always)]
            fn tbcnt_rd(&self) -> u16 {
                self.$ucbxtbcnt.read().bits()
            }
            #[inline(always)]
            fn tbcnt_wr(&self, val: u16) {
                self.$ucbxtbcnt.write(|w| unsafe { w.bits(val) });
            }

            #[inline(always)]
            fn ucrxbuf_rd(&self) -> u8 {
                self.$ucbxrxbuf().read().bits() as u8
            }
            #[inline(always)]
            fn uctxbuf_wr(&self, val: u8) {
                self.$ucbxtxbuf().write(|w| unsafe { w.bits(val as u16) });
            }

            fn i2coa_rd(&self, which: u8) -> UcbI2coa {
                match which {
                    1 => {
                        let content = self.$ucbxi2coa1.read();
                        UcbI2coa {
                            ucgcen: false,
                            ucoaen: content.ucoaen().bit(),
                            i2coa0: <u16>::from(content.i2coa1().bits()),
                        }
                    }
                    2 => {
                        let content = self.$ucbxi2coa2.read();
                        UcbI2coa {
                            ucgcen: false,
                            ucoaen: content.ucoaen().bit(),
                            i2coa0: <u16>::from(content.i2coa2().bits()),
                        }
                    }
                    3 => {
                        let content = self.$ucbxi2coa3.read();
                        UcbI2coa {
                            ucgcen: false,
                            ucoaen: content.ucoaen().bit(),
                            i2coa0: <u16>::from(content.i2coa3().bits()),
                        }
                    }
                    _ => {
                        let content = self.$ucbxi2coa0.read();
                        UcbI2coa {
                            ucgcen: content.ucgcen().bit(),
                            ucoaen: content.ucoaen().bit(),
                            i2coa0: <u16>::from(content.i2coa0().bits()),
                        }
                    }
                }
            }

            fn i2coa_wr(&self, which: u8, reg: &UcbI2coa) {
                match which {
                    1 => {
                        self.$ucbxi2coa1.write(|w| unsafe {
                            w.ucoaen().bit(reg.ucoaen).i2coa1().bits(reg.i2coa0 as u16)
                        });
                    }
                    2 => {
                        self.$ucbxi2coa2.write(|w| unsafe {
                            w.ucoaen().bit(reg.ucoaen).i2coa2().bits(reg.i2coa0 as u16)
                        });
                    }
                    3 => {
                        self.$ucbxi2coa3.write(|w| unsafe {
                            w.ucoaen().bit(reg.ucoaen).i2coa3().bits(reg.i2coa0 as u16)
                        });
                    }
                    _ => {
                        self.$ucbxi2coa0.write(|w| unsafe {
                            w.ucgcen()
                                .bit(reg.ucgcen)
                                .ucoaen()
                                .bit(reg.ucoaen)
                                .i2coa0()
                                .bits(reg.i2coa0 as u16)
                        });
                    }
                }
            }

            #[inline(always)]
            fn addrx_rd(&self) -> u16 {
                self.$ucbxaddrx.read().bits()
            }

            #[inline(always)]
            fn addmask_rd(&self) -> u16 {
                self.$ucbxaddmask.read().bits()
            }
            #[inline(always)]
            fn addmask_wr(&self, val: u16) {
                self.$ucbxaddmask.write(|w| unsafe { w.bits(val) });
            }

            #[inline(always)]
            fn i2csa_rd(&self) -> u16 {
                self.$ucbxi2csa.read().bits()
            }
            #[inline(always)]
            fn i2csa_wr(&self, val: u16) {
                self.$ucbxi2csa.write(|w| unsafe { w.bits(val) });
            }

            #[inline(always)]
            fn ie_wr(&self, reg: u16) {
                self.$ucbxie().write(|w| unsafe { w.bits(reg) });
            }

            #[inline(always)]
            fn ie_set(&self, mask: u16) {
                unsafe{ self.$ucbxie().set_bits(|w| w.bits(mask)) };
            }
            #[inline(always)]
            fn ie_clr(&self, mask: u16) {
                unsafe{ self.$ucbxie().clear_bits(|w| w.bits(mask)) };
            }

            #[inline(always)]
            fn ifg_rd(&self) -> Self::IfgOut {
                self.$ucbxifg().read()
            }

            #[inline(always)]
            fn ifg_wr(&self, reg: u16) {
                self.$ucbxifg().write(|w| unsafe { w.bits(reg) });
            }

            #[inline(always)]
            fn ifg_rst(&self) {
                self.$ucbxifg().reset();
            }

            #[inline(always)]
            fn iv_rd(&self) -> u16 {
                self.$ucbxiv().read().bits()
            }
        }

        impl I2CUcbIfgOut for $Ifg {
            #[inline(always)]
            fn ucbcntifg(&self) -> bool {
                self.ucbcntifg().bit()
            }

            #[inline(always)]
            fn ucnackifg(&self) -> bool {
                self.ucnackifg().bit()
            }

            #[inline(always)]
            fn ucalifg(&self) -> bool {
                self.ucalifg().bit()
            }

            #[inline(always)]
            fn ucstpifg(&self) -> bool {
                self.ucstpifg().bit()
            }

            #[inline(always)]
            fn ucsttifg(&self) -> bool {
                self.ucsttifg().bit()
            }

            #[inline(always)]
            fn uctxifg0(&self) -> bool {
                self.uctxifg0().bit()
            }

            #[inline(always)]
            fn ucrxifg0(&self) -> bool {
                self.ucrxifg0().bit()
            }
        }
    };
}

eusci_a_impl!(
    EUSCI_A0,
    E_USCI_A0,
    e_usci_a0,
    uca0ctlw0,
    uca0ctlw1,
    uca0brw,
    uca0mctlw,
    uca0statw,
    uca0rxbuf,
    uca0txbuf,
    uca0ie,
    uca0ifg,
    uca0iv,
    pac::e_usci_a0::uca0statw::R,
    pac::e_usci_a0::uca0statw_spi::R,
    uca0ctlw0_spi,
    uca0statw_spi,
    uca0ie_spi,
    uca0ifg_spi
);

eusci_a_impl!(
    EUSCI_A1,
    E_USCI_A1,
    e_usci_a1,
    uca1ctlw0,
    uca1ctlw1,
    uca1brw,
    uca1mctlw,
    uca1statw,
    uca1rxbuf,
    uca1txbuf,
    uca1ie,
    uca1ifg,
    uca1iv,
    pac::e_usci_a1::uca1statw::R,
    pac::e_usci_a1::uca1statw_spi::R,
    uca1ctlw0_spi,
    uca1statw_spi,
    uca1ie_spi,
    uca1ifg_spi
);

eusci_b_impl!(
    EUSCI_B0,
    E_USCI_B0,
    e_usci_b0,
    ucb0ctlw0,
    ucb0ctlw1,
    ucb0brw,
    ucb0statw,
    ucb0tbcnt,
    ucb0rxbuf,
    ucb0txbuf,
    ucb0i2coa0,
    ucb0i2coa1,
    ucb0i2coa2,
    ucb0i2coa3,
    ucb0addrx,
    ucb0addmask,
    ucb0i2csa,
    ucb0ie,
    ucb0ifg,
    ucb0iv,
    pac::e_usci_b0::ucb0statw::R,
    pac::e_usci_b0::ucb0ifg::R,
    pac::e_usci_b0::ucb0statw_spi::R,
    ucb0ctlw0_spi,
    ucb0statw_spi,
    ucb0ie_spi,
    ucb0ifg_spi
);

eusci_b_impl!(
    EUSCI_B1,
    E_USCI_B1,
    e_usci_b1,
    ucb1ctlw0,
    ucb1ctlw1,
    ucb1brw,
    ucb1statw,
    ucb1tbcnt,
    ucb1rxbuf,
    ucb1txbuf,
    ucb1i2coa0,
    ucb1i2coa1,
    ucb1i2coa2,
    ucb1i2coa3,
    ucb1addrx,
    ucb1addmask,
    ucb1i2csa,
    ucb1ie,
    ucb1ifg,
    ucb1iv,
    pac::e_usci_b1::ucb1statw::R,
    pac::e_usci_b1::ucb1ifg::R,
    pac::e_usci_b1::ucb1statw_spi::R,
    ucb1ctlw0_spi,
    ucb1statw_spi,
    ucb1ie_spi,
    ucb1ifg_spi
);
