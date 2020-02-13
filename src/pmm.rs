//! Power management module

use msp430fr2355::PMM;

/// PMM type
pub struct Pmm(());

mod sealed {
    use super::*;

    pub trait SealedPmmExt {}

    impl SealedPmmExt for PMM {}
}

/// Extension for PMM peripheral
pub trait PmmExt: sealed::SealedPmmExt {
    /// Sets the LOCKLPM5 bit and returns a `Pmm`
    fn freeze(self) -> Pmm;
}

impl PmmExt for PMM {
    #[inline(always)]
    fn freeze(self) -> Pmm {
        self.pm5ctl0.write(|w| w.locklpm5().locklpm5_0());
        Pmm(())
    }
}
