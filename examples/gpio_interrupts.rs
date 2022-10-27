#![no_main]
#![no_std]
#![feature(abi_msp430_interrupt)]

use critical_section::with;
use msp430fr2355::interrupt;

use core::cell::RefCell;
use embedded_hal::digital::v2::*;
use embedded_hal::timer::*;
use msp430::interrupt::{enable as enable_int, Mutex};
use msp430_rt::entry;
use msp430fr2x5x_hal::{
    clock::{ClockConfig, MclkDiv, SmclkDiv},
    fram::Fram,
    gpio::{Batch, GpioVector, Output, Pin, Pin0, PxIV, P1, P2},
    pmm::Pmm,
    watchdog::{Wdt, WdtClkPeriods},
};
use nb::block;
use panic_msp430 as _;

static RED_LED: Mutex<RefCell<Option<Pin<P1, Pin0, Output>>>> = Mutex::new(RefCell::new(None));
static P2IV: Mutex<RefCell<Option<PxIV<P2>>>> = Mutex::new(RefCell::new(None));

// Red LED should blink 2 seconds on, 2 seconds off
// Both green and red LEDs should blink when P2.3 LED is pressed
#[entry]
fn main() -> ! {
    let periph = msp430fr2355::Peripherals::take().unwrap();
    let (_smclk, aclk) = ClockConfig::new(periph.CS)
        .mclk_refoclk(MclkDiv::_1)
        // 32 KHz SMCLK
        .smclk_on(SmclkDiv::_2)
        .aclk_vloclk()
        .freeze(&mut Fram::new(periph.FRCTL));
    let mut wdt = Wdt::constrain(periph.WDT_A).to_interval();
    let pmm = Pmm::new(periph.PMM);

    let p1 = Batch::new(periph.P1).split(&pmm);
    let p2 = Batch::new(periph.P2)
        .config_pin3(|p| p.pullup())
        .split(&pmm);
    let p6 = Batch::new(periph.P6)
        .config_pin6(|p| p.to_output())
        .split(&pmm);

    let red_led = p1.pin0.to_output();
    // Onboard button with interrupt disabled
    let mut button = p2.pin3;
    // Some random pin with interrupt enabled. IFG will be set manually.
    let mut pin = p2.pin7.pulldown();
    let mut green_led = p6.pin6;
    let p2iv = p2.pxiv;

    with(|cs| *RED_LED.borrow(cs).borrow_mut() = Some(red_led));
    with(|cs| *P2IV.borrow(cs).borrow_mut() = Some(p2iv));

    wdt.set_aclk(&aclk)
        .enable_interrupts()
        .start(WdtClkPeriods::_32K);
    pin.select_rising_edge_trigger().enable_interrupts();
    button.select_falling_edge_trigger();

    unsafe { enable_int() };

    loop {
        block!(button.wait_for_ifg()).ok();
        green_led.toggle().ok();
        pin.set_ifg();
    }
}

#[interrupt]
fn PORT2() {
    with(|cs| {
        RED_LED.borrow(cs).borrow_mut().as_mut().map(|red_led| {
            match P2IV
                .borrow(cs)
                .borrow_mut()
                .as_mut()
                .unwrap()
                .get_interrupt_vector()
            {
                GpioVector::Pin7Isr => red_led.toggle().ok(),
                _ => panic!(),
            }
        })
    });
}

#[interrupt]
fn WDT() {
    with(|cs| {
        RED_LED.borrow(cs).borrow_mut().as_mut().map(|red_led| {
            red_led.toggle().ok();
        })
    });
}

// The compiler will emit calls to the abort() compiler intrinsic if debug assertions are
// enabled (default for dev profile). MSP430 does not actually have meaningful abort() support
// so for now, we create our own in each application where debug assertions are present.
#[no_mangle]
extern "C" fn abort() -> ! {
    panic!();
}
