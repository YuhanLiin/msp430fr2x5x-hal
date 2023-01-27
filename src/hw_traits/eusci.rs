use super::Steal;
use msp430fr2355 as pac;

use crate::pac::e_usci_b0::ucb0ctlw1::{UCCLTO_A, UCASTP_A};
use crate::pac::e_usci_b0::ucb0statw::{UCSCLLOW_A, UCGC_A, UCBBUSY_A};
use crate::pac::e_usci_b0::ucb0ctlw0::{UCMODE_A};
use crate::pac::e_usci_b0::ucb0ie::{UCBIT9IE_A, UCTXIE3_A, UCRXIE3_A, UCTXIE2_A, UCRXIE2_A,
                                    UCTXIE1_A, UCRXIE1_A, UCCLTOIE_A, UCBCNTIE_A, UCNACKIE_A,
                                    UCALIE_A, UCSTPIE_A, UCSTTIE_A, UCTXIE0_A, UCRXIE0_A};
use crate::pac::e_usci_b0::ucb0ifg::{UCBIT9IFG_A, UCTXIFG3_A, UCRXIFG3_A, UCTXIFG2_A, UCRXIFG2_A,
                                     UCTXIFG1_A, UCRXIFG1_A, UCCLTOIFG_A, UCBCNTIFG_A, UCNACKIFG_A,
                                     UCALIFG_A, UCSTPIFG_A, UCSTTIFG_A, UCTXIFG0_A, UCRXIFG0_A};
use crate::pac::e_usci_b0::ucb0iv::UCIV_A;


macro_rules! from_u8 {
    ($type: ty) => {
        impl From<u8> for $type {
            #[inline(always)]
            fn from(variant: u8) -> Self {
                <$type>::from(variant)
            }
        }
    };
}

/// Defines a macro to
/// Also
macro_rules! reg_struct {
    (
        pub struct $struct_name : ident, $macro_rd : ident, $macro_wr : ident {
            flags{
                $(pub $bool_name : ident : bool,)*
            }
            vals{
                $(pub $val_name : ident : $val_type : ty : $size : ty,)*
            }

        }
    ) => {
        pub struct $struct_name {
            $(pub $bool_name : bool,)*
            $(pub $val_name : $val_type ,)*
        }
        macro_rules! $macro_rd {
            ($reader : expr) => {
                $struct_name{
                    $($bool_name : $reader.$bool_name().bit(),)*
                    $($val_name : <$val_type>::from(<$size>::from($reader.$val_name().variant())),)*
                }
            };
        }

        macro_rules! $macro_wr {
            ($reg : expr) => { |w| {
                w$(.$bool_name().bit($reg.$bool_name))*
                 $(.$val_name().bits($reg.$val_name as $size))*
            }};
        }
    };
}

pub enum Ucssel {
    Uclk,
    Aclk,
    Smclk,
}

from_u8!(Ucssel);

pub enum Ucmode {
    ThreePinSPI,
    FourPinSPI_1,
    FourPinSPI_0,
    I2CMode,
}

from_u8!(Ucmode);

pub enum Ucglit {
    Max50ns,
    Max25ns,
    Max12_5ns,
    Max6_25ns,
}

from_u8!(Ucglit);

pub struct UcaCtlw0{
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
    vals{
        pub ucmode: Ucmode : u8,
        pub ucssel: Ucssel : u8,
    }
}
}

pub struct UcbCtlw1 {
    pub ucetxint: bool,
    pub ucclto: UCCLTO_A,
    pub ucstpnack: bool,
    pub ucswack: bool,
    pub ucastp: UCASTP_A,
    pub ucglit: Ucglit,
}

pub struct UcbStatw {
    pub ucbcnt: u8,
    pub ucscllow: UCSCLLOW_A,
    pub ucgc: UCGC_A,
    pub ucbbusy: UCBBUSY_A,
}

pub struct UcbI2coa {
    /// ucgcen is only available for i2c0a0
    pub ucgcen: bool,
    pub ucoaen: bool,
    /// 10 bits
    pub i2coa0: u16,
}

pub struct UcbIe {
    pub ucbit9ie: bool,
    pub uctxie3: bool,
    pub ucrxie3: bool,
    pub uctxie2: bool,
    pub ucrxie2: bool,
    pub uctxie1: bool,
    pub ucrxie1: bool,
    pub uccltoie: bool,
    pub ucbcntie: bool,
    pub ucnackie: bool,
    pub ucalie: bool,
    pub ucstpie: bool,
    pub ucsttie: bool,
    pub uctxie0: bool,
    pub ucrxie0: bool,
}

