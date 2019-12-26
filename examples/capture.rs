#![no_std]

extern crate panic_msp430;

use embedded_hal::digital::v2::*;
use embedded_hal::prelude::*;
use msp430fr2x5x_hal::{
    capture::{CapSelect, CapThreeChannel, CapTrigger, OverCapture, TimerConfig},
    clock::{DcoclkFreqSel, MclkDiv, SmclkDiv},
    prelude::*,
    serial::*,
};
use nb::block;
use void::ResultVoidExt;

// Connect push button input to P1.6. When button is pressed, putty should print the # of cycles
// since the last press. Sometimes we get 2 consecutive readings due to lack of debouncing.
fn main() {
    let periph = msp430fr2355::Peripherals::take().unwrap();

    let mut fram = periph.FRCTL.constrain();
    periph.WDT_A.constrain();

    let pmm = periph.PMM.freeze();
    let p4 = periph.P4.batch().split(&pmm);
    let mut p1 = periph.P1.batch().config_pin0(|p| p.to_output()).split(&pmm);

    let (smclk, aclk) = periph
        .CS
        .constrain()
        .mclk_dcoclk(DcoclkFreqSel::_1MHz, MclkDiv::_1)
        .smclk_on(SmclkDiv::_1)
        .aclk_vloclk()
        .freeze(&mut fram);

    let mut tx = periph
        .E_USCI_A1
        .to_serial(
            BitOrder::LsbFirst,
            BitCount::EightBits,
            StopBits::OneStopBit,
            Parity::NoParity,
            Loopback::NoLoop,
            9600,
        )
        .use_smclk(&smclk)
        .tx_only(p4.pin3.to_alternate1());

    let mut capture = periph.TB0.to_capture(TimerConfig::aclk(&aclk));
    const CHAN1: CapThreeChannel = CapThreeChannel::Chan1;
    capture.set_capture_trigger(CHAN1, CapTrigger::FallingEdge);
    capture.set_input_select(CHAN1, CapSelect::InputA);

    p1.pin6.to_alternate2();
    write(&mut tx, '4');

    let mut last_cap = 0;
    loop {
        match block!(capture.capture(CHAN1)) {
            Ok(cap) => {
                let diff = cap.wrapping_sub(last_cap);
                last_cap = cap;
                p1.pin0.set_high().void_unwrap();
                print_num(&mut tx, diff);
            }
            Err(OverCapture(_)) => {
                p1.pin0.set_high().void_unwrap();
                write(&mut tx, '!');
                write(&mut tx, '\n');
            }
        }
    }
}

fn print_num<U: SerialUsci>(tx: &mut Tx<U>, num: u16) {
    write(tx, '0');
    write(tx, 'x');
    print_hex(tx, num >> 12);
    print_hex(tx, (num >> 8) & 0xF);
    print_hex(tx, (num >> 4) & 0xF);
    print_hex(tx, num & 0xF);
    write(tx, '\n');
}

fn print_hex<U: SerialUsci>(tx: &mut Tx<U>, h: u16) {
    let c = match h {
        0 => '0',
        1 => '1',
        2 => '2',
        3 => '3',
        4 => '4',
        5 => '5',
        6 => '6',
        7 => '7',
        8 => '8',
        9 => '9',
        10 => 'a',
        11 => 'b',
        12 => 'c',
        13 => 'd',
        14 => 'e',
        15 => 'f',
        _ => '?',
    };
    write(tx, c);
}

fn write<U: SerialUsci>(tx: &mut Tx<U>, ch: char) {
    block!(tx.write(ch as u8)).void_unwrap();
}
