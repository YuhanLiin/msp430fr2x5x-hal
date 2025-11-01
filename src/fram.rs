//! FRAM controller

use crate::pac;
use pac::FRCTL;

/// FRAM controller
pub struct Fram {
    periph: FRCTL,
}

impl Fram {
    /// Turn FRCTL into `Fram`
    pub fn new(fram: FRCTL) -> Self {
        Fram { periph: fram }
    }
}

const PASSWORD: u8 = 0xA5;

/// FRAM wait states
pub enum WaitStates {
    /// No wait
    Wait0,
    /// Wait 1 cycle
    Wait1,
    /// Wait 2 cycles
    Wait2,
    /// Wait 3 cycles
    Wait3,
    /// Wait 4 cycles
    Wait4,
    /// Wait 5 cycles
    Wait5,
    /// Wait 6 cycles
    Wait6,
    /// Wait 7 cycles
    Wait7,
}

impl Fram {
    /// Set number of FRAM wait states. Could cause issues reading instructions from FRAM if
    /// incorrect. Should wait 1 cycle if MCLK > 8MHz and 2 cycles if MCLK > 16MHz.
    #[inline]
    pub unsafe fn set_wait_states(&mut self, wait: WaitStates) {
        self.periph
            .frctl0
            .write(|w| w.frctlpw().bits(PASSWORD).nwaits().bits(wait as u8));
    }
}