pub struct UcbIFG {
    pub ucbit9ifg: bool,
    pub uctxifg3: bool,
    pub ucrxifg3: bool,
    pub uctxifg2: bool,
    pub ucrxifg2: bool,
    pub uctxifg1: bool,
    pub ucrxifg1: bool,
    pub uccltoifg: bool,
    pub ucbcntifg: bool,
    pub ucnackifg: bool,
    pub ucalifg: bool,
    pub ucstpifg: bool,
    pub ucsttifg: bool,
    pub uctxifg0: bool,
    pub ucrxifg0: bool,
}

pub trait EUsci : Steal {

}

pub trait EUsciUart: EUsci {
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

pub trait EUsciI2C: EUsci {

    // Read or write to UCSWRST
    fn ctw0_rd_rst(&self) -> bool;
    fn ctw0_wr_rst(&self, bit:bool);

    // Modify only when UCSWRST = 1
    fn ctw0_rd(&self) -> UcbCtlw0;
    fn ctw0_wr(&self, reg:UcbCtlw0);

    // Modify only when UCSWRST = 1
    fn ctw1_rd(&self) -> UcbCtlw1;
    fn ctw1_wr(&self, reg:UcbCtlw1);

    // Modify only when UCSWRST = 1
    fn brw_rd(&self) -> u16;
    fn brw_wr(&self, val:u16);

    fn statw_rd(&self) -> UcbStatw;

    // Modify only when UCSWRST = 1
    fn tbcnt_rd(&self) -> u16;
    fn tbcnt_wr(&self, val:u16);

    fn ucrxbuf_rd(&self) -> u8;
    fn uctxbuf_wr(&self, val: u8);

    // Modify only when UCSWRST = 1
    // the which parameter is used to select one of the 4 registers
    fn i2coa_rd(&self, which:u8) -> UcbI2coa;
    fn i2coa_wr(&self, which:u8, reg:UcbI2coa);

    fn addrx_rd(&self) -> u16;

    // Modify only when UCSWRST = 1
    fn addrmask_rd(&self) -> u16;
    fn addrmask_wr(&self, val:u16);

    fn i2csa_rd(&self) -> u16;
    fn i2csa_wr(&self, val:u16);

    fn ucbie_rd(&self) -> UcbIe;
    fn ucboe_wr(&self, reg:UcbIe);
    //some common bitwise operations for this register
    fn ucboe_rmw_or(&self, reg:UcbIe);
    fn ucboe_rmw_and(&self, reg:UcbIe);

