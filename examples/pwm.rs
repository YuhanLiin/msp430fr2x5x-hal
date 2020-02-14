#![no_main]
#![no_std]

use embedded_hal::prelude::*;
use msp430_rt::entry;
use msp430fr2x5x_hal::{
    clock::{DcoclkFreqSel, MclkDiv, SmclkDiv},
    prelude::*,
    pwm::{CapCmpPeriph, Pwm, PwmGpio, TimerConfig},
};
use panic_msp430 as _;

// P6.4 LED should be bright, P6.3 LED should be dim
#[entry]
fn main() -> ! {
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

    let pwm = periph.TB3.to_pwm(TimerConfig::smclk(&smclk), 5000);
    let mut pwm4 = pwm.pwm4.init(p6.pin3.to_output().to_alternate1());
    let mut pwm5 = pwm.pwm5.init(p6.pin4.to_output().to_alternate1());

    config_pwm(&mut pwm4, 100);
    config_pwm(&mut pwm5, 3795);

    loop {}
}

fn config_pwm<T, C>(pwm: &mut Pwm<T, C>, duty: u16)
where
    (T, C): PwmGpio,
    T: CapCmpPeriph<C>,
{
    pwm.enable();
    pwm.set_duty(duty);
}
