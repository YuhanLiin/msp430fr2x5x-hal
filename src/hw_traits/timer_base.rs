// Functionality common to both TimerA and TimerB

use super::Steal;

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

pub enum Cm {
    NoCap,
    RisingEdge,
    FallingEdge,
    BothEdges,
}

pub enum Ccis {
    InputA,
    InputB,
    Gnd,
    Vcc,
}

pub trait TimerBase: Steal {
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

    /// Resume a *stopped* timer. Assumes the previous mode was 'stop'.
    /// Atomic, fast.
    fn resume(&self, mode: RunningMode);

    /// Change a timer's mode. Non-atomic, slower.
    fn change_mode(&self, mode: Mode);

    /// Set expansion register clock divider settings
    fn set_tbidex(&self, tbidex: TimerExDiv);

    fn tbifg_rd(&self) -> bool;
    fn tbifg_clr(&self);

    fn tbie_set(&self);
    fn tbie_clr(&self);

    fn tbxiv_rd(&self) -> u16;

    /// Get the current timer value.
    fn get_tbxr(&self) -> u16;
}

pub enum RunningMode {
    Up = 0b01,
    Continuous = 0b10,
    UpDown = 0b11,
}
pub enum Mode {
    Stop = 0b00,
    Up = 0b01,
    Continuous = 0b10,
    UpDown = 0b11,
}

pub trait CCRn<C>: Steal {
    fn set_ccrn(&self, count: u16);
    fn get_ccrn(&self) -> u16;

    fn config_outmod(&self, outmod: Outmod);
    fn config_cap_mode(&self, cm: Cm, ccis: Ccis);

    fn ccifg_rd(&self) -> bool;
    fn ccifg_clr(&self);

    fn ccie_set(&self);
    fn ccie_clr(&self);

    fn cov_ccifg_rd(&self) -> (bool, bool);
    fn cov_ccifg_clr(&self);
}

/// Label for capture-compare register 0
pub struct CCR0;
/// Label for capture-compare register 1
pub struct CCR1;
/// Label for capture-compare register 2
pub struct CCR2;
/// Label for capture-compare register 3
pub struct CCR3;
/// Label for capture-compare register 4
pub struct CCR4;
/// Label for capture-compare register 5
pub struct CCR5;
/// Label for capture-compare register 6
pub struct CCR6;

macro_rules! ccrn_impl {
    ($TBx:ident, $CCRn:ident, $tbxcctln:ident, $tbxccrn:ident) => {
        impl CCRn<$CCRn> for pac::$TBx {
            #[inline(always)]
            fn set_ccrn(&self, count: u16) {
                self.$tbxccrn.write(|w| unsafe { w.bits(count) });
            }

            #[inline(always)]
            fn get_ccrn(&self) -> u16 {
                self.$tbxccrn.read().bits()
            }

            #[inline(always)]
            fn config_outmod(&self, outmod: Outmod) {
                self.$tbxcctln.write(|w| w.outmod().bits(outmod as u8));
            }

            #[inline(always)]
            fn config_cap_mode(&self, cm: Cm, ccis: Ccis) {
                self.$tbxcctln.write(|w| {
                    w.cap()
                        .capture()
                        .scs()
                        .sync()
                        .cm()
                        .bits(cm as u8)
                        .ccis()
                        .bits(ccis as u8)
                });
            }

            #[inline(always)]
            fn ccifg_rd(&self) -> bool {
                self.$tbxcctln.read().ccifg().bit()
            }

            #[inline(always)]
            fn ccifg_clr(&self) {
                unsafe { self.$tbxcctln.clear_bits(|w| w.ccifg().clear_bit()) };
            }

            #[inline(always)]
            fn ccie_set(&self) {
                unsafe { self.$tbxcctln.set_bits(|w| w.ccie().set_bit()) };
            }

            #[inline(always)]
            fn ccie_clr(&self) {
                unsafe { self.$tbxcctln.clear_bits(|w| w.ccie().clear_bit()) };
            }

            #[inline(always)]
            fn cov_ccifg_rd(&self) -> (bool, bool) {
                let cctl = self.$tbxcctln.read();
                (cctl.cov().bit(), cctl.ccifg().bit())
            }

            #[inline(always)]
            fn cov_ccifg_clr(&self) {
                unsafe {
                    self.$tbxcctln
                        .clear_bits(|w| w.ccifg().clear_bit().cov().clear_bit())
                };
            }
        }
    };
}
pub(crate) use ccrn_impl;

macro_rules! timer_base_impl {
    ($TBx:ident, $tbx:ident, $tbxctl:ident, $tbxex:ident, $tbxiv:ident, $tbxr:ident, $([$CCRn:ident, $tbxcctln:ident, $tbxccrn:ident]),*) => {
        impl Steal for pac::$TBx {
            #[inline(always)]
            unsafe fn steal() -> Self {
                pac::Peripherals::conjure().$TBx
            }
        }

        impl TimerBase for pac::$TBx {
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
            fn change_mode(&self, mode: Mode) {
                self.$tbxctl.modify(|_,w| w.mc().bits(mode as u8))
            }

            #[inline(always)]
            fn resume(&self, mode: RunningMode) {
                unsafe { self.$tbxctl.set_bits(|w| w.mc().bits(mode as u8)) };
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

            #[inline(always)]
            fn tbxiv_rd(&self) -> u16 {
                self.$tbxiv.read().bits()
            }

            #[inline(always)]
            fn get_tbxr(&self) -> u16 {
                self.$tbxr.read().bits()
            }
        }

        $(ccrn_impl!($TBx, $CCRn, $tbxcctln, $tbxccrn);)*
    };
}
pub(crate) use timer_base_impl;
