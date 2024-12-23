#![no_main]
#![no_std]

use embedded_hal::digital::v2::*;
use msp430_rt::entry;
use msp430fr2x5x_hal::{
    clock::{ClockConfig, DcoclkFreqSel, MclkDiv, SmclkDiv},
    fram::Fram,
    gpio::Batch,
    hal::blocking::delay::DelayMs,
    pmm::Pmm,
    watchdog::Wdt,
};
use panic_msp430 as _;

// Red onboard LED should blink at a steady period.
#[entry]
fn main() -> ! {
    // Take peripherals and disable watchdog
    let periph = msp430fr2355::Peripherals::take().unwrap();
    let _wdt = Wdt::constrain(periph.WDT_A);

    // Configure GPIO
    let pmm = Pmm::new(periph.PMM);
    let port1 = Batch::new(periph.P1).split(&pmm);
    let mut p1_0 = port1.pin0.to_output();

    // Configure clocks to get accurate delay timing
    let mut fram = Fram::new(periph.FRCTL);
    let (_smclk, _aclk, mut delay) = ClockConfig::new(periph.CS)
        .mclk_dcoclk(DcoclkFreqSel::_8MHz, MclkDiv::_1)
        .smclk_on(SmclkDiv::_1)
        .freeze(&mut fram);

    loop {
        // `toggle()` returns a `Result` because of embedded_hal, but the result is always `Ok` with MSP430 GPIO.
        // Rust complains about unused Results, so we 'use' the Result by calling .ok()
        p1_0.toggle().ok();
        delay.delay_ms(500_u16);
    }
}

// The compiler will emit calls to the abort() compiler intrinsic if debug assertions are
// enabled (default for dev profile). MSP430 does not actually have meaningful abort() support
// so for now, we create our own in each application where debug assertions are present.
#[no_mangle]
extern "C" fn abort() -> ! {
    panic!();
}
