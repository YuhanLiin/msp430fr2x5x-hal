#![no_main]
#![no_std]

use embedded_hal::{delay::DelayNs, pwm::SetDutyCycle};
use msp430_rt::entry;
use msp430fr2x5x_hal::{
    clock::{ClockConfig, DcoclkFreqSel, MclkDiv, SmclkDiv},
    fram::Fram,
    gpio::Batch,
    pmm::Pmm,
    pwm::{PwmParts3, TimerConfig},
    watchdog::Wdt,
};
use panic_msp430 as _;

// P1.1 LED should breathe from 0 to 100% brightness
#[entry]
fn main() -> ! {
    let periph = msp430fr2433::Peripherals::take().unwrap();

    let mut fram = Fram::new(periph.fram);
    Wdt::constrain(periph.watchdog_timer);

    let pmm = Pmm::new(periph.pmm);
    let p1 = Batch::new(periph.p1).split(&pmm);

    let (smclk, _aclk, mut delay) = ClockConfig::new(periph.cs)
        .mclk_dcoclk(DcoclkFreqSel::_1MHz, MclkDiv::_1)
        .smclk_on(SmclkDiv::_1)
        .aclk_refoclk()
        .freeze(&mut fram);

    let pwm = PwmParts3::new(periph.timer_0_a3, TimerConfig::smclk(&smclk), 5000);
    let mut pwm1 = pwm.pwm1.init(p1.pin1.to_output().to_alternate2());

    loop {
        for percent in (0..=100).chain((0..100).rev()) {
            pwm1.set_duty_cycle_percent(percent);
            delay.delay_ms(5);
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
