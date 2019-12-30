#![no_std]
#![feature(abi_msp430_interrupt)]

use core::cell::RefCell;
use embedded_hal::digital::v2::ToggleableOutputPin;
use msp430::interrupt::{enable, free, Mutex};
use msp430fr2355::interrupt;
use msp430fr2x5x_hal::{
    capture::{CapSelect, CapThreeChannel, CapTrigger, CapturePort, CaptureVector, TimerConfig},
    clock::{DcoclkFreqSel, MclkDiv, SmclkDiv},
    gpio::*,
    prelude::*,
};
use panic_msp430 as _;
use void::ResultVoidExt;

static CAPTURE: Mutex<RefCell<Option<CapturePort<msp430fr2355::TB0>>>> =
    Mutex::new(RefCell::new(None));
static RED_LED: Mutex<RefCell<Option<Pin<Port1, Pin0, Output>>>> = Mutex::new(RefCell::new(None));

// Connect push button input to P1.6. When button is pressed, red LED should toggle. No debouncing,
// so sometimes inputs are missed.
fn main() {
    let periph = msp430fr2355::Peripherals::take().unwrap();

    let mut fram = periph.FRCTL.constrain();
    periph.WDT_A.constrain();

    let pmm = periph.PMM.freeze();
    let p1 = periph.P1.batch().config_pin0(|p| p.to_output()).split(&pmm);
    let red_led = p1.pin0;

    free(|cs| *RED_LED.borrow(&cs).borrow_mut() = Some(red_led));

    let (_smclk, aclk) = periph
        .CS
        .constrain()
        .mclk_dcoclk(DcoclkFreqSel::_1MHz, MclkDiv::_1)
        .smclk_on(SmclkDiv::_1)
        .aclk_vloclk()
        .freeze(&mut fram);

    let mut capture = periph.TB0.to_capture(TimerConfig::aclk(&aclk));
    const CHAN1: CapThreeChannel = CapThreeChannel::Chan1;
    capture.set_capture_trigger(CHAN1, CapTrigger::FallingEdge);
    capture.set_input_select(CHAN1, CapSelect::InputA);

    p1.pin6.to_alternate2();
    capture.enable_cap_intr(CHAN1);
    free(|cs| *CAPTURE.borrow(&cs).borrow_mut() = Some(capture));
    unsafe { enable() };

    loop {}
}

interrupt!(TIMER0_B1, capture_isr);
fn capture_isr() {
    free(|cs| {
        match CAPTURE
            .borrow(&cs)
            .borrow_mut()
            .as_mut()
            .unwrap()
            .interrupt_vector()
        {
            CaptureVector::Capture1(cap) => {
                if cap.is_ok() {
                    RED_LED
                        .borrow(&cs)
                        .borrow_mut()
                        .as_mut()
                        .unwrap()
                        .toggle()
                        .void_unwrap();
                }
            }
            _ => unreachable!(),
        };
    });
}
