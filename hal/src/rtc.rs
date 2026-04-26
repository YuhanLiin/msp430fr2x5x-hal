//! Real time counter
//!
//! Can be used as a periodic 16-bit timer.
//! 
//! Supports SMCLK, ACLK, VLOCLK, and XT1CLK as clock sources.
//! 
//! Note: On FR2x5x and FR247x series, ACLK and SMCLK share the same RTCCKSEL 
//! hardware bit pattern and are further distinguished via SYSCFG2 selection.

use crate::clock::{Smclk, Xt1clk};
use core::{convert::Infallible, marker::PhantomData};
use crate::_pac::{self, rtc::rtcctl::Rtcss};

#[cfg(feature = "rtc_aclk")]
use crate::clock::Aclk;

mod sealed {
    use super::*;

    pub trait SealedRtcClockSrc {}

    impl SealedRtcClockSrc for RtcSmclk {}
    impl SealedRtcClockSrc for RtcVloclk {}
    #[cfg(feature = "rtc_aclk")]
    impl SealedRtcClockSrc for RtcAclk {}
    impl SealedRtcClockSrc for RtcXt1clk {}
}

/// Marker trait for RTC clock sources
pub trait RtcClockSrc: sealed::SealedRtcClockSrc {
    #[doc(hidden)]
    const CLK_SRC: Rtcss;

    /// Optional hook for clock-specific hardware configuration (e.g., SYSCFG muxes)
    #[doc(hidden)]
    fn apply_sys_config() {}
}

/// Typestate representing the SMCLK clock source for RTC
pub struct RtcSmclk;

impl RtcClockSrc for RtcSmclk {
    const CLK_SRC: Rtcss = Rtcss::Smclk;
    
    #[cfg(feature = "rtc_aclk")]
    fn apply_sys_config() {
        // Ensure the mux is set to SMCLK (0)
        let sys = unsafe { &*_pac::Sys::ptr() };
        sys.syscfg2().modify(|_, w| w.rtccksel().clear_bit());
    }
}

/// Typestate representing the VLOCLK clock source for RTC
pub struct RtcVloclk;

impl RtcClockSrc for RtcVloclk {
    const CLK_SRC: Rtcss = Rtcss::Vloclk;
}

/// Typestate representing the ACLK clock source for RTC
#[cfg(feature = "rtc_aclk")]
pub struct RtcAclk;

#[cfg(feature = "rtc_aclk")]
impl RtcClockSrc for RtcAclk {
    const CLK_SRC: Rtcss = Rtcss::Smclk;
    
    fn apply_sys_config() {
        // Ensure the mux is set to SMCLK (0)
        let sys = unsafe { &*_pac::Sys::ptr() };
        sys.syscfg2().modify(|_, w| w.rtccksel().set_bit());
    }
}

/// Typestate representing the XT1CLK clock source for RTC
pub struct RtcXt1clk;

impl RtcClockSrc for RtcXt1clk {
    const CLK_SRC: Rtcss = Rtcss::Xt1clk;
}

/// 16-bit real-time counter
pub struct Rtc<SRC: RtcClockSrc> {
    periph: _pac::Rtc,
    _src: PhantomData<SRC>,
}

impl Rtc<RtcVloclk> {
    /// Convert into RTC object with VLOCLK as clock source
    pub fn new(rtc: _pac::Rtc) -> Self {
        Rtc {
            periph: rtc,
            _src: PhantomData,
        }
    }
}

pub use crate::_pac::rtc::rtcctl::Rtcps as RtcDiv;

impl<SRC: RtcClockSrc> Rtc<SRC> {
    /// Configure the RTC to use SMCLK as clock source. Setting comes in effect the next time RTC
    /// is started.
    #[inline]
    pub fn use_smclk(self, _smclk: &Smclk) -> Rtc<RtcSmclk> {
        Rtc {
            periph: self.periph,
            _src: PhantomData,
        }
    }

    /// Configure the RTC to use ACLK as clock source. Setting comes in effect the next time RTC
    /// is started.
    #[inline]
    #[cfg(feature = "rtc_aclk")]
    pub fn use_aclk(self, _aclk: &Aclk) -> Rtc<RtcAclk> {
        Rtc {
            periph: self.periph,
            _src: PhantomData,
        }
    }

