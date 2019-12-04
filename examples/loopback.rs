#![no_std]

use embedded_hal::digital::v2::OutputPin;
use embedded_hal::prelude::*;
use embedded_hal::serial::Read;
use msp430fr2x5x_hal::{
    clock::{DcoclkFreqSel, MclkDiv, Smclk, SmclkDiv},
    prelude::*,
    serial::*,
};
use nb::block;
use panic_msp430 as _;

fn setup_uart<S: SerialUsci>(
    usci: S,
    tx: S::TxPin,
    rx: S::RxPin,
    parity: Parity,
    loopback: Loopback,
    baudrate: u32,
    smclk: &Smclk,
) -> (Tx<S>, Rx<S>) {
    usci.to_serial(
        BitOrder::LsbFirst,
        BitCount::EightBits,
        StopBits::TwoStopBits,
        parity,
        loopback,
        baudrate,
    )
    .use_smclk(&smclk)
    .split(tx, rx)
}

fn read_unwrap<R: Read<u8>>(rx: &mut R, err: char) -> u8 {
    match block!(rx.read()) {
        Ok(c) => c,
        Err(_) => err as u8,
    }
}

// Echoes serial input on UART1 by roundtripping to UART0
// Only UART1 settings matter for the host
fn main() {
    let periph = msp430fr2355::Peripherals::take().unwrap();

    let mut fram = periph.FRCTL.constrain();
    let _wdt = periph.WDT_A.constrain();

    let (smclk, _aclk) = periph
        .CS
        .constrain()
        .mclk_dcoclk(DcoclkFreqSel::_4MHz, MclkDiv::_1)
        .smclk_on(SmclkDiv::_2)
        .aclk_refoclk()
        .freeze(&mut fram);

    let pmm = periph.PMM.freeze();
    let p1 = periph.P1.batch().split(&pmm);
    let p4 = periph.P4.batch().split(&pmm);
    let mut led = p1.pin0.to_output();
    led.set_low().ok();

    let (mut tx0, mut rx0) = setup_uart(
        periph.E_USCI_A0,
        p1.pin7.to_alternate1().into(),
        p1.pin6.to_alternate1().into(),
        Parity::EvenParity,
        Loopback::Loopback,
        20000,
        &smclk,
    );

    let (mut tx1, mut rx1) = setup_uart(
        periph.E_USCI_A1,
        p4.pin3.to_alternate1().into(),
        p4.pin2.to_alternate1().into(),
        Parity::NoParity,
        Loopback::NoLoop,
        19200,
        &smclk,
    );

    led.set_high().ok();

    loop {
        let ch = read_unwrap(&mut rx1, '!');
        block!(tx0.write(ch)).ok();
        let ch = read_unwrap(&mut rx0, '?');
        block!(tx1.write(ch)).ok();
    }
}
