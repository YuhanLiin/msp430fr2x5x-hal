//! Timers
//!
//! Each timer has a configurable clock source and clock dividers. In addition, each timer
//! peripheral provides their own "sub-timers". Each sub-timer has its own configurable threshold
//! and will set its own IFG when the main timer counts to its threshold.

use crate::clock::{Aclk, Smclk};
use crate::gpio::{Alternate1, Floating, Input, Pin, Pin2, Pin6, Pin7, P2, P5, P6};
use crate::hw_traits::timerb::{CCRn, Tbssel, TimerB, TimerSteal};
use crate::util::SealedDefault;
use core::marker::PhantomData;
use embedded_hal::timer::{Cancel, CountDown, Periodic};
use msp430fr2355 as pac;

pub use crate::hw_traits::timerb::{
    TimerDiv, TimerExDiv, CCR0, CCR1, CCR2, CCR3, CCR4, CCR5, CCR6,
};

// Trait effectively sealed by CCRn
/// Trait indicating that the peripheral can be used as a sub-timer, PWM, or capture
pub trait CapCmpPeriph<C>: CCRn<C> + CCRn<CCR0> {}
impl<T: CCRn<C> + CCRn<CCR0>, C> CapCmpPeriph<C> for T {}

// Trait effectively sealed by CCRn
/// Trait indicating that the peripheral can be used as a timer
pub trait TimerPeriph: TimerB + CCRn<CCR0> {
    /// Pin type used for external TBxCLK of this timer
    type Tbxclk;
}

// Traits effectively sealed by CCRn
/// Trait indicating that the peripheral has 3 capture compare registers
pub trait ThreeCCRnTimer: TimerPeriph + CCRn<CCR1> + CCRn<CCR2> {}
/// Trait indicating that the peripheral has 7 capture compare registers
pub trait SevenCCRnTimer:
    TimerPeriph + CCRn<CCR1> + CCRn<CCR2> + CCRn<CCR3> + CCRn<CCR4> + CCRn<CCR5> + CCRn<CCR6>
{
}

impl TimerPeriph for pac::tb0::RegisterBlock {
    type Tbxclk = Pin<P2, Pin7, Alternate1<Input<Floating>>>;
}
impl ThreeCCRnTimer for pac::tb0::RegisterBlock {}

impl TimerPeriph for pac::tb1::RegisterBlock {
    type Tbxclk = Pin<P2, Pin2, Alternate1<Input<Floating>>>;
}
impl ThreeCCRnTimer for pac::tb1::RegisterBlock {}

impl TimerPeriph for pac::tb2::RegisterBlock {
    type Tbxclk = Pin<P5, Pin2, Alternate1<Input<Floating>>>;
}
impl ThreeCCRnTimer for pac::tb2::RegisterBlock {}

impl TimerPeriph for pac::tb3::RegisterBlock {
    type Tbxclk = Pin<P6, Pin6, Alternate1<Input<Floating>>>;
}
impl SevenCCRnTimer for pac::tb3::RegisterBlock {}

/// Configures all HAL objects that use the TimerB timers
pub struct TimerConfig<T: TimerPeriph> {
    _timer: PhantomData<T>,
    sel: Tbssel,
    div: TimerDiv,
    ex_div: TimerExDiv,
}

impl<T: TimerPeriph> TimerConfig<T> {
    /// Configure timer clock source to ACLK
    #[inline]
    pub fn aclk(_aclk: &Aclk) -> Self {
        TimerConfig {
            _timer: PhantomData,
            sel: Tbssel::Aclk,
            div: TimerDiv::_1,
            ex_div: TimerExDiv::_1,
        }
    }

    /// Configure timer clock source to SMCLK
    #[inline]
    pub fn smclk(_smclk: &Smclk) -> Self {
        TimerConfig {
            _timer: PhantomData,
            sel: Tbssel::Smclk,
            div: TimerDiv::_1,
            ex_div: TimerExDiv::_1,
        }
    }

    /// Configure timer clock source to TBCLK
    #[inline]
    pub fn tbclk(_pin: T::Tbxclk) -> Self {
        TimerConfig {
            _timer: PhantomData,
            sel: Tbssel::Tbxclk,
            div: TimerDiv::_1,
            ex_div: TimerExDiv::_1,
        }
    }

    /// Configure the normal clock divider and expansion clock divider settings
    #[inline]
    pub fn clk_div(self, div: TimerDiv, ex_div: TimerExDiv) -> Self {
        TimerConfig {
            _timer: PhantomData,
            sel: self.sel,
            div,
            ex_div,
        }
    }

