use msp430fr2355 as pac;

pub enum Ucssel {
    Uclk,
    Aclk,
    Smclk,
}

pub struct UcxCtl0 {
    pub ucpen: bool,
    pub ucpar: bool,
    pub ucmsb: bool,
    pub uc7bit: bool,
    pub ucspb: bool,
    pub ucssel: Ucssel,
}

pub trait EUsci {
    fn ctl0_reset(&self);

    // only call while in reset state
    fn brw_settings(&self, ucbr: u16);

    // only call while in reset state
    fn loopback(&self, loopback: bool);

    fn rx_rd(&self) -> u8;

    fn tx_wr(&self, val: u8);

    fn txie_wr(&self, ie: bool);
    fn rxie_wr(&self, ie: bool);

    fn txifg_rd(&self) -> bool;
    fn rxifg_rd(&self) -> bool;

    fn iv_rd(&self) -> u16;
}

pub trait EUsciUart: EUsci {
    type Statw: UcaxStatw;

    // only call while in reset state
    fn ctl0_settings(&self, reg: UcxCtl0);

    fn mctlw_settings(&self, ucos16: bool, ucbrs: u8, ucbrf: u8);

    fn statw_rd(&self) -> Self::Statw;
}

pub trait UcaxStatw {
    fn ucfe(&self) -> bool;
    fn ucoe(&self) -> bool;
    fn ucpe(&self) -> bool;
    fn ucbrk(&self) -> bool;
    fn ucbusy(&self) -> bool;
}

macro_rules! eusci_a_impl {
    ($EUsci:ty, $ucaxctlw0:ident, $ucaxctlw1:ident, $ucaxbrw:ident, $ucaxmctlw:ident,
     $ucaxstatw:ident, $ucaxrxbuf:ident, $ucaxtxbuf:ident, $ucaxie:ident, $ucaxifg:ident,
     $ucaxiv:ident, $Statw:ty) => {
        impl EUsci for $EUsci {
            fn ctl0_reset(&self) {
                self.$ucaxctlw0().write(|w| w.ucswrst().set_bit());
            }

            fn brw_settings(&self, ucbr: u16) {
                self.$ucaxbrw().write(|w| unsafe { w.bits(ucbr) });
            }

            fn loopback(&self, loopback: bool) {
                self.$ucaxstatw().write(|w| w.uclisten().bit(loopback));
            }

            fn rx_rd(&self) -> u8 {
                self.$ucaxrxbuf().read().ucrxbuf().bits()
            }

            fn tx_wr(&self, bits: u8) {
                self.$ucaxtxbuf()
                    .write(|w| unsafe { w.uctxbuf().bits(bits) });
            }

            fn txie_wr(&self, ie: bool) {
                self.$ucaxie().write(|w| w.uctxie().bit(ie));
            }

            fn rxie_wr(&self, ie: bool) {
                self.$ucaxie().write(|w| w.ucrxie().bit(ie));
            }

            fn txifg_rd(&self) -> bool {
                self.$ucaxifg().read().uctxifg().bit()
            }

            fn rxifg_rd(&self) -> bool {
                self.$ucaxifg().read().ucrxifg().bit()
            }

            fn iv_rd(&self) -> u16 {
                self.$ucaxiv().read().bits()
            }
        }

        impl EUsciUart for $EUsci {
            type Statw = $Statw;

            fn ctl0_settings(&self, reg: UcxCtl0) {
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
                });
            }

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

            fn statw_rd(&self) -> Self::Statw {
                self.$ucaxstatw().read()
            }
        }

        impl UcaxStatw for $Statw {
            fn ucfe(&self) -> bool {
                self.ucfe().bit()
            }

            fn ucoe(&self) -> bool {
                self.ucoe().bit()
            }

            fn ucpe(&self) -> bool {
                self.ucpe().bit()
            }

            fn ucbrk(&self) -> bool {
                self.ucbrk().bit()
            }

            fn ucbusy(&self) -> bool {
                self.ucbusy().bit()
            }
        }
    };
}

eusci_a_impl!(
    pac::E_USCI_A0,
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
    pac::E_USCI_A1,
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
