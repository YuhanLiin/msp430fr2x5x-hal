#![no_std]

use msp430fr2x5x_hal::prelude::*;
use panic_msp430 as _;

fn main() {
    let periph = msp430fr2355::Peripherals::take().unwrap();
    let _wdt = periph.WDT_A.constrain();

    let pmm = periph.PMM.freeze();
    let p1 = periph.P1.batch().split(&pmm);

    // Convert P1.0 to SMCLK output
    // Alternate 1 to alternate 2 conversion requires using SELC register
    // Expect red LED to light up
    p1.pin0.to_output().to_alternate1().to_alternate2();

    loop {
        msp430::asm::nop();
    }
}
