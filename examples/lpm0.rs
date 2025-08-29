#![no_main]
#![no_std]
#![feature(abi_msp430_interrupt)]
#![feature(asm_experimental_arch)]

// NOTE: This example relies on the new wake-cpu feature recently added to the msp430-rt crate to return the CPU to active mode
// after the interrupt returns. This depends on Rust 1.88+. For a version compatible with the MSRV of this crate see lpm0_msrv.rs

use critical_section::with;
use msp430fr2355::{interrupt, P2, P3, P4, P5, P6};

use core::cell::RefCell;
use embedded_hal::digital::*;
use msp430::{asm, interrupt::{enable as enable_interrupts, Mutex}};
use msp430_rt::entry;
use msp430fr2x5x_hal::{
    gpio::{Batch, GpioVector, PxIV}, lpm::enter_lpm0, pmm::Pmm, watchdog::Wdt
};
use panic_msp430 as _;

static P2IV: Mutex<RefCell<Option< PxIV<P2> >>> = Mutex::new(RefCell::new(None));

macro_rules! init_port_as_pulldowns {
    ($port: expr) => {
        Batch::new($port)
            .config_pin0(|p| p.pulldown())
            .config_pin1(|p| p.pulldown())
            .config_pin2(|p| p.pulldown())
            .config_pin3(|p| p.pulldown())
            .config_pin4(|p| p.pulldown())
            .config_pin5(|p| p.pulldown())
            .config_pin6(|p| p.pulldown())
            .config_pin7(|p| p.pulldown())
    };
}

// P1.0 should toggle when P2.3 is pressed
#[entry]
fn main() -> ! {
    let periph = msp430fr2355::Peripherals::take().unwrap();

    let _wdt = Wdt::constrain(periph.WDT_A);
    let pmm = Pmm::new(periph.PMM);

    // Floating input pins consume a *huge* amount of power (relatively speaking).
    // Set unused pins to outputs or enable their pull resistors.
    let p1 = init_port_as_pulldowns!(periph.P1)
        .config_pin0(|p| p.to_output())
        .split(&pmm);
    let mut red_led = p1.pin0;

    let p2 = init_port_as_pulldowns!(periph.P2)
        .config_pin3(|p| p.pullup())
        .split(&pmm);
    let mut button = p2.pin3;
    let p2iv = p2.pxiv;

    init_unused_gpio(periph.P3, periph.P4, periph.P5, periph.P6, &pmm);

    with(|cs| {
        P2IV.borrow_ref_mut(cs).replace(p2iv);
    });

    button.select_falling_edge_trigger().enable_interrupts();

    unsafe { enable_interrupts() };

    loop {
        // Since no peripherals were configured to use SMCLK / ACLK we could just as well enter LPM3 / LPM4 here
        enter_lpm0();
        red_led.toggle().ok();

        for _ in 0..15_000 { // Debouncing
            asm::nop();
        }
    }
}

// Interrupt handlers with the `wake_cpu` argument will set the MSP430 back to Active Mode after the interrupt completes.
#[interrupt(wake_cpu)]
fn PORT2() {
    with(|cs| {
        let Some(ref mut p2iv) = *P2IV.borrow_ref_mut(cs) else {return};
        if let GpioVector::Pin3Isr = p2iv.get_interrupt_vector() {
            // Button pressed
        }
    });
}

/// Enable pulldowns on unused ports to massively reduce power usage.
fn init_unused_gpio(p3: P3, p4: P4, p5: P5, p6: P6, pmm: &Pmm) {
    init_port_as_pulldowns!(p3).split(pmm);
    init_port_as_pulldowns!(p4).split(pmm);
    init_port_as_pulldowns!(p5).split(pmm);
    init_port_as_pulldowns!(p6).split(pmm);
}

// The compiler will emit calls to the abort() compiler intrinsic if debug assertions are
// enabled (default for dev profile). MSP430 does not actually have meaningful abort() support
// so for now, we create our own in each application where debug assertions are present.
#[no_mangle]
extern "C" fn abort() -> ! {
    panic!();
}
