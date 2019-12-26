//! Timer abstraction

use crate::clock::{Aclk, Smclk};
use crate::gpio::{Alternate1, Floating, Input, Pin, Pin2, Pin6, Pin7, Port2, Port5, Port6};
use crate::hw_traits::timerb::{CCRn, Tbssel, TimerB};
use core::marker::PhantomData;
use embedded_hal::timer::{Cancel, CountDown, Periodic};
use msp430fr2355 as pac;

pub use crate::hw_traits::timerb::{TimerDiv, TimerExDiv};

pub(crate) trait TimerClkPin: TimerB {
    // Pin type used for external TBxCLK of this timer
    type Tbxclk;
}

/// Trait indicating that the peripheral can be used as a timer
pub trait TimerPeriph: TimerClkPin {
    #[doc(hidden)]
    type Channel: Into<CCRn> + Copy;
}

impl TimerClkPin for pac::TB0 {
    type Tbxclk = Pin<Port2, Pin7, Alternate1<Input<Floating>>>;
}
impl TimerPeriph for pac::TB0 {
    type Channel = TimerTwoChannel;
}

impl TimerClkPin for pac::TB1 {
    type Tbxclk = Pin<Port2, Pin2, Alternate1<Input<Floating>>>;
}
impl TimerPeriph for pac::TB1 {
    type Channel = TimerTwoChannel;
}

impl TimerClkPin for pac::TB2 {
    type Tbxclk = Pin<Port5, Pin2, Alternate1<Input<Floating>>>;
}
impl TimerPeriph for pac::TB2 {
    type Channel = TimerTwoChannel;
}

impl TimerClkPin for pac::TB3 {
    type Tbxclk = Pin<Port6, Pin6, Alternate1<Input<Floating>>>;
}
impl TimerPeriph for pac::TB3 {
    type Channel = TimerSixChannel;
}

/// Configures all HAL objects that use the TimerB timers
pub struct TimerConfig<T: TimerClkPin> {
    _timer: PhantomData<T>,
    sel: Tbssel,
    div: TimerDiv,
    ex_div: TimerExDiv,
}

impl<T: TimerClkPin> TimerConfig<T> {
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

/// Periodic countdown timer
pub struct Timer<T: TimerPeriph> {
    timer: T,
}

/// Extension trait for creating timers
pub trait TimerExt {
    #[doc(hidden)]
    type Timer;

    /// Create new timer out of peripheral
    fn to_timer(self, config: TimerConfig<Self>) -> Self::Timer;
}

impl<T: TimerPeriph> TimerExt for T {
    type Timer = Timer<T>;

    #[inline(always)]
    fn to_timer(self, config: TimerConfig<Self>) -> Self::Timer {
        config.write_regs(&self);
        Timer { timer: self }
    }
}

impl<T: TimerPeriph> CountDown for Timer<T> {
    type Time = u16;

    #[inline]
    fn start<U: Into<Self::Time>>(&mut self, count: U) {
        // 2 reads 1 write if timer is already stopped, 2 reads 2 writes if timer is not stopped
        if !self.timer.is_stopped() {
            self.timer.stop();
        }
        self.timer.set_ccrn(CCRn::CCR0, count.into());
        self.timer.upmode();
    }

    #[inline]
    fn wait(&mut self) -> nb::Result<(), void::Void> {
        if self.timer.tbifg_rd() {
            self.timer.tbifg_clr();
            Ok(())
        } else {
            Err(nb::Error::WouldBlock)
        }
    }
}

impl<T: TimerPeriph> Cancel for Timer<T> {
    type Error = void::Void;

    #[inline]
    fn cancel(&mut self) -> Result<(), Self::Error> {
        self.timer.stop();
        Ok(())
    }
}

impl<T: TimerPeriph> Periodic for Timer<T> {}

impl<T: TimerPeriph> Timer<T> {
    /// Enable timer countdown expiration interrupts
    #[inline]
    pub fn enable_interrupts(&mut self) {
        self.timer.tbie_set();
    }

    /// Disable timer countdown expiration interrupts
    #[inline]
    pub fn disable_interrupts(&mut self) {
        self.timer.tbie_clr();
    }
}

/// A timer with sub-timers that have their own interrupts and thresholds
pub trait SubTimer {
    /// Enumeration of available sub-timers
    type Channel;

    /// Set the threshold for one of the subtimers. Once the main timer counts to this threshold
    /// the subtimer will fire. Note that the main timer resets once it counts to its own
    /// threshold, not the sub-timer thresholds. It follows that the sub-timer threshold must be
    /// less than the main threshold for it to fire.
    fn set_subtimer_count(&mut self, chan: Self::Channel, count: u16);

    /// Wait for the subtimer to fire
    fn wait_subtimer(&mut self, chan: Self::Channel) -> nb::Result<(), void::Void>;

    /// Enable the subtimer interrupts, which fire once the subtimer fires.
    fn enable_subtimer_interrupts(&mut self, chan: Self::Channel);

    /// Disable the subtimer interrupts, which fire once the subtimer fires.
    fn disable_subtimer_interrupts(&mut self, chan: Self::Channel);
}

/// Sub-timer channel enumeration for the 2-channel timers (3 capture-control registers)
#[derive(Clone, Copy)]
pub enum TimerTwoChannel {
    /// Sub-timer 1
    Chan1,
    /// Sub-timer 2
    Chan2,
}

impl Into<CCRn> for TimerTwoChannel {
    #[inline]
    fn into(self) -> CCRn {
        match self {
            TimerTwoChannel::Chan1 => CCRn::CCR1,
            TimerTwoChannel::Chan2 => CCRn::CCR2,
        }
    }
}

/// Sub-timer channel enumeration for the 6-channel timers (7 capture-control registers)
#[derive(Clone, Copy)]
pub enum TimerSixChannel {
    /// Sub-timer 1
    Chan1,
    /// Sub-timer 2
    Chan2,
    /// Sub-timer 3
    Chan3,
    /// Sub-timer 4
    Chan4,
    /// Sub-timer 5
    Chan5,
    /// Sub-timer 6
    Chan6,
}

impl Into<CCRn> for TimerSixChannel {
    #[inline]
    fn into(self) -> CCRn {
        match self {
            TimerSixChannel::Chan1 => CCRn::CCR1,
            TimerSixChannel::Chan2 => CCRn::CCR2,
            TimerSixChannel::Chan3 => CCRn::CCR3,
            TimerSixChannel::Chan4 => CCRn::CCR4,
            TimerSixChannel::Chan5 => CCRn::CCR5,
            TimerSixChannel::Chan6 => CCRn::CCR6,
        }
    }
}

impl<T: TimerPeriph> SubTimer for Timer<T> {
    type Channel = T::Channel;

    #[inline]
    fn set_subtimer_count(&mut self, chan: Self::Channel, count: u16) {
        self.timer.set_ccrn(chan.into(), count);
        self.timer.ccifg_clr(chan.into());
    }

    #[inline]
    fn wait_subtimer(&mut self, chan: Self::Channel) -> nb::Result<(), void::Void> {
        if self.timer.ccifg_rd(chan.into()) {
            self.timer.ccifg_clr(chan.into());
            Ok(())
        } else {
            Err(nb::Error::WouldBlock)
        }
    }

    #[inline]
    fn enable_subtimer_interrupts(&mut self, chan: Self::Channel) {
        self.timer.ccie_set(chan.into());
    }

    #[inline]
    fn disable_subtimer_interrupts(&mut self, chan: Self::Channel) {
        self.timer.ccie_clr(chan.into());
    }
}
