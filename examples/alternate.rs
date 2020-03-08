#![no_main]
#![no_std]

use msp430_rt::entry;
use msp430fr2x5x_hal::{gpio::Batch, pmm::Pmm, watchdog::Wdt};
use panic_msp430 as _;

#[entry]
fn main() -> ! {
    let periph = msp430fr2355::Peripherals::take().unwrap();
    let _wdt = Wdt::constrain(periph.WDT_A);

    let pmm = Pmm::new(periph.PMM);
    let p1 = Batch::new(periph.P1).split(&pmm);

    // Convert P1.0 to SMCLK output
    // Alternate 1 to alternate 2 conversion requires using SELC register
    // Expect red LED to light up
    p1.pin0.to_output().to_alternate1().to_alternate2();

    loop {
        msp430::asm::nop();
    }
}