    /// Configure the RTC to use VLOCLK as clock source. Setting comes in effect the next time RTC
    /// is started.
    #[inline]
    pub fn use_vloclk(self) -> Rtc<RtcVloclk> {
        Rtc {
            periph: self.periph,
            _src: PhantomData,
        }
    }

    /// Configure the RTC to use XT1CLK as clock source. Setting comes in effect the next time RTC
    /// is started.
    #[inline]
    pub fn use_xt1clk(self, _xt1clk: &Xt1clk) -> Rtc<RtcXt1clk> {
        Rtc {
            periph: self.periph,
            _src: PhantomData,
        }
    }

    /// Set RTC clock frequency divider
    #[inline]
    pub fn set_clk_div(&mut self, div: RtcDiv) {
        self.periph
            .rtcctl()
            .modify(|r, w| unsafe { w.bits(r.bits()) }.rtcps().variant(div));
    }

    /// Enable RTC timer interrupts
    #[inline]
    pub fn enable_interrupts(&mut self) {
        unsafe { self.periph.rtcctl().set_bits(|w| w.rtcie().set_bit()) };
    }

    /// Disable RTC timer interrupts
    #[inline]
    pub fn disable_interrupts(&mut self) {
        unsafe { self.periph.rtcctl().clear_bits(|w| w.rtcie().clear_bit()) };
    }

    /// Clear interrupt flag
    #[inline]
    pub fn clear_interrupt(&mut self) {
        self.periph.rtciv().read();
    }

    /// Read current timer count, which goes up from 0 to 2^16-1
    #[inline]
    pub fn get_count(&self) -> u16 {
        self.periph.rtccnt().read().bits()
    }

    #[inline]
    /// Clear the timer contents and start the timer counting up to `count`.
    pub fn start(&mut self, count: u16) {
        self.periph
            .rtcmod()
            .write(|w| unsafe { w.bits(count) });
        // Need to clear interrupt flag from last timer run
        self.periph.rtciv().read();
        SRC::apply_sys_config();
        self.periph.rtcctl().modify(|r, w| {
            unsafe { w.bits(r.bits()) }
                .rtcss()
                .variant(SRC::CLK_SRC)
                .rtcsr()
                .set_bit()
        });
    }

    #[inline]
    /// Checks if the timer has reached the target value, returns `Ok(())` if so, otherwise `WouldBlock`.
    pub fn wait(&mut self) -> nb::Result<(), Infallible> {
        if self.periph.rtcctl().read().rtcifg().bit() {
            self.periph.rtciv().read();
            Ok(())
        } else {
            Err(nb::Error::WouldBlock)
        }
    }

    #[inline]
    /// Pauses the timer.
    pub fn pause(&mut self) {
        unsafe {
            self.periph
                .rtcctl()
                // Bit pattern is all 0s, so we can use clear instead of modify
                .clear_bits(|w| w.rtcss().variant(Rtcss::Disabled))
        };
    }

    #[inline]
    /// Resumes counting from the previous value.
    pub fn resume(&mut self) {
        unsafe {
            self.periph
                .rtcctl()
                .set_bits(|w| w.rtcss().variant(SRC::CLK_SRC))
        }
    }
}

#[cfg(feature = "embedded-hal-02")]
mod ehal02 {
    use super::*;
    use embedded_hal_02::timer::{Cancel, CountDown, Periodic};
    use void::Void;

    impl<SRC: RtcClockSrc> CountDown for Rtc<SRC> {
        type Time = u16;

        #[inline]
        fn start<T: Into<Self::Time>>(&mut self, count: T) {
            self.start(count.into())
        }

        #[inline]
        fn wait(&mut self) -> nb::Result<(), Void> {
            self.wait().map_err(|_| nb::Error::WouldBlock)
        }
    }

    impl<SRC: RtcClockSrc> Cancel for Rtc<SRC> {
        type Error = Void;

        #[inline]
        fn cancel(&mut self) -> Result<(), Self::Error> {
            self.pause();
            Ok(())
        }
    }

    impl<SRC: RtcClockSrc> Periodic for Rtc<SRC> {}
}