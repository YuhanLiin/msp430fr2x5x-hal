#![no_main]
#![no_std]

use embedded_hal::digital::v2::OutputPin;
use embedded_hal::prelude::*;
use embedded_hal::serial::Read;
use msp430_rt::entry;
use msp430fr2x5x_hal::{
    clock::{ClockConfig, DcoclkFreqSel, MclkDiv, Smclk, SmclkDiv},
    fram::Fram,
    gpio::Batch,
    pmm::Pmm,
    serial::*,
    watchdog::Wdt,
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
    SerialConfig::new(
        usci,
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
#[entry]
fn main() -> ! {
    let periph = msp430fr2355::Peripherals::take().unwrap();

    let mut fram = Fram::new(periph.FRCTL);
    let _wdt = Wdt::constrain(periph.WDT_A);

    let (smclk, _aclk) = ClockConfig::new(periph.CS)
        .mclk_dcoclk(DcoclkFreqSel::_4MHz, MclkDiv::_1)
        .smclk_on(SmclkDiv::_2)
        .aclk_refoclk()
        .freeze(&mut fram);

    let pmm = Pmm::new(periph.PMM);
    let p1 = Batch::new(periph.P1).split(&pmm);
    let p4 = Batch::new(periph.P4).split(&pmm);
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

// The compiler will emit calls to the abort() compiler intrinsic if debug assertions are
// enabled (default for dev profile). MSP430 does not actually have meaningful abort() support
// so for now, we create our own in each application where debug assertions are present.
#[no_mangle]
extern "C" fn abort() -> ! {
    panic!();
}
