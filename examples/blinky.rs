#![no_std]

use embedded_hal::digital::v2::*;
use msp430::asm;
use msp430fr2x5x_hal::{gpio::*, pmm::*};
use panic_msp430 as _;

fn main() {
    let periph = msp430fr2355::Peripherals::take().unwrap();
    let wdt = periph.WDT_A;
    wdt.wdtctl
        .write(|w| unsafe { w.wdtpw().bits(0x5A) }.wdthold().hold());

    let pmm = periph.PMM.freeze();
    let mut p1 = periph.P1.batch().split(&pmm);
    let p2 = periph
        .P2
        .batch()
        .config_pin3(|p| p.to_input().pullup())
        .split(&pmm);
    let mut p6 = periph.P6.batch().config_pin6(|p| p.to_output()).split(&pmm);

    let mut p1_0 = p1.pin0.to_output(&mut p1.pxdir);
    let p2_3 = p2.pin3;
    let mut p6_6 = p6.pin6;

    loop {
        p1_0.proxy(&mut p1.pxout).toggle().ok();

        for _ in 0..5000 {
            if p2_3.is_high().unwrap() {
                p6_6.proxy(&mut p6.pxout).set_low().ok();
            } else {
                p6_6.proxy(&mut p6.pxout).set_high().ok();
            }
        }
    }
}
