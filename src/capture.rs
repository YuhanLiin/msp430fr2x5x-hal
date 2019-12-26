//! Capture ports
//!
//! Signal capture ports are created from timers. TB0, TB1, and TB2 create 3-channel ports and TB3
//! creates a 7-channel port. Each capture channel has its own configurable input signal, edge
//! trigger, and capture storage. Each port has a configurable timer counting from 0 to `2^16-1`,
//! whose value will be stored whenever a channel capture event is triggered.

use crate::hw_traits::timerb::CCRn;
use crate::timer::{read_tbxiv, TimerClkPin};
use embedded_hal::Capture;
use msp430fr2355 as pac;

pub use crate::hw_traits::timerb::{CapSelect, CapTrigger};
pub use crate::timer::{TimerConfig, TimerDiv, TimerExDiv, TimerVector};

/// Capture channel for 3-channel capture ports
#[derive(Clone, Copy)]
pub enum CapThreeChannel {
    /// Channel 0
    Chan0,
    /// Channel 1
    Chan1,
    /// Channel 2
    Chan2,
}

impl Into<CCRn> for CapThreeChannel {
    #[inline]
    fn into(self) -> CCRn {
        match self {
            CapThreeChannel::Chan0 => CCRn::CCR0,
            CapThreeChannel::Chan1 => CCRn::CCR1,
            CapThreeChannel::Chan2 => CCRn::CCR2,
        }
    }
}

/// Capture channel for 7-channel capture ports
#[derive(Clone, Copy)]
pub enum CapSevenChannel {
    /// Channel 0
    Chan0,
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

impl Into<CCRn> for CapSevenChannel {
    #[inline]
    fn into(self) -> CCRn {
        match self {
            CapSevenChannel::Chan0 => CCRn::CCR0,
            CapSevenChannel::Chan1 => CCRn::CCR1,
            CapSevenChannel::Chan2 => CCRn::CCR2,
            CapSevenChannel::Chan3 => CCRn::CCR3,
            CapSevenChannel::Chan4 => CCRn::CCR4,
            CapSevenChannel::Chan5 => CCRn::CCR5,
            CapSevenChannel::Chan6 => CCRn::CCR6,
        }
    }
}

/// Trait indicating that a peripheral can be used as a capture port
pub trait CapturePeriph: TimerClkPin {
    #[doc(hidden)]
    type Channel: Into<CCRn> + Copy;
}

impl CapturePeriph for pac::TB0 {
    type Channel = CapThreeChannel;
}

impl CapturePeriph for pac::TB1 {
    type Channel = CapThreeChannel;
}

impl CapturePeriph for pac::TB2 {
    type Channel = CapThreeChannel;
}

impl CapturePeriph for pac::TB3 {
    type Channel = CapSevenChannel;
}

/// Capture port with multiple channels. When a channel's input triggers a capture, the current
/// 16-bit timer value is recorded.
pub struct CapturePort<T: CapturePeriph> {
    timer: T,
}

impl<T: CapturePeriph> CapturePort<T> {
    #[inline(always)]
    fn start_all(&mut self) {
        self.timer.continuous();
    }

    #[inline(always)]
    fn pause_all(&mut self) {
        self.timer.stop();
    }

    /// Set which input the channel will use to trigger captures. Defaults to capture input A.
    #[inline]
    pub fn set_input_select(&mut self, chan: T::Channel, sel: CapSelect) {
        self.pause_all();
        // Need to disable capture mode before changing capture settings
        self.timer.set_cmp_mode(chan.into());
        self.timer.set_cap_select(chan.into(), sel);
        self.start_all();
    }

    /// Set the edge trigger for the channel's capture. Defaults to no-capture.
    #[inline]
    pub fn set_capture_trigger(&mut self, chan: T::Channel, mode: CapTrigger) {
        self.pause_all();
        // Need to disable capture mode before changing capture settings
        self.timer.set_cmp_mode(chan.into());
        self.timer.set_cap_trigger(chan.into(), mode);
        self.start_all();
    }

    /// Enable capture interrupts for the selected channel
    #[inline]
    pub fn enable_cap_intr(&mut self, chan: T::Channel) {
        self.timer.ccie_set(chan.into());
    }

    /// Disable capture interrupts for the selected channel
    #[inline]
    pub fn disable_cap_intr(&mut self, chan: T::Channel) {
        self.timer.ccie_clr(chan.into());
    }

    #[inline]
    /// Read the timer interrupt vector. Automatically resets corresponding interrupt flag.
    pub fn interrupt_vector(&mut self) -> TimerVector {
        read_tbxiv(&self.timer)
    }
}

/// Error returned when the previous capture was overwritten before being read
pub struct OverCapture(pub u16);

impl<T: CapturePeriph> Capture for CapturePort<T> {
    type Error = OverCapture;
    type Channel = T::Channel;
    /// Number of cycles. Equivalent to `Self::Capture`.
    type Time = u16;
    /// Number of cycles. Equivalent to `Self::Time`.
    type Capture = u16;

    #[inline(always)]
    fn get_resolution(&self) -> Self::Time {
        1
    }

    #[inline(always)]
    fn set_resolution<U: Into<Self::Time>>(&mut self, _res: U) {}

    #[inline]
    fn capture(&mut self, chan: Self::Channel) -> nb::Result<Self::Capture, Self::Error> {
        let (cov, ccifg) = self.timer.cov_ccifg_rd(chan.into());
        if ccifg {
            let ccrn = self.timer.get_ccrn(chan.into());
            self.timer.cov_ccifg_clr(chan.into());
            if cov {
                Err(nb::Error::Other(OverCapture(ccrn)))
            } else {
                Ok(ccrn)
            }
        } else {
            Err(nb::Error::WouldBlock)
        }
    }

    #[inline]
    fn enable(&mut self, _chan: Self::Channel) {
        unimplemented!();
    }

    #[inline]
    fn disable(&mut self, _chan: Self::Channel) {
        unimplemented!();
    }
}

/// Extension trait for creating capture ports from timer peripherals
pub trait CaptureExt: Sized + TimerClkPin {
    #[doc(hidden)]
    type Capture;

    /// Create new capture port out of timer
    fn to_capture(self, config: TimerConfig<Self>) -> Self::Capture;
}

impl<T: CapturePeriph> CaptureExt for T {
    type Capture = CapturePort<T>;

    #[inline]
    fn to_capture(self, config: TimerConfig<Self>) -> Self::Capture {
        config.write_regs(&self);
        CapturePort { timer: self }
    }
}
