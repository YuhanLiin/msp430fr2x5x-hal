#![no_main]
#![no_std]
#![feature(abi_msp430_interrupt)]
#![feature(asm_experimental_arch)]

// This examples enters LPM4.5, then when a button on P2.3 is pressed the system wakes and flashes the red LED.

use embedded_hal::digital::*;
use msp430::asm::nop;
use msp430_rt::entry;
use msp430fr2355::{P3, P4, P5, P6};
use msp430fr2x5x_hal::{gpio::Batch, lpm::{enter_lpm4_5, SvsState}, pmm::Pmm, watchdog::Wdt};
use panic_msp430 as _;

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

#[entry]
fn main() -> ! {
    let periph = msp430fr2355::Peripherals::take().unwrap();

    let wdt = Wdt::constrain(periph.WDT_A);
    let pmm = Pmm::new(periph.PMM);

    // Floating input pins consume a *huge* amount of power (relatively speaking).
    // Set unused pins to outputs or enable their pull resistors.
    let port1 = init_port_as_pulldowns!(periph.P1)
        .config_pin0(|p| p.to_output())
        .split(&pmm);
    let mut red_led = port1.pin0;

    let port2 = init_port_as_pulldowns!(periph.P2)
        .config_pin3(|p| p.pullup())
        .split(&pmm);

    init_unused_gpio(periph.P3, periph.P4, periph.P5, periph.P6, &pmm);

    // If this reset was a wake up from LPMx.5...
    if periph.SYS.sysrstiv.read().sysrstiv().is_lpm5wu() {
        loop {
            for _ in 0..10_000 {
                nop();
            }
            red_led.toggle().ok();
        }
    }
    // Otherwise it was a regular reset. Prepare to enter LPM4.5.
    else {
        // Configure P2.3 for interrupts
        let mut button = port2.pin3;
        button.select_falling_edge_trigger().enable_interrupts();

        // And enter LPM4.5. Global interrupts are enabled before LPM4.5 is entered.
        enter_lpm4_5(wdt, periph.RTC, SvsState::SVSHE_0);
    }
}

/// Enable pulldowns on unused ports to massively reduce power usage.
fn init_unused_gpio(p3: P3, p4: P4, p5: P5, p6: P6, pmm: &Pmm) {
    init_port_as_pulldowns!(p3).split(pmm);
    init_port_as_pulldowns!(p4).split(pmm);
    init_port_as_pulldowns!(p5).split(pmm);
    init_port_as_pulldowns!(p6).split(pmm);
}

// Note: In this case we don't need an ISR when waking from LPMx.5, since power on disables interrupts.
// You *can* service the interrupt that causes the wakeup, but this isn't done here.

// The compiler will emit calls to the abort() compiler intrinsic if debug assertions are
// enabled (default for dev profile). MSP430 does not actually have meaningful abort() support
// so for now, we create our own in each application where debug assertions are present.
#[no_mangle]
extern "C" fn abort() -> ! {
    panic!();
}
