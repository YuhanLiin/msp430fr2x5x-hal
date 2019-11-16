#![no_std]

use embedded_hal::digital::v2::*;
use msp430fr2x5x_hal::{gpio::*, pmm::*};
use panic_msp430 as _;

fn main() {
    let periph = msp430fr2355::Peripherals::take().unwrap();
    let wdt = periph.WDT_A;
    wdt.wdtctl
        .write(|w| unsafe { w.wdtpw().bits(0x5A) }.wdthold().hold());

    let pmm = periph.PMM.freeze();
    let p1 = periph.P1.batch().split(&pmm);
    let p2 = periph
        .P2
        .batch()
        .config_pin3(|p| p.to_input_pullup())
        .split(&pmm);
    let p6 = periph.P6.batch().config_pin6(|p| p.to_output()).split(&pmm);

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
