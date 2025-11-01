//! Watchdog timer, configurable as either a traditional watchdog or a 16-bit timer.
//!
//! **Note**: MSP430 devices will reset after bootup if watchdog is not stopped after an initial 32
//! ms interval (roughly). If this is undesirable, call `Wdt::constrain()` as soon in the
//! application as possible to stop the watchdog.

use crate::clock::{Aclk, Smclk};
use core::{convert::Infallible, marker::PhantomData};
use crate::pac;
use pac::wdt_a::wdtctl::WDTSSEL_A;

const PASSWORD: u8 = 0x5A;

pub use pac::wdt_a::wdtctl::WDTIS_A as WdtClkPeriods;

mod sealed {
    use super::*;

    pub trait SealedWatchdogSelect {}

    impl SealedWatchdogSelect for WatchdogMode {}
    impl SealedWatchdogSelect for IntervalMode {}
}

/// Watchdog timer which can be configured to watchdog or interval (timer) mode
pub struct Wdt<MODE> {
    _mode: PhantomData<MODE>,
    periph: pac::WDT_A,
}

impl Wdt<WatchdogMode> {
    /// Convert WDT peripheral into a watchdog timer (watchdog mode) and disable the watchdog. Set
    /// clock source to VLOCLK.
    pub fn constrain(wdt: pac::WDT_A) -> Self {
        // Disable first
        wdt.wdtctl.write(|w| {
            unsafe { w.wdtpw().bits(PASSWORD) }
                .wdthold()
                .hold()
                .wdtssel()
                .variant(WDTSSEL_A::VLOCLK)
        });
        Wdt {
            _mode: PhantomData,
            periph: wdt,
        }
    }
}

/// Watchdog mode typestate
pub struct WatchdogMode;
/// Interval mode typestate
pub struct IntervalMode;

/// Marker trait for watchdog modes
pub trait WatchdogSelect: sealed::SealedWatchdogSelect {
    #[doc(hidden)]
    fn mode_bit() -> bool;
}
impl WatchdogSelect for WatchdogMode {
    #[inline(always)]
    fn mode_bit() -> bool {
        false
    }
}
impl WatchdogSelect for IntervalMode {
    #[inline(always)]
    fn mode_bit() -> bool {
        true
    }
}

type WdtWriter = pac::wdt_a::wdtctl::W;

impl<MODE: WatchdogSelect> Wdt<MODE> {
    #[inline(always)]
    fn prewrite(w: &mut WdtWriter, bits: u16) -> &mut WdtWriter {
        // Write argument bits, password, and correct mode bit to the watchdog write proxy
        unsafe { w.bits(bits).wdtpw().bits(PASSWORD) }
            .wdttmsel()
            .bit(MODE::mode_bit())
    }

    #[inline]
    fn set_clk(&mut self, clk_src: WDTSSEL_A) -> &mut Self {
        // Halt timer first, as specified in the user's guide
        self.periph.wdtctl.write(|w| {
            Self::prewrite(w, 0)
                .wdthold()
                .hold()
                // Also reset timer
                .wdtcntcl()
                .set_bit()
        });
        // Set clock src and keep timer halted
        self.periph.wdtctl.write(|w| {
            Self::prewrite(w, 0)
                .wdtssel()
                .variant(clk_src)
                .wdthold()
                .hold()
        });
        self
    }

    /// Set watchdog clock source to ACLK and halt timer.
    #[inline]
    pub fn set_aclk(&mut self, _clks: &Aclk) -> &mut Self {
        self.set_clk(WDTSSEL_A::ACLK)
    }

    /// Set watchdog clock source to VLOCLK and halt timer.
    #[inline]
    pub fn set_vloclk(&mut self) -> &mut Self {
        self.set_clk(WDTSSEL_A::VLOCLK)
    }

    /// Set watchdog clock source to SMCLK and halt timer.
    #[inline]
    pub fn set_smclk(&mut self, _clks: &Smclk) -> &mut Self {
        self.set_clk(WDTSSEL_A::SMCLK)
    }

    /// Reset countdown, unpause timer, and set timeout in a single write
    #[inline]
    pub fn set_interval_and_start(&mut self, periods: WdtClkPeriods) {
        self.periph.wdtctl.modify(|r, w| {
            Self::prewrite(w, r.bits())
                .wdtcntcl()
                .set_bit()
                .wdthold()
                .unhold()
                .wdtis()
                .variant(periods)
        });
    }

    /// Pause the timer.
    #[inline]
    pub fn pause(&mut self) {
        self.periph
            .wdtctl
            .modify(|r, w| Self::prewrite(w, r.bits()).wdthold().hold());
    }

