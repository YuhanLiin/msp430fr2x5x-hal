//! Timer abstraction

use crate::clock::{Aclk, Smclk};
use crate::hw_traits::timerb::{
    SubTimerB, Tbssel, TimerB, CCR0, CCR1, CCR2, CCR3, CCR4, CCR5, CCR6,
};
use embedded_hal::timer::{Cancel, CountDown, Periodic};
use msp430fr2355 as pac;

pub use crate::hw_traits::timerb::{TimerDiv, TimerExDiv};

/// Marker trait for timers with 3 CC registers
pub trait ThreeCapCmpTimer: TimerB + SubTimerB<CCR0> + SubTimerB<CCR1> + SubTimerB<CCR2> {}

/// Marker trait for timers with 7 CC registers
pub trait SevenCapCmpTimer:
    TimerB
    + SubTimerB<CCR0>
    + SubTimerB<CCR1>
    + SubTimerB<CCR2>
    + SubTimerB<CCR3>
    + SubTimerB<CCR4>
    + SubTimerB<CCR5>
    + SubTimerB<CCR6>
{
}

impl ThreeCapCmpTimer for pac::TB0 {}
impl ThreeCapCmpTimer for pac::TB1 {}
impl ThreeCapCmpTimer for pac::TB2 {}
impl SevenCapCmpTimer for pac::TB3 {}

/// Configures all HAL objects that use the TimerB timers
pub struct TimerConfig {
    sel: Tbssel,
    div: TimerDiv,
    ex_div: TimerExDiv,
}

impl TimerConfig {
    /// Configure timer clock source to ACLK
    pub fn aclk(_aclk: &Aclk) -> Self {
        TimerConfig {
            sel: Tbssel::Aclk,
            div: TimerDiv::_1,
            ex_div: TimerExDiv::_1,
        }
    }

    /// Configure timer clock source to SMCLK
    pub fn smclk(_smclk: &Smclk) -> Self {
        TimerConfig {
            sel: Tbssel::Aclk,
            div: TimerDiv::_1,
            ex_div: TimerExDiv::_1,
        }
    }

    /// Configure timer clock source to INCLK
    pub fn inclk() -> Self {
        TimerConfig {
            sel: Tbssel::Inclk,
            div: TimerDiv::_1,
            ex_div: TimerExDiv::_1,
        }
    }

    /// Configure timer clock source to TBCLK
    pub fn tbclk() -> Self {
        TimerConfig {
            sel: Tbssel::Tbxclk,
            div: TimerDiv::_1,
            ex_div: TimerExDiv::_1,
        }
    }

    /// Configure the normal clock divider and expansion clock divider settings
    pub fn clk_div(self, div: TimerDiv, ex_div: TimerExDiv) -> Self {
        TimerConfig {
            sel: self.sel,
            div,
            ex_div,
        }
    }

    fn write_regs<T: TimerB>(self, timer: &T) {
        timer.reset();
        timer.set_tbidex(self.ex_div);
        timer.config_clock(self.sel, self.div);
    }
}

/// Periodic countdown timer
pub struct Timer<T: TimerB> {
    timer: T,
}

/// Extension trait for creating timers
pub trait TimerExt {
    #[doc(hidden)]
    type Timer;

    /// Create new timer out of peripheral
    fn to_timer(self, config: TimerConfig) -> Self::Timer;
}

impl<T: TimerB> TimerExt for T {
    type Timer = Timer<T>;

    fn to_timer(self, config: TimerConfig) -> Self::Timer {
        config.write_regs(&self);
        Timer { timer: self }
    }
}

impl<T: TimerB + SubTimerB<CCR0>> CountDown for Timer<T> {
    type Time = u16;

    fn start<U: Into<Self::Time>>(&mut self, count: U) {
        // 2 reads 1 write if timer is already stopped, 2 reads 2 writes if timer is not stopped
        if !self.timer.is_stopped() {
            self.timer.stop();
        }
        SubTimerB::<CCR0>::set_ccrn(&self.timer, count.into());
        self.timer.upmode();
    }

    fn wait(&mut self) -> nb::Result<(), void::Void> {
        if self.timer.tbifg_rd() {
            self.timer.tbifg_clr();
            Ok(())
        } else {
            Err(nb::Error::WouldBlock)
        }
    }
}

impl<T: TimerB + SubTimerB<CCR0>> Cancel for Timer<T> {
    type Error = void::Void;

    fn cancel(&mut self) -> Result<(), Self::Error> {
        self.timer.stop();
        Ok(())
    }
}

impl<T: TimerB + SubTimerB<CCR0>> Periodic for Timer<T> {}

impl<T: TimerB> Timer<T> {
    /// Enable timer countdown expiration interrupts
    pub fn enable_interrupts(&mut self) {
        self.timer.tbie_set();
    }

    /// Disable timer countdown expiration interrupts
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
pub enum TimerTwoChannel {
    Chan1,
    Chan2,
}

/// Sub-timer channel enumeration for the 6-channel timers (7 capture-control registers)
pub enum TimerSixChannel {
    Chan1,
    Chan2,
    Chan3,
    Chan4,
    Chan5,
    Chan6,
}

impl<T: ThreeCapCmpTimer> SubTimer for Timer<T> {
    type Channel = TimerTwoChannel;

    fn set_subtimer_count(&mut self, chan: Self::Channel, count: u16) {
        match chan {
            TimerTwoChannel::Chan1 => {
                SubTimerB::<CCR1>::set_ccrn(&self.timer, count);
                SubTimerB::<CCR1>::ccifg_clr(&self.timer);
            },
            TimerTwoChannel::Chan2 => {
                SubTimerB::<CCR2>::set_ccrn(&self.timer, count);
                SubTimerB::<CCR2>::ccifg_clr(&self.timer);
            },
        }
    }

    fn wait_subtimer(&mut self, chan: Self::Channel) -> nb::Result<(), void::Void>;

    fn enable_subtimer_interrupts(&mut self, chan: Self::Channel);

    fn disable_subtimer_interrupts(&mut self, chan: Self::Channel);
}
