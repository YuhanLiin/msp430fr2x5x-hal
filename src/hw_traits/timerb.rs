use msp430fr2355 as pac;

pub enum Tbssel {
    Tbxclk,
    Aclk,
    Smclk,
    Inclk,
}

/// Timer clock divider
pub enum TimerDiv {
    /// No division
    _1,
    /// Divide by 2
    _2,
    /// Divide by 4
    _4,
    /// Divide by 8
    _8,
}

/// Timer expansion clock divider, applied on top of the normal clock divider
pub enum TimerExDiv {
    /// No division
    _1,
    /// Divide by 2
    _2,
    /// Divide by 3
    _3,
    /// Divide by 4
    _4,
    /// Divide by 5
    _5,
    /// Divide by 6
    _6,
    /// Divide by 7
    _7,
    /// Divide by 8
    _8,
}

pub enum Outmod {
    Out,
    Set,
    ToggleReset,
    SetReset,
    Toggle,
    Reset,
    ToggleSet,
    ResetSet,
}

pub enum CapMode {
    NoCap,
    RisingEdge,
    FallingEdge,
    BothEdges,
}

pub enum CapSelect {
    InputA,
    InputB,
}

pub trait TimerB {
    /// Reset timer countdown
    fn reset(&self);

    /// Set to upmode, reset timer, and clear interrupts
    fn upmode(&self);
    /// Set to continuous mode, reset timer, and clear interrupts
    fn continuous(&self);

    /// Apply clock select settings
    fn config_clock(&self, tbssel: Tbssel, div: TimerDiv);

    /// Check if timer is stopped
    fn is_stopped(&self) -> bool;

    /// Stop timer
    fn stop(&self);

    /// Set expansion register clock divider settings
    fn set_tbidex(&self, tbidex: TimerExDiv);

    fn tbifg_rd(&self) -> bool;
    fn tbifg_clr(&self);

    fn tbie_set(&self);
    fn tbie_clr(&self);

    fn set_ccrn(&self, ccrn: CCRn, count: u16);
    fn get_ccrn(&self, ccrn: CCRn) -> u16;

    fn config_cmp_mode(&self, ccrn: CCRn, outmod: Outmod);

    fn config_cap_mode(&self, ccrn: CCRn, cm: CapMode, ccis: CapSelect);

    fn ccifg_rd(&self, ccrn: CCRn) -> bool;
    fn ccifg_clr(&self, ccrn: CCRn);

    fn ccie_set(&self, ccrn: CCRn);
    fn ccie_clr(&self, ccrn: CCRn);

    fn cov_rd(&self, ccrn: CCRn) -> bool;
    fn cov_ccifg_clr(&self, ccrn: CCRn);
}

pub enum CCRn {
    CCR0,
    CCR1,
    CCR2,
    CCR3,
    CCR4,
    CCR5,
    CCR6,
}

