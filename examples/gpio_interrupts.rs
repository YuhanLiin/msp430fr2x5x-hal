#![no_std]
#![feature(abi_msp430_interrupt)]

use msp430fr2355::interrupt;

use core::cell::RefCell;
use embedded_hal::digital::v2::*;
use embedded_hal::timer::*;
use msp430::interrupt::{enable as enable_int, free, Mutex};
use msp430fr2x5x_hal::{clock::*, fram::*, gpio::*, pmm::*, watchdog::*};
use nb::block;
use panic_msp430 as _;

static RED_LED: Mutex<RefCell<Option<Pin<Port1, Pin0, Output>>>> = Mutex::new(RefCell::new(None));
static P2IV: Mutex<RefCell<Option<PxIV<Port2>>>> = Mutex::new(RefCell::new(None));

// Red LED should blink 2 seconds on, 2 seconds off
// Both green and red LEDs should blink when P2.3 LED is pressed
fn main() {
    let periph = msp430fr2355::Peripherals::take().unwrap();
    let (_smclk, aclk) = periph
        .CS
        .constrain()
        .mclk_refoclk(MclkDiv::_1)
        // 32 KHz SMCLK
        .smclk_on(SmclkDiv::_2)
        .aclk_vloclk()
        .freeze(&mut periph.FRCTL.constrain());
    let mut wdt = periph.WDT_A.constrain().to_interval();
    let pmm = periph.PMM.freeze();

    let p1 = periph.P1.batch().split(&pmm);
    let p2 = periph
        .P2
        .batch()
        .config_pin3(|p| p.to_input_pullup())
        .split(&pmm);
    let p6 = periph.P6.batch().config_pin6(|p| p.to_output()).split(&pmm);

    let red_led = p1.pin0.to_output();
    // Onboard button with interrupt disabled
    let mut button = p2.pin3;
    // Some random pin with interrupt enabled. IFG will be set manually.
    let mut pin = p2.pin7.pulldown();
    let mut green_led = p6.pin6;
    let p2iv = p2.pxiv;

    free(|cs| *RED_LED.borrow(&cs).borrow_mut() = Some(red_led));
    free(|cs| *P2IV.borrow(&cs).borrow_mut() = Some(p2iv));

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

interrupt!(PORT2, pin_isr);
fn pin_isr() {
    free(|cs| {
        RED_LED.borrow(&cs).borrow_mut().as_mut().map(|red_led| {
            match P2IV
                .borrow(&cs)
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

interrupt!(WDT, wdt_isr);
fn wdt_isr() {
    free(|cs| {
        RED_LED.borrow(&cs).borrow_mut().as_mut().map(|red_led| {
            red_led.toggle().ok();
        })
    });
}
