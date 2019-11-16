#![no_std]
#![feature(abi_msp430_interrupt)]

use msp430fr2355::interrupt;

use core::cell::RefCell;
use embedded_hal::digital::v2::*;
use embedded_hal::timer::*;
use msp430::interrupt::{enable as enable_int, free, Mutex};
use msp430fr2x5x_hal::{clock::*, gpio::*, pmm::*, watchdog::*};
use nb::block;
use panic_msp430 as _;

static RED_LED: Mutex<RefCell<Option<Pin<Port1, Pin0, Output>>>> = Mutex::new(RefCell::new(None));

fn main() {
    let periph = msp430fr2355::Peripherals::take().unwrap();
    let (_mclk, _smclk, aclk) = periph
        .CS
        .constrain()
        .mclk_refoclk(MclkDiv::_1)
        // 32 KHz SMCLK
        .smclk_on(SmclkDiv::_2)
        .aclk_vloclk()
        .freeze();
    let mut wdt = periph.WDT_A.constrain().set_aclk(&aclk).to_interval();
    let pmm = periph.PMM.freeze();

    let p1 = periph.P1.batch().split(&pmm);
    let p2 = periph
        .P2
        .batch()
        .config_pin3(|p| p.to_input_pullup())
        .split(&pmm);
    let p6 = periph.P6.batch().config_pin6(|p| p.to_output()).split(&pmm);

    let red_led = p1.pin0.to_output();
    let mut button = p2.pin3;
    let mut pin = p2.pin7.pulldown();
    let mut green_led = p6.pin6;

    free(|cs| *RED_LED.borrow(&cs).borrow_mut() = Some(red_led));

    wdt.enable_interrupts().start(WdtClkPeriods::_32K);
    pin.select_rising_edge_trigger().enable_interrupts();
    button.select_falling_edge_trigger();

    unsafe { enable_int() };

    loop {
        block!(button.wait_for_ifg()).ok();
        green_led.toggle().ok();
        pin.set_ifg();
    }
}

interrupt!(PORT2, pin_isr);
fn pin_isr() {
    free(|cs| {
        RED_LED.borrow(&cs).borrow_mut().as_mut().map(|red_led| {
            match Port2::get_interrupt_vector() {
                InterruptVector::Pin7Isr => red_led.toggle().ok(),
                _ => unreachable!(),
            }
        })
    });
}

interrupt!(WDT, wdt_isr);
fn wdt_isr() {
    free(|cs| {
        RED_LED.borrow(&cs).borrow_mut().as_mut().map(|red_led| {
            red_led.toggle().ok();
        })
    });
}
