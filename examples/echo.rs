#![no_std]

use embedded_hal::digital::v2::OutputPin;
use embedded_hal::prelude::*;
use msp430fr2x5x_hal::{
    clock::{DcoclkFreqSel, MclkDiv, SmclkDiv},
    prelude::*,
    serial::*,
};
use nb::block;
use panic_msp430 as _;

// Prints "HELLO" when started then echos on UART1
// Serial settings are listed in the code
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
            // Launchpad UART-to-USB converter doesn't handle parity, so we don't use it
            Parity::NoParity,
            Loopback::NoLoop,
            9600,
        )
        .use_aclk(&aclk)
        .split(p4.pin3.to_alternate1(), p4.pin2.to_alternate1());

    led.set_high().ok();
    tx.bwrite_all(b"HELLO\n").ok();

    loop {
        let ch = match block!(rx.read()) {
            Ok(c) => c,
            Err(err) => {
                (match err {
                    RecvError::Parity => '!',
                    RecvError::Overrun(_) => '}',
                    RecvError::Framing => '?',
                }) as u8
            }
        };
        block!(tx.write(ch)).ok();
    }
}