macro_rules! timerb_impl {
    ($TBx:ident, $tbx:ident, $tbxctl:ident, $tbxex:ident, $([$CCRn:ident, $tbxcctln:ident, $tbxccrn:ident]),*) => {
        impl TimerB for pac::$TBx {
            #[inline(always)]
            fn reset(&self) {
                unsafe { self.$tbxctl.set_bits(|w| w.tbclr().set_bit()) };
            }

            #[inline(always)]
            fn upmode(&self) {
                self.$tbxctl.modify(|r, w| {
                    unsafe { w.bits(r.bits()) }
                        .tbclr()
                        .set_bit()
                        .tbifg()
                        .clear_bit()
                        .mc()
                        .up()
                });
            }

            #[inline(always)]
            fn continuous(&self) {
                self.$tbxctl.modify(|r, w| {
                    unsafe { w.bits(r.bits()) }
                        .tbclr()
                        .set_bit()
                        .tbifg()
                        .clear_bit()
                        .mc()
                        .continuous()
                });
            }

            #[inline(always)]
            fn config_clock(&self, tbssel: Tbssel, div: TimerDiv) {
                self.$tbxctl
                    .write(|w| w.tbssel().bits(tbssel as u8).id().bits(div as u8));
            }

            #[inline(always)]
            fn is_stopped(&self) -> bool {
                self.$tbxctl.read().mc().is_stop()
            }

            #[inline(always)]
            fn stop(&self) {
                unsafe { self.$tbxctl.clear_bits(|w| w.mc().stop()) };
            }

            #[inline(always)]
            fn set_tbidex(&self, tbidex: TimerExDiv) {
                self.$tbxex.write(|w| w.tbidex().bits(tbidex as u8));
            }

            #[inline(always)]
            fn tbifg_rd(&self) -> bool {
                self.$tbxctl.read().tbifg().bit()
            }

            #[inline(always)]
            fn tbifg_clr(&self) {
                unsafe { self.$tbxctl.clear_bits(|w| w.tbifg().clear_bit()) };
            }

            #[inline(always)]
            fn tbie_set(&self) {
                unsafe { self.$tbxctl.set_bits(|w| w.tbie().set_bit()) };
            }

            #[inline(always)]
            fn tbie_clr(&self) {
                unsafe { self.$tbxctl.clear_bits(|w| w.tbie().clear_bit()) };
            }

            #[inline]
            #[allow(unreachable_patterns)]
            fn set_ccrn(&self, ccrn: CCRn, count: u16) {
                match ccrn {
                    $(CCRn::$CCRn => self.$tbxccrn.write(|w| unsafe { w.bits(count) }),)*
                    _ => panic!()
                }
            }

            #[inline]
            #[allow(unreachable_patterns)]
            fn get_ccrn(&self, ccrn: CCRn) -> u16 {
                match ccrn {
                    $(CCRn::$CCRn => self.$tbxccrn.read().bits(),)*
                    _ => panic!()
                }
            }

            #[inline]
            #[allow(unreachable_patterns)]
            fn config_cmp_mode(&self, ccrn: CCRn, outmod: Outmod) {
                match ccrn {
                    $(CCRn::$CCRn => self.$tbxcctln.write(|w| w.outmod().bits(outmod as u8)),)*
                    _ => panic!()
                }
            }

            #[inline]
            #[allow(unreachable_patterns)]
            fn config_cap_mode(&self, ccrn: CCRn, cm: CapMode, ccis: CapSelect) {
                match ccrn {
                    $(CCRn::$CCRn => self.$tbxcctln.write(|w| w.cap().capture().scs().sync().cm().bits(cm as u8).ccis().bits(ccis as u8)),)*
                    _ => panic!()
                }
            }

            #[inline]
            #[allow(unreachable_patterns)]
            fn ccifg_rd(&self, ccrn: CCRn) -> bool {
                match ccrn {
                    $(CCRn::$CCRn => self.$tbxcctln.read().ccifg().bit(),)*
                    _ => panic!()
                }
            }

            #[inline]
            #[allow(unreachable_patterns)]
            fn ccifg_clr(&self, ccrn: CCRn) {
                match ccrn {
                    $(CCRn::$CCRn => unsafe { self.$tbxcctln.clear_bits(|w| w.ccifg().clear_bit()) },)*
                    _ => panic!()
                }
            }

            #[inline]
            #[allow(unreachable_patterns)]
            fn ccie_set(&self, ccrn: CCRn) {
                match ccrn {
                    $(CCRn::$CCRn => unsafe { self.$tbxcctln.set_bits(|w| w.ccie().set_bit()) },)*
                    _ => panic!()
                }
            }

            #[inline]
            #[allow(unreachable_patterns)]
            fn ccie_clr(&self, ccrn: CCRn) {
                match ccrn {
                    $(CCRn::$CCRn => unsafe { self.$tbxcctln.clear_bits(|w| w.ccie().clear_bit()) },)*
                    _ => panic!()
                }
            }

            #[inline]
            #[allow(unreachable_patterns)]
            fn cov_rd(&self, ccrn: CCRn) -> bool {
                match ccrn {
                    $(CCRn::$CCRn => self.$tbxcctln.read().cov().bit(),)*
                    _ => panic!()
                }
            }

            #[inline]
            #[allow(unreachable_patterns)]
            fn cov_ccifg_clr(&self, ccrn: CCRn) {
                match ccrn {
                    $(CCRn::$CCRn => unsafe {
                        self.$tbxcctln
                            .clear_bits(|w| w.ccie().clear_bit().cov().clear_bit())
                    },)*
                    _ => panic!()
                }
            }
        }
    };
}

timerb_impl!(
    TB0,
    tb0,
    tb0ctl,
    tb0ex0,
    [CCR0, tb0cctl0, tb0ccr0],
    [CCR1, tb0cctl1, tb0ccr1],
    [CCR2, tb0cctl2, tb0ccr2]
);

timerb_impl!(
    TB1,
    tb1,
    tb1ctl,
    tb1ex0,
    [CCR0, tb1cctl0, tb1ccr0],
    [CCR1, tb1cctl1, tb1ccr1],
    [CCR2, tb1cctl2, tb1ccr2]
);

timerb_impl!(
    TB2,
    tb2,
    tb2ctl,
    tb2ex0,
    [CCR0, tb2cctl0, tb2ccr0],
    [CCR1, tb2cctl1, tb2ccr1],
    [CCR2, tb2cctl2, tb2ccr2]
);

timerb_impl!(
    TB3,
    tb3,
    tb3ctl,
    tb3ex0,
    [CCR0, tb3cctl0, tb3ccr0],
    [CCR1, tb3cctl1, tb3ccr1],
    [CCR2, tb3cctl2, tb3ccr2],
    [CCR3, tb3cctl3, tb3ccr3],
    [CCR4, tb3cctl4, tb3ccr4],
    [CCR5, tb3cctl5, tb3ccr5],
    [CCR6, tb3cctl6, tb3ccr6]
);
