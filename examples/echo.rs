#![no_std]

use embedded_hal::digital::v2::OutputPin;
use embedded_hal::prelude::*;
use msp430fr2x5x_hal::{clock::*, fram::*, gpio::*, pmm::*, serial::*, watchdog::*};
use nb::block;
use panic_msp430 as _;

fn main() {
    let periph = msp430fr2355::Peripherals::take().unwrap();

    let mut fram = periph.FRCTL.constrain();
    let _wdt = periph.WDT_A.constrain();

    let (_smclk, aclk) = periph
        .CS
        .constrain()
        .mclk_dcoclk(DcoclkFreqSel::_1MHz, MclkDiv::_1)
        .smclk_on(SmclkDiv::_2)
        .aclk_refoclk()
        .freeze(&mut fram);

    let pmm = periph.PMM.freeze();
    let mut led = periph.P1.batch().split(&pmm).pin0.to_output();
    let p4 = periph.P4.batch().split(&pmm);
    led.set_low().ok();

    let (mut tx, mut rx) = periph
        .E_USCI_A1
        .to_serial(
            BitOrder::LsbFirst,
            BitCount::EightBits,
            StopBits::OneStopBit,
            Parity::NoParity,
            Loopback::NoLoop,
            9600,
        )
        .use_aclk(&aclk)
        .split(p4.pin3.to_alternate1(), p4.pin2.to_alternate1());

    led.set_high().ok();

    loop {
        let ch = match block!(rx.read()) {
            Ok(c) => c,
            Err(_err) => '!' as u8,
        };
        block!(tx.write(ch)).ok();
    }
}
