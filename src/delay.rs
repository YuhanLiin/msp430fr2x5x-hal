//! Embedded hal delay implementation
use crate::hal::blocking::delay::DelayMs;
use msp430::asm;

/// Delay provider struct
#[derive(Copy, Clone)]
pub struct Delay {
    nops_per_ms: u16,
}

impl Delay {
    /// Create a new delay object
    pub(crate) fn new(freq: u32) -> Self {
        // ~21 nops needed per 2^20 Hz to delay 1 ms
        let nops: u32 = 210 * (freq >> 20);
        Delay {
            nops_per_ms: (nops as u16),
        }
    }
}

impl DelayMs<u16> for Delay {
    #[inline]
    fn delay_ms(&mut self, ms: u16) {
        for _ in 0..ms {
            for _ in 0..self.nops_per_ms {
                asm::nop();
            }
        }
    }
}