    /// Resumes the timer, counting from the previously stored value.
    #[inline]
    pub fn resume(&mut self) {
        self.periph
            .wdtctl
            .modify(|r, w| Self::prewrite(w, r.bits()).wdthold().unhold());
    }

    /// Checks if the timer has expired, returning `Ok(())` if it has, otherwise `WouldBlock`.
    /// If called while the timer is not running, this will always return `WouldBlock`.
    #[inline]
    pub fn wait(&mut self) -> nb::Result<(), Infallible> {
        let sfr = unsafe { &*pac::SFR::ptr() };
        if sfr.sfrifg1.read().wdtifg().is_wdtifg_1() {
            unsafe { sfr.sfrifg1.clear_bits(|w| w.wdtifg().clear_bit()) };
            Ok(())
        } else {
            Err(nb::Error::WouldBlock)
        }
    }
}

impl Wdt<WatchdogMode> {
    /// Convert to interval mode and pause timer
    #[inline]
    pub fn to_interval(self) -> Wdt<IntervalMode> {
        let mut wdt = Wdt {
            _mode: PhantomData,
            periph: self.periph,
        };
        // Change mode bit and pause timer
        wdt.pause();
        wdt
    }

    /// Refreshes the watchdog timer, preventing the processor from being reset.
    pub fn feed(&mut self) {
        self.periph
            .wdtctl
            .modify(|r, w| Self::prewrite(w, r.bits()).wdtcntcl().set_bit());
    }
}

impl Wdt<IntervalMode> {
    /// Convert to watchdog mode and pause timer
    #[inline]
    pub fn to_watchdog(self) -> Wdt<WatchdogMode> {
        let mut wdt = Wdt {
            _mode: PhantomData,
            periph: self.periph,
        };
        // Change mode bit and pause timer
        wdt.pause();
        // Wipe out old interrupt flag, which may cause a watchdog reset
        let sfr = unsafe { &*pac::SFR::ptr() };
        unsafe { sfr.sfrifg1.clear_bits(|w| w.wdtifg().clear_bit()) };
        wdt
    }

    /// Enable interrupts for watchdog, which fires when the watchdog interrupt flag is set in
    /// interval mode. This setting does nothing in watchdog mode, but will carry over when
    /// switching to interval mode.
    #[inline]
    pub fn enable_interrupts(&mut self) -> &mut Self {
        let sfr = unsafe { &*pac::SFR::ptr() };
        unsafe { sfr.sfrie1.set_bits(|w| w.wdtie().set_bit()) };
        self
    }

    /// Disable interrupts for watchdog.
    #[inline]
    pub fn disable_interrupts(&mut self) -> &mut Self {
        let sfr = unsafe { &*pac::SFR::ptr() };
        unsafe { sfr.sfrie1.clear_bits(|w| w.wdtie().clear_bit()) };
        self
    }
}

#[cfg(feature = "embedded-hal-02")]
mod ehal02 {
    use super::*;
    use embedded_hal_02::timer::{Cancel, CountDown, Periodic};
    use embedded_hal_02::watchdog::{Watchdog, WatchdogDisable, WatchdogEnable};

    impl Watchdog for Wdt<WatchdogMode> {
        #[inline]
        fn feed(&mut self) {
            self.feed()
        }
    }

    impl WatchdogEnable for Wdt<WatchdogMode> {
        type Time = WdtClkPeriods;

        #[inline]
        fn start<T>(&mut self, period: T)
        where
            T: Into<Self::Time>,
        {
            self.set_interval_and_start(period.into());
        }
    }

    impl WatchdogDisable for Wdt<WatchdogMode> {
        #[inline]
        fn disable(&mut self) {
            self.pause();
        }
    }

    impl CountDown for Wdt<IntervalMode> {
        type Time = WdtClkPeriods;

        #[inline]
        fn start<T>(&mut self, count: T)
        where
            T: Into<Self::Time>,
        {
            self.set_interval_and_start(count.into());
        }

        /// If called while timer is not running, this will always return WouldBlock.
        #[inline]
        fn wait(&mut self) -> nb::Result<(), void::Void> {
            self.wait().map_err(|_| nb::Error::WouldBlock)
        }
    }

    impl Cancel for Wdt<IntervalMode> {
        type Error = void::Void;

        /// This implementation will never return error even if watchdog has already been paused, hence
        /// the `Void` error type.
        #[inline]
        fn cancel(&mut self) -> Result<(), Self::Error> {
            self.pause();
            Ok(())
        }
    }

    impl Periodic for Wdt<IntervalMode> {}
}
