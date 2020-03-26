//! Real time counter
//!
//! Can be used as a periodic 16-bit timer

use crate::clock::Smclk;
use core::marker::PhantomData;
use embedded_hal::timer::{Cancel, CountDown, Periodic};
use msp430fr2355 as pac;
use pac::{rtc::rtcctl::RTCSS_A, RTC};
use void::Void;

mod sealed {
    use super::*;

    pub trait SealedRtcClockSrc {}

    impl SealedRtcClockSrc for RtcSmclk {}
    impl SealedRtcClockSrc for RtcVloclk {}
}

/// Marker trait for RTC clock sources
pub trait RtcClockSrc: sealed::SealedRtcClockSrc {
    #[doc(hidden)]
    const CLK_SRC: RTCSS_A;
}

/// Typestate representing the SMCLK clock source for RTC
pub struct RtcSmclk;

impl RtcClockSrc for RtcSmclk {
    const CLK_SRC: RTCSS_A = RTCSS_A::SMCLK;
}

/// Typestate representing the VLOCLK clock source for RTC
pub struct RtcVloclk;

impl RtcClockSrc for RtcVloclk {
    const CLK_SRC: RTCSS_A = RTCSS_A::VLOCLK;
}

/// 16-bit real-time counter
pub struct Rtc<SRC: RtcClockSrc> {
    periph: RTC,
    _src: PhantomData<SRC>,
}

impl Rtc<RtcVloclk> {
    /// Convert into RTC object with VLOCLK as clock source
    pub fn new(rtc: RTC) -> Self {
        Rtc {
            periph: rtc,
            _src: PhantomData,
        }
    }
}

pub use pac::rtc::rtcctl::RTCPS_A as RtcDiv;

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

    /// Configure the RTC to use VLOCLK as clock source. Setting comes in effect the next time RTC
    /// is started.
    #[inline]
    pub fn use_vloclk(self) -> Rtc<RtcVloclk> {
        Rtc {
            periph: self.periph,
            _src: PhantomData,
        }
    }

    /// Set RTC clock frequency divider
    #[inline]
    pub fn set_clk_div(&mut self, div: RtcDiv) {
        self.periph
            .rtcctl
            .modify(|r, w| unsafe { w.bits(r.bits()) }.rtcps().variant(div));
    }

    /// Enable RTC timer interrupts
    #[inline]
    pub fn enable_interrupts(&mut self) {
        unsafe { self.periph.rtcctl.set_bits(|w| w.rtcie().set_bit()) };
    }

    /// Disable RTC timer interrupts
    #[inline]
    pub fn disable_interrupts(&mut self) {
        unsafe { self.periph.rtcctl.clear_bits(|w| w.rtcie().clear_bit()) };
    }

    /// Clear interrupt flag
    #[inline]
    pub fn clear_interrupt(&mut self) {
        self.periph.rtciv.read();
    }

    /// Read current timer count, which goes up from 0 to 2^16-1
    #[inline]
    pub fn get_count(&self) -> u16 {
        self.periph.rtccnt.read().bits()
    }
}

impl<SRC: RtcClockSrc> CountDown for Rtc<SRC> {
    type Time = u16;

    #[inline]
    fn start<T: Into<Self::Time>>(&mut self, count: T) {
        self.periph
            .rtcmod
            .write(|w| unsafe { w.bits(count.into()) });
        // Need to clear interrupt flag from last timer run
        self.periph.rtciv.read();
        self.periph.rtcctl.modify(|r, w| {
            unsafe { w.bits(r.bits()) }
                .rtcss()
                .variant(SRC::CLK_SRC)
                .rtcsr()
                .set_bit()
        });
    }

    #[inline]
    fn wait(&mut self) -> nb::Result<(), Void> {
        if self.periph.rtcctl.read().rtcifg().bit() {
            self.periph.rtciv.read();
            Ok(())
        } else {
            Err(nb::Error::WouldBlock)
        }
    }
}

impl<SRC: RtcClockSrc> Cancel for Rtc<SRC> {
    type Error = Void;

    #[inline]
    fn cancel(&mut self) -> Result<(), Self::Error> {
        unsafe {
            self.periph
                .rtcctl
                // Bit pattern is all 0s, so we can use clear instead of modify
                .clear_bits(|w| w.rtcss().variant(RTCSS_A::DISABLED))
        };
        Ok(())
    }
}

impl<SRC: RtcClockSrc> Periodic for Rtc<SRC> {}
