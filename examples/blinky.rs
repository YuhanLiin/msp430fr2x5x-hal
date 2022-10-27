#![no_main]
#![no_std]

use embedded_hal::digital::v2::*;
use msp430_rt::entry;
use msp430fr2x5x_hal::{gpio::Batch, pmm::Pmm, watchdog::Wdt};
use panic_msp430 as _;

// Red onboard LED should blink at a steady period.
// Green onboard LED should go on when P2.3 button is pressed
#[entry]
fn main() -> ! {
    let periph = msp430fr2355::Peripherals::take().unwrap();
    let _wdt = Wdt::constrain(periph.WDT_A);

    let pmm = Pmm::new(periph.PMM);
    let p1 = Batch::new(periph.P1).split(&pmm);
    let p2 = Batch::new(periph.P2)
        .config_pin3(|p| p.pullup())
        .split(&pmm);
    let p6 = Batch::new(periph.P6)
        .config_pin6(|p| p.to_output())
        .split(&pmm);

    let mut p1_0 = p1.pin0.to_output();
    let p2_3 = p2.pin3;
    let mut p6_6 = p6.pin6;

    loop {
        p1_0.toggle().ok();

        for _ in 0..5000 {
            if p2_3.is_high().unwrap() {
                p6_6.set_low().ok();
            } else {
                p6_6.set_high().ok();
            }
        }
    }
}

// The compiler will emit calls to the abort() compiler intrinsic if debug assertions are
// enabled (default for dev profile). MSP430 does not actually have meaningful abort() support
// so for now, we create our own in each application where debug assertions are present.
#[no_mangle]
extern "C" fn abort() -> ! {
    panic!();
}
