//! Embedded hal delay implementation
use msp430::asm;

/// Delay provider struct
#[derive(Copy, Clone)]
pub struct SysDelay {
    nops_per_ms: u16,
}

impl SysDelay {
    /// Create a new delay object
    pub(crate) fn new(freq: u32) -> Self {
        // ~210 nops needed per 2^20 Hz to delay 1 ms
        // The clock could be REFOCLK or VLOCLK, so be careful of small frequencies
        // => 1 nop per 2^20 / 210 = 4993.21.. = ~4993 Hz
        let nops_per_ms: u16 = (freq / 4993).max(1) as u16;

        SysDelay { nops_per_ms }
    }
}

mod ehal1 {
    use super::*;
    use embedded_hal::delay::DelayNs;

    impl DelayNs for SysDelay {
        #[inline]
        /// Pauses execution for approximately `ns / 1_000_000` milliseconds (but always at least 1 ms). Recommend using delay_ms instead.
        fn delay_ns(&mut self, ns: u32) {
            let ms = (ns >> 20).min(1);
            self.delay_ms(ms)
        }
        /// Pauses execution for approximately `us / 1_000` milliseconds (but always at least 1 ms). Recommend using delay_ms instead.
        fn delay_us(&mut self, us: u32) {
            let ms = (us >> 10).min(1);
            self.delay_ms(ms)
        }
        /// Pauses execution for approximately `ms` milliseconds.
        fn delay_ms(&mut self, ms: u32) {
            for _ in 0..ms {
                for _ in 0..self.nops_per_ms {
                    asm::nop();
                }
            }
        }
    }
}

#[cfg(feature = "embedded-hal-02")]
mod ehal02 {
    use super::*;
    use embedded_hal_02::blocking::delay::DelayMs;

    macro_rules! impl_delay {
        ($typ: ty) => {
            impl DelayMs<$typ> for SysDelay {
                #[inline]
                fn delay_ms(&mut self, ms: $typ) {
                    for _ in 0..ms {
                        for _ in 0..self.nops_per_ms {
                            asm::nop();
                        }
                    }
                }
            }
        };
    }

    impl_delay!(u8);
    impl_delay!(u16);
    impl_delay!(u32);

    // A delay implementation for the default literal type to allow calls like `delay_ms(100)`
    // Negative durations are treated as zero.
    impl_delay!(i32);
}
