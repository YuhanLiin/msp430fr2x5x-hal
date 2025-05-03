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
        // ~21 nops needed per 2^20 Hz to delay 1 ms
        let nops: u32 = 210 * (freq >> 20);
        SysDelay {
            nops_per_ms: (nops as u16),
        }
    }
}

#[cfg(feature = "embedded-hal-02")]
mod ehal02 {
    use embedded_hal_02::blocking::delay::DelayMs;
    use super::*;

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
