//! Embedded hal delay implementation
use msp430::asm;
use crate::hal::blocking::delay::{DelayMs, DelayUs};


/// Delay provider struct
pub struct Delay{
    freq: u32
}

impl Delay{
    #[doc(hidden)]
    pub fn new(freq: u32) -> Self{
        Delay{freq}
    }
}

impl DelayMs<u8> for Delay{
    #[inline]
    fn delay_ms(&mut self, ms: u8){
        self.delay_ms(ms as u16);
    }
}

impl DelayMs<u16> for Delay{
    fn delay_ms(&mut self, ms: u16){
        //TODO take into account clock freq for delay
        //
        for _ in 0 .. ms{
            for _ in 0 .. 200{
                asm::nop();
            }
        }
    }
}
