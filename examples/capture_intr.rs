#![no_main]
#![no_std]
#![feature(abi_msp430_interrupt)]
#![feature(let_else)]

// This example also demonstrates how to write panic-free code using panic_never.

use core::cell::UnsafeCell;
use critical_section::with;
use embedded_hal::digital::v2::ToggleableOutputPin;
use msp430::interrupt::{enable, Mutex};
use msp430_rt::entry;
use msp430fr2355::interrupt;
use msp430fr2x5x_hal::{
    capture::{
        CapCmp, CapTrigger, Capture, CaptureParts3, CaptureVector, TBxIV, TimerConfig, CCR1,
    },
    clock::{ClockConfig, DcoclkFreqSel, MclkDiv, SmclkDiv},
    fram::Fram,
    gpio::Batch,
    gpio::*,
    pmm::Pmm,
    watchdog::Wdt,
};
use void::ResultVoidExt;

#[cfg(debug_assertions)]
use panic_msp430 as _;

#[cfg(not(debug_assertions))]
use panic_never as _;

// We use UnsafeCell as a panic-free version of RefCell. If you aren't using `panic_never` then RefCell is more ergonomic.
static CAPTURE: Mutex<UnsafeCell<Option<Capture<msp430fr2355::TB0, CCR1>>>> =
    Mutex::new(UnsafeCell::new(None));
static VECTOR: Mutex<UnsafeCell<Option<TBxIV<msp430fr2355::TB0>>>> =
    Mutex::new(UnsafeCell::new(None));
static RED_LED: Mutex<UnsafeCell<Option<Pin<P1, Pin0, Output>>>> =
    Mutex::new(UnsafeCell::new(None));

// Connect push button input to P1.6. When button is pressed, red LED should toggle. No debouncing,
// so sometimes inputs are missed.
#[entry]
fn main() -> ! {
    let Some(periph) = msp430fr2355::Peripherals::take() else { loop {} };
    let mut fram = Fram::new(periph.FRCTL);
    Wdt::constrain(periph.WDT_A);

    let pmm = Pmm::new(periph.PMM);
    let p1 = Batch::new(periph.P1)
        .config_pin0(|p| p.to_output())
        .split(&pmm);
    let red_led = p1.pin0;

    with(|cs| unsafe { *RED_LED.borrow(cs).get() = Some(red_led) });

    let (_smclk, aclk, _delay) = ClockConfig::new(periph.CS)
        .mclk_dcoclk(DcoclkFreqSel::_1MHz, MclkDiv::_1)
        .smclk_on(SmclkDiv::_1)
        .aclk_vloclk()
        .freeze(&mut fram);

    let captures = CaptureParts3::config(periph.TB0, TimerConfig::aclk(&aclk))
        .config_cap1_input_A(p1.pin6.to_alternate2())
        .config_cap1_trigger(CapTrigger::FallingEdge)
        .commit();
    let mut capture = captures.cap1;
    let vectors = captures.tbxiv;

    setup_capture(&mut capture);
    with(|cs| {
        unsafe { *CAPTURE.borrow(cs).get() = Some(capture) }
        unsafe { *VECTOR.borrow(cs).get() = Some(vectors) }
    });
    unsafe { enable() };

    loop {}
}

fn setup_capture<T: CapCmp<C>, C>(capture: &mut Capture<T, C>) {
    capture.enable_interrupts();
}

#[interrupt]
fn TIMER0_B1() {
    with(|cs| {
        let Some(vector) = unsafe { &mut *VECTOR.borrow(cs).get() }.as_mut() else { return; };
        let Some(capture) = unsafe { &mut *CAPTURE.borrow(cs).get() }.as_mut() else { return; };
        let Some(led) = unsafe { &mut *RED_LED.borrow(cs).get() }.as_mut() else { return; };

        if let CaptureVector::Capture1(cap) = vector.interrupt_vector() {
            if cap.interrupt_capture(capture).is_ok() {
                led.toggle().void_unwrap();
            }
        };
    });
}

// The compiler will emit calls to the abort() compiler intrinsic if debug assertions are
// enabled (default for dev profile). MSP430 does not actually have meaningful abort() support
// so for now, we create our own in each application where debug assertions are present.
#[no_mangle]
extern "C" fn abort() -> ! {
    panic!();
}
