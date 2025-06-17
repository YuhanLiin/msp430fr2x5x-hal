#![no_main]
#![no_std]

use embedded_hal::digital::*;
use msp430_rt::entry;
use msp430fr2x5x_hal::{
    adc::{AdcConfig, ClockDivider, Predivider, Resolution, SampleTime, SamplingRate},
    gpio::Batch,
    pmm::Pmm,
    watchdog::Wdt,
};
use nb::block;
use panic_msp430 as _;

// If pin 1.1 is between 1V and 2V, the LED on pin 1.0 should light up.
#[entry]
fn main() -> ! {
    // Take peripherals and disable watchdog
    let periph = msp430fr2355::Peripherals::take().unwrap();
    let _wdt = Wdt::constrain(periph.WDT_A);

    // Configure GPIO
    let pmm = Pmm::new(periph.PMM);
    let port1 = Batch::new(periph.P1).split(&pmm);
    let mut led = port1.pin0.to_output();
    let mut adc_pin = port1.pin1.to_alternate3();

    // ADC setup
    let mut adc = AdcConfig::new(
        ClockDivider::_1,
        Predivider::_1,
        Resolution::_8BIT,
        SamplingRate::_50KSPS,
        SampleTime::_4,
    )
    .use_modclk()
    .configure(periph.ADC);

    loop {
        // Get ADC voltage, assuming the ADC reference voltage is 3300mV
        // It's infallible besides nb::WouldBlock, so it's safe to unwrap after block!()
        // If you want a raw count use adc.read_count() instead.
        let reading_mv = block!( adc.read_voltage_mv(&mut adc_pin, 3300) ).unwrap();

        // Turn on LED if voltage between 1000 and 2000mV
        if (1000..=2000).contains(&reading_mv) {
            led.set_high().ok();
        } else {
            led.set_low().ok();
        }
    }
}

// The compiler will emit calls to the abort() compiler intrinsic if debug assertions are
// enabled (default for dev profile). MSP430 does not actually have meaningful abort() support
// so for now, we create our own in each application where debug assertions are present.
#[no_mangle]
extern "C" fn abort() -> ! {
    panic!();
}
