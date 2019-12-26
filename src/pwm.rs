//! PWM ports
//!
//! PWM ports are created from timers. TB0, TB1, and TB2 create 2-channel ports and TB3 create
//! 6-channel ports. Each channel has its own configurable duty cycle, but share the same period as
//! other channels in the same port.

use crate::hw_traits::timerb::{CCRn, Outmod, TimerB};
use crate::timer::{read_tbxiv, TimerClkPin};
use embedded_hal::Pwm;
use msp430fr2355 as pac;

pub use crate::timer::{TimerConfig, TimerDiv, TimerExDiv, TimerVector};

/// Trait indicating that the peripheral can be used as a PWM
pub trait PwmPeriph: TimerClkPin {
    #[doc(hidden)]
    type Channel: Into<CCRn>;

    #[doc(hidden)]
    // Configure each CCRn register required
    fn config_channels(&self);
}

impl PwmPeriph for pac::TB0 {
    type Channel = PwmTwoChannel;

    #[inline]
    fn config_channels(&self) {
        self.config_outmod(CCRn::CCR1, Outmod::ResetSet);
        self.config_outmod(CCRn::CCR2, Outmod::ResetSet);
    }
}

impl PwmPeriph for pac::TB1 {
    type Channel = PwmTwoChannel;

    #[inline]
    fn config_channels(&self) {
        self.config_outmod(CCRn::CCR1, Outmod::ResetSet);
        self.config_outmod(CCRn::CCR2, Outmod::ResetSet);
    }
}

impl PwmPeriph for pac::TB2 {
    type Channel = PwmTwoChannel;

    #[inline]
    fn config_channels(&self) {
        self.config_outmod(CCRn::CCR1, Outmod::ResetSet);
        self.config_outmod(CCRn::CCR2, Outmod::ResetSet);
    }
}

impl PwmPeriph for pac::TB3 {
    type Channel = PwmSixChannel;

    #[inline]
    fn config_channels(&self) {
        self.config_outmod(CCRn::CCR1, Outmod::ResetSet);
        self.config_outmod(CCRn::CCR2, Outmod::ResetSet);
        self.config_outmod(CCRn::CCR3, Outmod::ResetSet);
        self.config_outmod(CCRn::CCR4, Outmod::ResetSet);
        self.config_outmod(CCRn::CCR5, Outmod::ResetSet);
        self.config_outmod(CCRn::CCR6, Outmod::ResetSet);
    }
}

/// PWM channel for 2-channel PWM ports
#[derive(Clone, Copy)]
pub enum PwmTwoChannel {
    /// Channel 1
    Chan1,
    /// Channel 2
    Chan2,
}

impl Into<CCRn> for PwmTwoChannel {
    #[inline]
    fn into(self) -> CCRn {
        match self {
            PwmTwoChannel::Chan1 => CCRn::CCR1,
            PwmTwoChannel::Chan2 => CCRn::CCR2,
        }
    }
}

/// PWM channel for 6-channel PWM ports
#[derive(Clone, Copy)]
pub enum PwmSixChannel {
    /// Channel 1
    Chan1,
    /// Channel 2
    Chan2,
    /// Channel 3
    Chan3,
    /// Channel 4
    Chan4,
    /// Channel 5
    Chan5,
    /// Channel 6
    Chan6,
}

impl Into<CCRn> for PwmSixChannel {
    #[inline]
    fn into(self) -> CCRn {
        match self {
            PwmSixChannel::Chan1 => CCRn::CCR1,
            PwmSixChannel::Chan2 => CCRn::CCR2,
            PwmSixChannel::Chan3 => CCRn::CCR3,
            PwmSixChannel::Chan4 => CCRn::CCR4,
            PwmSixChannel::Chan5 => CCRn::CCR5,
            PwmSixChannel::Chan6 => CCRn::CCR6,
        }
    }
}

/// PWM port with multiple channels and a single period
pub struct PwmPort<T: PwmPeriph> {
    timer: T,
}

/// Extension trait for creating PWM ports from timer peripherals
pub trait PwmExt: Sized + TimerClkPin {
    #[doc(hidden)]
    type Pwm;

    /// Create new PWM port out of timer
    fn to_pwm(self, config: TimerConfig<Self>) -> Self::Pwm;
}

impl<T: PwmPeriph> PwmExt for T {
    type Pwm = PwmPort<T>;

    #[inline]
    fn to_pwm(self, config: TimerConfig<Self>) -> Self::Pwm {
        config.write_regs(&self);
        self.config_outmod(CCRn::CCR0, Outmod::Toggle);
        self.config_channels();
        // Start the timer to run PWM
        self.upmode();
        PwmPort { timer: self }
    }
}

impl<T: PwmPeriph> PwmPort<T> {
    #[inline(always)]
    fn start_all(&mut self) {
        self.timer.upmode();
    }

    #[inline(always)]
    fn pause_all(&mut self) {
        self.timer.stop();
    }

    #[inline]
    /// Read the timer interrupt vector. Automatically resets corresponding interrupt flag.
    pub fn interrupt_vector(&mut self) -> TimerVector {
        read_tbxiv(&self.timer)
    }
}

impl<T: PwmPeriph> Pwm for PwmPort<T> {
    type Channel = T::Channel;
    /// Number of cycles.
    type Time = u16;
    /// Number of cycles. Does not depend on the period.
    type Duty = u16;

    #[inline(always)]
    fn set_period<P: Into<Self::Time>>(&mut self, period: P) {
        self.timer.set_ccrn(CCRn::CCR0, period.into());
    }

    #[inline(always)]
    fn set_duty(&mut self, channel: Self::Channel, duty: Self::Duty) {
        self.timer.set_ccrn(channel.into(), duty);
    }

    #[inline(always)]
    fn get_period(&self) -> Self::Time {
        self.timer.get_ccrn(CCRn::CCR0)
    }

    #[inline(always)]
    fn get_duty(&self, channel: Self::Channel) -> Self::Duty {
        self.timer.get_ccrn(channel.into())
    }

    /// Maximum valid duty is equal to the period. If number of duty cycles exceeds number of
    /// period cycles, then signal stays high (equivalent to 100% duty cycle).
    #[inline(always)]
    fn get_max_duty(&self) -> Self::Duty {
        self.get_period()
    }

    // Less efficient than disable_all
    #[inline]
    fn disable(&mut self, channel: Self::Channel) {
        self.pause_all();
        // Forces the channel to always output low signal
        self.timer.config_outmod(channel.into(), Outmod::Out);
        self.start_all();
    }

    // Less efficient than disable_all
    #[inline]
    fn enable(&mut self, channel: Self::Channel) {
        self.pause_all();
        // Make channel work the same as normal PWM
        self.timer.config_outmod(channel.into(), Outmod::ResetSet);
        self.start_all();
    }
}
