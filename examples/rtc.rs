#![no_std]

extern crate panic_msp430;

use embedded_hal::digital::v2::*;
use embedded_hal::prelude::*;
use embedded_hal::timer::Cancel;
use msp430fr2x5x_hal::{clock::*, gpio::*, pmm::*, rtc::*, watchdog::*};

// Red LED blinks 2 seconds on, 2 off
// Pressing P2.3 button halts program
fn main() {
    let periph = msp430fr2355::Peripherals::take().unwrap();

    periph.WDT_A.constrain();

    let pmm = periph.PMM.freeze();
    let p1 = periph.P1.batch().config_pin0(|p| p.to_output()).split(&pmm);
    let p2 = periph
        .P2
        .batch()
        .config_pin3(|p| p.to_input_pullup())
        .split(&pmm);
    let mut led = p1.pin0;
    let mut button = p2.pin3;

    let (_smclk, _aclk) = periph
        .CS
        .constrain()
        .mclk_refoclk(MclkDiv::_1)
        .smclk_on(SmclkDiv::_1)
        .aclk_vloclk()
        .freeze();

    let mut rtc = periph.RTC.use_vloclk();
    rtc.set_clk_div(RtcDiv::_10);

    button.select_falling_edge_trigger();
    led.set_high().ok();

    loop {
        // 2 seconds
        rtc.start(2000u16);
        while let Err(nb::Error::WouldBlock) = rtc.wait() {
            if let Ok(_) = button.wait_for_ifg() {
                led.toggle().ok();
                rtc.cancel().ok();
            }
        }
        led.toggle().ok();
    }
}
