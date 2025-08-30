#![no_main]
#![no_std]

use embedded_hal::pwm::SetDutyCycle;
use msp430_rt::entry;
use msp430fr2x5x_hal::{
    clock::{ClockConfig, DcoclkFreqSel, MclkDiv, SmclkDiv},
    fram::Fram,
    gpio::Batch,
    pmm::Pmm,
    pwm::{PwmParts7, TimerConfig},
    watchdog::Wdt,
};
use panic_msp430 as _;

// P6.4 LED should be bright, P6.3 LED should be dim
#[entry]
fn main() -> ! {
    let periph = msp430fr2355::Peripherals::take().unwrap();

    let mut fram = Fram::new(periph.FRCTL);
    Wdt::constrain(periph.WDT_A);

    let pmm = Pmm::new(periph.PMM);
    let p6 = Batch::new(periph.P6).split(&pmm);

    let (smclk, _aclk, _delay) = ClockConfig::new(periph.CS)
        .mclk_dcoclk(DcoclkFreqSel::_1MHz, MclkDiv::_1)
        .smclk_on(SmclkDiv::_1)
        .aclk_vloclk()
        .freeze(&mut fram);

    let pwm = PwmParts7::new(periph.TB3, TimerConfig::smclk(&smclk), 5000);
    let mut pwm4 = pwm.pwm4.init(p6.pin3.to_output().to_alternate1());
    let mut pwm5 = pwm.pwm5.init(p6.pin4.to_output().to_alternate1());

    pwm4.set_duty_cycle(100).unwrap();
    pwm5.set_duty_cycle(3795).unwrap();

    loop {}
}

// The compiler will emit calls to the abort() compiler intrinsic if debug assertions are
// enabled (default for dev profile). MSP430 does not actually have meaningful abort() support
// so for now, we create our own in each application where debug assertions are present.
#[no_mangle]
extern "C" fn abort() -> ! {
    panic!();
}
