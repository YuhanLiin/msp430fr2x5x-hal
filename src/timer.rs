//! Timer abstraction

use crate::clock::{Aclk, Smclk};
use crate::hw_traits::timerb::{SubTimer, Tbssel, TimerB, CCR0};
use embedded_hal::timer::{Cancel, CountDown, Periodic};
use msp430fr2355 as pac;

pub use crate::hw_traits::timerb::{TimerDiv, TimerExDiv};

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

impl<T: TimerB + SubTimer<CCR0>> CountDown for Timer<T> {
    type Time = u16;

    fn start<U: Into<Self::Time>>(&mut self, count: U) {
        // 2 reads 1 write if timer is already stopped, 2 reads 2 writes if timer is not stopped
        if !self.timer.is_stopped() {
            self.timer.stop();
        }
        SubTimer::<CCR0>::set_ccrn(&self.timer, count.into());
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

impl<T: TimerB + SubTimer<CCR0>> Cancel for Timer<T> {
    type Error = void::Void;

    fn cancel(&mut self) -> Result<(), Self::Error> {
        self.timer.stop();
        Ok(())
    }
}

impl<T: TimerB + SubTimer<CCR0>> Periodic for Timer<T> {}

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
