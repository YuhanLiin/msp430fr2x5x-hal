#![no_main]
#![no_std]

use embedded_hal::digital::*;
use msp430::asm;
use msp430_rt::entry;
use msp430fr2x5x_hal::{gpio::Batch, info_mem::InfoMemory, pmm::Pmm, watchdog::Wdt};
use panic_msp430 as _;

// Use the non-volatile information memory to toggle the red onboard LED.
// Resetting or power cycling the board toggles the red LED.

#[entry]
fn main() -> ! {
    // Take peripherals
    let periph = msp430fr2355::Peripherals::take().unwrap();
    let _wdt = Wdt::constrain(periph.WDT_A);

    // Configure GPIO
    let pmm = Pmm::new(periph.PMM);
    let mut led = Batch::new(periph.P1).split(&pmm).pin0.to_output();

    // Wait a little bit to 'debounce' any power cycles.
    for _ in 0..100 {
        asm::nop();
    }

    // Disable write protection and get the information memory as an array type
    let (nv_mem, _) = InfoMemory::as_u8s(periph.SYS);

    // Toggle the first byte between 0 and 1.
    nv_mem[0] = (nv_mem[0].wrapping_add(1)) & 1;

    // Turn the LED on if 0
    led.set_state((nv_mem[0] == 0).into()).ok();

    loop {
        msp430::asm::nop();
    }
}

// The compiler will emit calls to the abort() compiler intrinsic if debug assertions are
// enabled (default for dev profile). MSP430 does not actually have meaningful abort() support
// so for now, we create our own in each application where debug assertions are present.
#[no_mangle]
extern "C" fn abort() -> ! {
    panic!();
}
