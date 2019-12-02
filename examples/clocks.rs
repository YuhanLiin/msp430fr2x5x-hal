#![no_std]

extern crate panic_msp430;

use embedded_hal::digital::v2::*;
use embedded_hal::timer::CountDown;
use embedded_hal::watchdog::WatchdogEnable;
use msp430fr2x5x_hal::{clock::*, fram::*, gpio::*, pmm::*, watchdog::*};
use nb::block;

// Red LED should blink 1 second on, 1 second off
fn main() {
    let periph = msp430fr2355::Peripherals::take().unwrap();

    let mut fram = periph.FRCTL.constrain();
    let wdt = periph.WDT_A.constrain();

    let pmm = periph.PMM.freeze();
    let p1 = periph.P1.batch().config_pin0(|p| p.to_output()).split(&pmm);
    let mut p1_0 = p1.pin0;

    let (smclk, _aclk) = periph
        .CS
        .constrain()
        .mclk_dcoclk(DcoclkFreqSel::_8MHz, MclkDiv::_1)
        .smclk_on(SmclkDiv::_1)
        .aclk_vloclk()
        .freeze(&mut fram);

    const DELAY: WdtClkPeriods = WdtClkPeriods::_8192K;

    // blinks should be 1 second on, 1 second off
    let mut wdt = wdt.to_interval();
    p1_0.set_high().ok();
    wdt.set_smclk(&smclk).start(DELAY);

    block!(wdt.wait()).ok();
    p1_0.set_low().ok();

    let mut wdt = wdt.to_watchdog();
    wdt.start(DELAY);

    loop {
        msp430::asm::nop();
    }
}
