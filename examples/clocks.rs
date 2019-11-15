#![no_std]

extern crate panic_msp430;

use embedded_hal::digital::v2::*;
use embedded_hal::timer::CountDown;
use embedded_hal::watchdog::WatchdogEnable;
use msp430fr2x5x_hal::{clock::*, gpio::*, pmm::*, watchdog::*};
use nb::block;

fn main() {
    let periph = msp430fr2355::Peripherals::take().unwrap();

    let wdt = periph.WDT_A.constrain();

    let pmm = periph.PMM.freeze();
    let mut p1 = periph.P1.batch().config_pin0(|p| p.to_output()).split(&pmm);
    let mut p1_0 = p1.pin0;

    let (_mclk, smclk, _aclk) = periph
        .CS
        .constrain()
        .mclk_refoclk(MclkDiv::_1)
        // 32 KHz SMCLK
        .smclk_on(SmclkDiv::_1)
        .aclk_vloclk()
        .freeze();

    // blinks should be 1 second on, 1 second off
    let mut wdt = wdt.set_smclk(&smclk).to_interval();
    wdt.start(WdtClkPeriods::_32K);

    block!(wdt.wait()).ok();
    p1_0.proxy(&mut p1.pxout).set_high().ok();

    let mut wdt = wdt.to_watchdog();
    wdt.start(WdtClkPeriods::_32K);

    loop {
        msp430::asm::nop();
    }
}