    #[inline]
    pub(crate) fn write_regs(self, timer: &T) {
        timer.reset();
        timer.set_tbidex(self.ex_div);
        timer.config_clock(self.sel, self.div);
    }
}

/// Main timer and sub-timers for timer peripherals with 3 capture-compare registers
pub struct ThreeCCRnParts<T: ThreeCCRnTimer> {
    /// Main timer
    pub timer: Timer<T>,
    /// Timer interrupt vector
    pub tbxiv: TBxIV<T>,
    /// Sub-timer 1 (derived from CCR1 register)
    pub subtimer1: SubTimer<T, CCR1>,
    /// Sub-timer 2 (derived from CCR2 register)
    pub subtimer2: SubTimer<T, CCR2>,
}

impl<T: ThreeCCRnTimer> SealedDefault for ThreeCCRnParts<T> {
    fn default() -> Self {
        Self {
            timer: SealedDefault::default(),
            tbxiv: TBxIV(PhantomData),
            subtimer1: SealedDefault::default(),
            subtimer2: SealedDefault::default(),
        }
    }
}

/// Main timer and sub-timers for timer peripherals with 7 capture-compare registers
pub struct SevenCCRnParts<T: SevenCCRnTimer> {
    /// Main timer
    pub timer: Timer<T>,
    /// Timer interrupt vector
    pub tbxiv: TBxIV<T>,
    /// Sub-timer 1 (derived from CCR1 register)
    pub subtimer1: SubTimer<T, CCR1>,
    /// Sub-timer 2 (derived from CCR2 register)
    pub subtimer2: SubTimer<T, CCR2>,
    /// Sub-timer 3 (derived from CCR3 register)
    pub subtimer3: SubTimer<T, CCR3>,
    /// Sub-timer 4 (derived from CCR4 register)
    pub subtimer4: SubTimer<T, CCR4>,
    /// Sub-timer 5 (derived from CCR5 register)
    pub subtimer5: SubTimer<T, CCR5>,
    /// Sub-timer 6 (derived from CCR6 register)
    pub subtimer6: SubTimer<T, CCR6>,
}

impl<T: SevenCCRnTimer> SealedDefault for SevenCCRnParts<T> {
    fn default() -> Self {
        Self {
            timer: SealedDefault::default(),
            tbxiv: TBxIV(PhantomData),
            subtimer1: SealedDefault::default(),
            subtimer2: SealedDefault::default(),
            subtimer3: SealedDefault::default(),
            subtimer4: SealedDefault::default(),
            subtimer5: SealedDefault::default(),
            subtimer6: SealedDefault::default(),
        }
    }
}

/// Periodic countdown timer
pub struct Timer<T: TimerPeriph>(PhantomData<T>);

impl<T: TimerPeriph> SealedDefault for Timer<T> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

/// Sub-timer with its own interrupts and threshold that shares its countdown with the main timer
pub struct SubTimer<T: CCRn<C>, C>(PhantomData<T>, PhantomData<C>);

impl<T: CapCmpPeriph<C>, C> SealedDefault for SubTimer<T, C> {
    fn default() -> Self {
        Self(PhantomData, PhantomData)
    }
}

mod sealed {
    use super::*;

    pub trait SealedTimerExt {}

    impl SealedTimerExt for pac::TB0 {}
    impl SealedTimerExt for pac::TB1 {}
    impl SealedTimerExt for pac::TB2 {}
    impl SealedTimerExt for pac::TB3 {}
}

/// Extension trait for creating timers
pub trait TimerExt: Sized + sealed::SealedTimerExt {
    /// Set of timers
    type Parts: SealedDefault;
    #[doc(hidden)]
    type Timer: TimerPeriph;

    /// Create new set of timers out of a TBx peripheral
    #[inline(always)]
    fn to_timer(self, config: TimerConfig<Self::Timer>) -> Self::Parts {
        config.write_regs(unsafe { Self::Timer::steal() });
        Self::Parts::default()
    }
}

impl TimerExt for pac::TB0 {
    type Parts = ThreeCCRnParts<Self::Timer>;
    type Timer = pac::tb0::RegisterBlock;
}

impl TimerExt for pac::TB1 {
    type Parts = ThreeCCRnParts<Self::Timer>;
    type Timer = pac::tb1::RegisterBlock;
}

impl TimerExt for pac::TB2 {
    type Parts = ThreeCCRnParts<Self::Timer>;
    type Timer = pac::tb2::RegisterBlock;
}