    fn ifg_rd(&self) -> UcbIFG;
    fn ifg_wr(&self, reg: UcbIFG);
    fn iv_rd(&self) -> UCIV_A;
}

pub trait UcxStatw {

}

pub trait UartUcxStatw : UcxStatw {
    fn ucfe(&self) -> bool;
    fn ucoe(&self) -> bool;
    fn ucpe(&self) -> bool;
    fn ucbrk(&self) -> bool;
    fn ucbusy(&self) -> bool;
}

pub trait I2CUcxStatw : UcxStatw {

}

macro_rules! eusci_impl {
    ($EUsci:ident, $eusci:ident, $ucxctlw0:ident, $ucxctlw1:ident, $ucxbrw:ident,
     $ucxstatw:ident, $ucxrxbuf:ident, $ucxtxbuf:ident, $ucxie:ident, $ucxifg:ident,
     $ucxiv:ident, $Statw:ty) => {
        impl Steal for pac::$EUsci {
            #[inline(always)]
            unsafe fn steal() -> Self {
                pac::Peripherals::conjure().$EUsci
            }
        }

        impl EUsci for pac::$EUsci {

        }

        impl UcxStatw for $Statw {

        }
    }
}

macro_rules! eusci_a_impl {
    ($EUsci:ident, $eusci:ident, $ucaxctlw0:ident, $ucaxctlw1:ident, $ucaxbrw:ident, $ucaxmctlw:ident,
     $ucaxstatw:ident, $ucaxrxbuf:ident, $ucaxtxbuf:ident, $ucaxie:ident, $ucaxifg:ident,
     $ucaxiv:ident, $Statw:ty) => {

        eusci_impl!(
            $EUsci,
            $eusci,
            $ucaxctlw0,
            $ucaxctlw1,
            $ucaxbrw,
            $ucaxstatw,
            $ucaxrxbuf,
            $ucaxtxbuf,
            $ucaxie,
            $ucaxifg,
            $ucaxiv,
            $Statw
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
    ($EUsci:ident, $eusci:ident, $ucbxctlw0:ident, $ucbxctlw1:ident, $ucbxbrw:ident,
     $ucbxstatw:ident, $ucbxtbcnt:ident, $ucbxrxbuf:ident, $ucbxtxbuf:ident, $ucbxi2coa0:ident,
     $ucbxi2coa1:ident, $ucbxi2coa2:ident, $ucbxi2coa3:ident, $ucbxaddrx:ident, $ucbxaddrmask:ident,
     $ucbxi2csa:ident, $ucbxie:ident,
     $ucbxifg:ident, $ucbxiv:ident, $Statw:ty) => {
        eusci_impl!(
            $EUsci,
            $eusci,
            $ucbxctlw0,
            $ucbxctlw1,
            $ucbxbrw,
            $ucbxstatw,
            $ucbxrxbuf,
            $ucbxtxbuf,
            $ucbxie,
            $ucbxifg,
            $ucbxiv,
            $Statw
        );



        impl EUsciI2C for pac::$EUsci {
            #[inline(always)]
            fn ctw0_rd_rst(&self) -> bool{
                self.$ucbxctlw0().read().ucswrst().bit()
            }
            #[inline(always)]
            fn ctw0_wr_rst(&self, bit:bool){
                self.$ucbxctlw0().write(|w| unsafe{w.ucswrst().bit(bit)})
            }

            #[inline(always)]
            fn ctw0_rd(&self) -> UcbCtlw0{
                let content = self.$ucbxctlw0().read();
                UcbCtlw0_rd! {content}
            }

            #[inline(always)]
            fn ctw0_wr(&self, reg:UcbCtlw0){
                self.$ucbxctlw0().write(UcbCtlw0_wr! {reg});
            }

            #[inline(always)]
            fn ctw1_rd(&self) -> UcbCtlw1;
            #[inline(always)]
            fn ctw1_wr(&self, reg:UcbCtlw1);

            #[inline(always)]
            fn brw_rd(&self) -> u16;
            #[inline(always)]
            fn brw_wr(&self, val:u16);

            #[inline(always)]
            fn statw_rd(&self) -> UcbStatw;

            #[inline(always)]
            fn tbcnt_rd(&self) -> u16;
            #[inline(always)]
            fn tbcnt_wr(&self, val:u16);

            #[inline(always)]
            fn ucrxbuf_rd(&self) -> u8;
            #[inline(always)]
            fn uctxbuf_wr(&self, val: u8);

            #[inline(always)]
            fn i2coa_rd(&self, which:u8) -> UcbI2coa;
            #[inline(always)]
            fn i2coa_wr(&self, which:u8, reg:UcbI2coa);

            #[inline(always)]
            fn addrx_rd(&self) -> u16;

            #[inline(always)]
            fn addrmask_rd(&self) -> u16;
            #[inline(always)]
            fn addrmask_wr(&self, val:u16);

            #[inline(always)]
            fn i2csa_rd(&self) -> u16;
            #[inline(always)]
            fn i2csa_wr(&self, val:u16);

            #[inline(always)]
            fn ucbie_rd(&self) -> UcbIe;
            #[inline(always)]
            fn ucboe_wr(&self, reg:UcbIe);

            #[inline(always)]
            fn ucboe_rmw_or(&self, reg:UcbIe);
            #[inline(always)]
            fn ucboe_rmw_and(&self, reg:UcbIe);

            #[inline(always)]
            fn ifg_rd(&self) -> UcbIFG;
            #[inline(always)]
            fn ifg_wr(&self, reg: UcbIFG);
            #[inline(always)]
            fn iv_rd(&self) -> UCIV_A;
        }
     };
}

eusci_a_impl!(
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
    pac::e_usci_a0::uca0statw::R
);

eusci_a_impl!(
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
    pac::e_usci_a1::uca1statw::R
);

eusci_b_impl!(
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
    ucb0addrmask,
    ucb0i2csa,
    ucb0ie,
    ucb0ifg,
    ucb0iv,
    pac::e_usci_b0::ucb0statw::R
);

eusci_b_impl!(
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
    ucb1addrmask,
    ucb1i2csa,
    ucb1ie,
    ucb1ifg,
    ucb1iv,
    pac::e_usci_b1::ucb1statw::R
);


