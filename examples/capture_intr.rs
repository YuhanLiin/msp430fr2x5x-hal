#![no_std]
#![feature(abi_msp430_interrupt)]

use core::cell::RefCell;
use embedded_hal::digital::v2::ToggleableOutputPin;
use msp430::interrupt::{enable, free, Mutex};
use msp430fr2355::interrupt;
use msp430fr2x5x_hal::{
    capture::{
        CapCmpPeriph, CapSelect, CapTrigger, Capture, CaptureConfig, CaptureVector, TBxIV,
        TimerConfig,
    },
    clock::{DcoclkFreqSel, MclkDiv, SmclkDiv},
    gpio::*,
    prelude::*,
    timer::CCR1,
};
use panic_msp430 as _;
use void::ResultVoidExt;

static CAPTURE: Mutex<RefCell<Option<Capture<msp430fr2355::tb0::RegisterBlock, CCR1>>>> =
    Mutex::new(RefCell::new(None));
static VECTOR: Mutex<RefCell<Option<TBxIV<msp430fr2355::tb0::RegisterBlock>>>> =
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

    let captures = periph.TB0.to_capture(
        TimerConfig::aclk(&aclk),
        CaptureConfig::new().config_capture1(CapSelect::CapInputA, CapTrigger::FallingEdge),
    );
    let mut capture = captures.cap1;
    let vectors = captures.tbxiv;

    p1.pin6.to_alternate2();
    setup_capture(&mut capture);
    free(|cs| {
        *CAPTURE.borrow(&cs).borrow_mut() = Some(capture);
        *VECTOR.borrow(&cs).borrow_mut() = Some(vectors)
    });
    unsafe { enable() };

    loop {}
}

fn setup_capture<T: CapCmpPeriph<C>, C>(capture: &mut Capture<T, C>) {
    capture.enable_interrupts();
}

interrupt!(TIMER0_B1, capture_isr);
fn capture_isr() {
    free(|cs| {
        match VECTOR
            .borrow(&cs)
            .borrow_mut()
            .as_mut()
            .unwrap()
            .interrupt_vector()
        {
            CaptureVector::Capture1(cap) => {
                if cap
                    .interrupt_capture(CAPTURE.borrow(&cs).borrow_mut().as_mut().unwrap())
                    .is_ok()
                {
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
