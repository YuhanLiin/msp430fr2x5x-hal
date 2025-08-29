#![no_main]
#![no_std]
#![feature(abi_msp430_interrupt)]
#![feature(asm_experimental_arch)]

use embedded_hal::digital::*;
use msp430_rt::entry;
use msp430fr2355::{P2, P3, P4, P5, P6};
use msp430fr2x5x_hal::{ bak_mem::BackupMemory, clock::VLOCLK, gpio::Batch, lpm::{enter_lpm3_5, enter_lpm3_5_unchecked, SvsState}, pmm::Pmm, rtc::{Rtc, RtcDiv}, watchdog::Wdt
};
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

// The RTC will wake the board every second. LED state is stored in and loaded from the backup memory.
// When programming with mspdebug you need to unplug and replug the board for the example to work, for some reason. 
// Programming via Uniflash or Code Composer Studio works fine.
#[entry]
fn main() -> ! {
    let periph = msp430fr2355::Peripherals::take().unwrap();
    
    let wdt = Wdt::constrain(periph.WDT_A);
    let pmm = Pmm::new(periph.PMM);
    
    // Floating input pins consume a *huge* amount of energy (relatively speaking).
    // Set unused pins to outputs or enable their pull resistors.
    let port1 = init_port_as_pulldowns!(periph.P1)
        .config_pin0(|p| p.to_output())
        .split(&pmm);
    let mut red_led = port1.pin0;

    init_unused_gpio(periph.P2, periph.P3, periph.P4, periph.P5, periph.P6, &pmm);

    // If this reset was a wake up from LPMx.5...
    if periph.SYS.sysrstiv.read().sysrstiv().is_lpm5wu() {
        // Toggle the LED.
        // I/O registers have their values reset coming out of LPMx.5, 
        // so we have to store state in the backup memory.
        let bak_mem = BackupMemory::as_u8s(periph.BKMEM);

        let old_value = bak_mem[0] == 1;
        red_led.set_state(old_value.into()).ok();

        let new_value = if old_value {0} else {1};
        bak_mem[0] = new_value;

        // Clear RTC interrupt flag
        periph.RTC.rtciv.read();

        // Enter LPM3.5 (without having to configure the RTC, we did that already).
        unsafe { enter_lpm3_5_unchecked(wdt, SvsState::SVSHE_0) };
    }
    // Otherwise this is a fresh start. Configure the RTC.
    else {
        // Configure RTC for 1 Hz interrupt
        let mut rtc = Rtc::new(periph.RTC).use_vloclk();
        rtc.set_clk_div(RtcDiv::_1);
        rtc.start(VLOCLK); // Count up to VLOCLK freq -> 1 Hz period
        rtc.enable_interrupts();
        // Global interrupts are enabled by `enter_lpm3_5()`
        // Leaving LPMx.5 requires a full system reset, so this function will never return.
        enter_lpm3_5(wdt, rtc, SvsState::SVSHE_0);
    }
}

/// Enable pulldowns on unused ports to massively reduce power usage.
fn init_unused_gpio(p2: P2, p3: P3, p4: P4, p5: P5, p6: P6, pmm: &Pmm) {
    init_port_as_pulldowns!(p2).split(pmm);
    init_port_as_pulldowns!(p3).split(pmm);
    init_port_as_pulldowns!(p4).split(pmm);
    init_port_as_pulldowns!(p5).split(pmm);
    init_port_as_pulldowns!(p6).split(pmm);
}

// Note: In this case we don't need an ISR when waking from LPMx.5, since power on disables interrupts
// and we clear the RTC interrupt flag before re-enabling interrupts.
// You *can* service the interrupt that causes the wakeup, but this isn't done here.

// The compiler will emit calls to the abort() compiler intrinsic if debug assertions are
// enabled (default for dev profile). MSP430 does not actually have meaningful abort() support
// so for now, we create our own in each application where debug assertions are present.
#[no_mangle]
extern "C" fn abort() -> ! {
    panic!();
}
