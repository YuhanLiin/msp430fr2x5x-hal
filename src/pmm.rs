use msp430fr2355::PMM;

pub struct Pmm(());

pub trait PmmExt {
    fn freeze(self) -> Pmm;
}

impl PmmExt for PMM {
    fn freeze(self) -> Pmm {
        self.pm5ctl0.write(|w| w.locklpm5().locklpm5_0());
        Pmm(())
    }
}
