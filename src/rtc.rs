use crate::clock::Smclk;
use embedded_hal::timer::{Cancel, CountDown, Periodic};
use msp430fr2355 as pac;
use pac::{rtc::rtcctl::RTCSS_A, RTC};
use void::Void;

/// Extension trait to convert the RTC peripheral to a HAL object
pub trait RtcExt {
    /// Configure the RTC to use SMCLK as clock source
    fn use_smclk(self, smclk: &Smclk) -> Rtc;
    /// Configure the RTC to use VLOCLK as clock source
    fn use_vloclk(self) -> Rtc;
}

/// 16-bit real-timer counter
pub struct Rtc {
    periph: RTC,
    clksel: RTCSS_A,
}

impl RtcExt for RTC {
    #[inline]
    fn use_smclk(self, _smclk: &Smclk) -> Rtc {
        Rtc {
            periph: self,
            clksel: RTCSS_A::SMCLK,
        }
    }

    #[inline]
    fn use_vloclk(self) -> Rtc {
        Rtc {
            periph: self,
            clksel: RTCSS_A::VLOCLK,
        }
    }
}

impl CountDown for Rtc {
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
                .variant(self.clksel)
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

impl Cancel for Rtc {
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

impl Periodic for Rtc {}

pub use pac::rtc::rtcctl::RTCPS_A as RtcDiv;

impl Rtc {
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
}
