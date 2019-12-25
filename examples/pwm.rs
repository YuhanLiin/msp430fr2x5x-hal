#![no_std]

extern crate panic_msp430;

use embedded_hal::prelude::*;
use msp430fr2x5x_hal::{
    clock::{DcoclkFreqSel, MclkDiv, SmclkDiv},
    prelude::*,
    pwm::{PwmSixChannel, TimerConfig},
};

// P6.4 LED should be bright, P6.3 LED should be dim
fn main() {
    let periph = msp430fr2355::Peripherals::take().unwrap();

    let mut fram = periph.FRCTL.constrain();
    periph.WDT_A.constrain();

    let pmm = periph.PMM.freeze();
    let p6 = periph.P6.batch().split(&pmm);

    let (smclk, _aclk) = periph
        .CS
        .constrain()
        .mclk_dcoclk(DcoclkFreqSel::_1MHz, MclkDiv::_1)
        .smclk_on(SmclkDiv::_1)
        .aclk_vloclk()
        .freeze(&mut fram);

    let mut pwm = periph.TB3.to_pwm(TimerConfig::smclk(&smclk));

    pwm.set_period(5000u16);
    pwm.set_duty(PwmSixChannel::Chan5, 100);
    pwm.set_duty(PwmSixChannel::Chan4, 795);
    pwm.start_all();

    p6.pin4.to_output().to_alternate1();
    p6.pin3.to_output().to_alternate1();

    loop {}
}