impl TimerExt for pac::TB3 {
    type Parts = SevenCCRnParts<Self::Timer>;
    type Timer = pac::tb3::RegisterBlock;
}

/// Timer TBIV interrupt vector
pub enum TimerVector {
    /// No pending interrupt
    NoInterrupt,
    /// Interrupt caused by sub-timer 1
    SubTimer1,
    /// Interrupt caused by sub-timer 2
    SubTimer2,
    /// Interrupt caused by sub-timer 3
    SubTimer3,
    /// Interrupt caused by sub-timer 4
    SubTimer4,
    /// Interrupt caused by sub-timer 5
    SubTimer5,
    /// Interrupt caused by sub-timer 6
    SubTimer6,
    /// Interrupt caused by main timer overflow
    MainTimer,
}

#[inline]
pub(crate) fn read_tbxiv<T: TimerB>(timer: &T) -> TimerVector {
    match timer.tbxiv_rd() {
        0 => TimerVector::NoInterrupt,
        2 => TimerVector::SubTimer1,
        4 => TimerVector::SubTimer2,
        6 => TimerVector::SubTimer3,
        8 => TimerVector::SubTimer4,
        10 => TimerVector::SubTimer5,
        12 => TimerVector::SubTimer6,
        14 => TimerVector::MainTimer,
        _ => unsafe { core::hint::unreachable_unchecked() },
    }
}

/// Interrupt vector register for determining which timer caused an ISR
pub struct TBxIV<T>(PhantomData<T>);

impl<T: TimerB> TBxIV<T> {
    #[inline]
    /// Read the timer interrupt vector. Automatically resets corresponding interrupt flag.
    pub fn interrupt_vector(&mut self) -> TimerVector {
        let timer = unsafe { T::steal() };
        read_tbxiv(timer)
    }
}

impl<T: TimerPeriph> CountDown for Timer<T> {
    type Time = u16;

    #[inline]
    fn start<U: Into<Self::Time>>(&mut self, count: U) {
        let timer = unsafe { T::steal() };
        timer.stop();
        timer.set_ccrn(count.into());
        timer.upmode();
    }

    #[inline]
    fn wait(&mut self) -> nb::Result<(), void::Void> {
        let timer = unsafe { T::steal() };
        if timer.tbifg_rd() {
            timer.tbifg_clr();
            Ok(())
        } else {
            Err(nb::Error::WouldBlock)
        }
    }
}

impl<T: TimerPeriph> Cancel for Timer<T> {
    type Error = void::Void;

    #[inline(always)]
    fn cancel(&mut self) -> Result<(), Self::Error> {
        let timer = unsafe { T::steal() };
        timer.stop();
        Ok(())
    }
}

impl<T: TimerPeriph> Periodic for Timer<T> {}

impl<T: TimerPeriph> Timer<T> {
    /// Enable timer countdown expiration interrupts
    #[inline(always)]
    pub fn enable_interrupts(&mut self) {
        let timer = unsafe { T::steal() };
        timer.tbie_set();
    }

    /// Disable timer countdown expiration interrupts
    #[inline(always)]
    pub fn disable_interrupts(&mut self) {
        let timer = unsafe { T::steal() };
        timer.tbie_clr();
    }
}

impl<T: CCRn<C>, C> SubTimer<T, C> {
    #[inline]
    /// Set the threshold for one of the subtimers. Once the main timer counts to this threshold
    /// the subtimer will fire. Note that the main timer resets once it counts to its own
    /// threshold, not the sub-timer thresholds. It follows that the sub-timer threshold must be
    /// less than the main threshold for it to fire.
    pub fn set_count(&mut self, count: u16) {
        let timer = unsafe { T::steal() };
        timer.set_ccrn(count);
        timer.ccifg_clr();
    }

    #[inline]
    /// Wait for the subtimer to fire
    pub fn wait(&mut self) -> nb::Result<(), void::Void> {
        let timer = unsafe { T::steal() };
        if timer.ccifg_rd() {
            timer.ccifg_clr();
            Ok(())
        } else {
            Err(nb::Error::WouldBlock)
        }
    }

    #[inline(always)]
    /// Enable the subtimer interrupts, which fire once the subtimer fires.
    pub fn enable_interrupts(&mut self) {
        let timer = unsafe { T::steal() };
        timer.ccie_set();
    }

    #[inline(always)]
    /// Disable the subtimer interrupts, which fire once the subtimer fires.
    pub fn disable_interrupts(&mut self) {
        let timer = unsafe { T::steal() };
        timer.ccie_clr();
    }
}
