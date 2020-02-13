//! PWM ports
//!
//! PWM pins are created from timer peripherals. TB0, TB1, and TB2 each create 2 PWMs and TB3
//! creates 6 PWMs. Each PWM has its own duty cycle and and GPIO pin, and all PWMs from the same timer
//! share the same period.

use crate::gpio::{
    Alternate1, Alternate2, ChangeSelectBits, Output, Pin, Pin0, Pin1, Pin2, Pin3, Pin4, Pin5,
    Pin6, Pin7, Port1, Port2, Port5, Port6,
};
use crate::hw_traits::timerb::{
    CCRn, Outmod, TimerB, TimerSteal, CCR0, CCR1, CCR2, CCR3, CCR4, CCR5, CCR6,
};
use crate::timer::{SevenCCRnTimer, ThreeCCRnTimer};
use crate::util::SealedDefault;
use core::marker::PhantomData;
use embedded_hal::PwmPin;
use msp430fr2355 as pac;

pub use crate::timer::{CapCmpPeriph, TimerConfig, TimerDiv, TimerExDiv, TimerPeriph};

mod sealed {
    use super::*;

    pub trait SealedPwmGpio {}
    pub trait SealedPwmExt {}

    impl SealedPwmGpio for (Tb0, CCR1) {}
    impl SealedPwmGpio for (Tb0, CCR2) {}
    impl SealedPwmGpio for (Tb1, CCR1) {}
    impl SealedPwmGpio for (Tb1, CCR2) {}
    impl SealedPwmGpio for (Tb2, CCR1) {}
    impl SealedPwmGpio for (Tb2, CCR2) {}
    impl SealedPwmGpio for (Tb3, CCR1) {}
    impl SealedPwmGpio for (Tb3, CCR2) {}
    impl SealedPwmGpio for (Tb3, CCR3) {}
    impl SealedPwmGpio for (Tb3, CCR4) {}
    impl SealedPwmGpio for (Tb3, CCR5) {}
    impl SealedPwmGpio for (Tb3, CCR6) {}

    impl SealedPwmExt for pac::TB0 {}
    impl SealedPwmExt for pac::TB1 {}
    impl SealedPwmExt for pac::TB2 {}
    impl SealedPwmExt for pac::TB3 {}
}

// This trait applies to RegisterBlock types rather than peripheral types, so users likely won't
// use the trait in their own code, so it can stay hidden
#[doc(hidden)]
pub trait PwmConfigChannels {
    // Configures each PWM channel of a peripheral during init
    fn config_channels(&self);
}

type Tb0 = pac::tb0::RegisterBlock;
type Tb1 = pac::tb1::RegisterBlock;
type Tb2 = pac::tb2::RegisterBlock;
type Tb3 = pac::tb3::RegisterBlock;

impl PwmConfigChannels for Tb0 {
    #[inline]
    fn config_channels(&self) {
        CCRn::<CCR1>::config_outmod(self, Outmod::ResetSet);
        CCRn::<CCR2>::config_outmod(self, Outmod::ResetSet);
    }
}

impl PwmConfigChannels for Tb1 {
    #[inline]
    fn config_channels(&self) {
        CCRn::<CCR1>::config_outmod(self, Outmod::ResetSet);
        CCRn::<CCR2>::config_outmod(self, Outmod::ResetSet);
    }
}

impl PwmConfigChannels for Tb2 {
    #[inline]
    fn config_channels(&self) {
        CCRn::<CCR1>::config_outmod(self, Outmod::ResetSet);
        CCRn::<CCR2>::config_outmod(self, Outmod::ResetSet);
    }
}

impl PwmConfigChannels for Tb3 {
    #[inline]
    fn config_channels(&self) {
        CCRn::<CCR1>::config_outmod(self, Outmod::ResetSet);
        CCRn::<CCR2>::config_outmod(self, Outmod::ResetSet);
        CCRn::<CCR3>::config_outmod(self, Outmod::ResetSet);
        CCRn::<CCR4>::config_outmod(self, Outmod::ResetSet);
        CCRn::<CCR5>::config_outmod(self, Outmod::ResetSet);
        CCRn::<CCR6>::config_outmod(self, Outmod::ResetSet);
    }
}

#[doc(hidden)]
pub enum Alt {
    Alt1,
    Alt2,
}

/// Associates PWM pins with specific GPIO pins
pub trait PwmGpio: sealed::SealedPwmGpio {
    /// GPIO type
    type Gpio: ChangeSelectBits;
    #[doc(hidden)]
    const ALT: Alt;

    #[doc(hidden)]
    fn to_alt(pin: &mut Self::Gpio) {
        match Self::ALT {
            Alt::Alt1 => pin.set_sel0(),
            Alt::Alt2 => pin.set_sel1(),
        }
    }

    #[doc(hidden)]
    fn to_gpio(pin: &mut Self::Gpio) {
        match Self::ALT {
            Alt::Alt1 => pin.clear_sel0(),
            Alt::Alt2 => pin.clear_sel1(),
        }
    }
}

impl PwmGpio for (Tb0, CCR1) {
    type Gpio = Pin<Port1, Pin6, Alternate2<Output>>;
    const ALT: Alt = Alt::Alt2;
}
impl PwmGpio for (Tb0, CCR2) {
    type Gpio = Pin<Port1, Pin7, Alternate2<Output>>;
    const ALT: Alt = Alt::Alt2;
}

impl PwmGpio for (Tb1, CCR1) {
    type Gpio = Pin<Port2, Pin0, Alternate1<Output>>;
    const ALT: Alt = Alt::Alt1;
}
impl PwmGpio for (Tb1, CCR2) {
    type Gpio = Pin<Port2, Pin1, Alternate1<Output>>;
    const ALT: Alt = Alt::Alt1;
}

impl PwmGpio for (Tb2, CCR1) {
    type Gpio = Pin<Port5, Pin0, Alternate1<Output>>;
    const ALT: Alt = Alt::Alt1;
}
impl PwmGpio for (Tb2, CCR2) {
    type Gpio = Pin<Port5, Pin1, Alternate1<Output>>;
    const ALT: Alt = Alt::Alt1;
}

impl PwmGpio for (Tb3, CCR1) {
    type Gpio = Pin<Port6, Pin0, Alternate1<Output>>;
    const ALT: Alt = Alt::Alt1;
}
impl PwmGpio for (Tb3, CCR2) {
    type Gpio = Pin<Port6, Pin1, Alternate1<Output>>;
    const ALT: Alt = Alt::Alt1;
}
impl PwmGpio for (Tb3, CCR3) {
    type Gpio = Pin<Port6, Pin2, Alternate1<Output>>;
    const ALT: Alt = Alt::Alt1;
}
impl PwmGpio for (Tb3, CCR4) {
    type Gpio = Pin<Port6, Pin3, Alternate1<Output>>;
    const ALT: Alt = Alt::Alt1;
}
impl PwmGpio for (Tb3, CCR5) {
    type Gpio = Pin<Port6, Pin4, Alternate1<Output>>;
    const ALT: Alt = Alt::Alt1;
}
impl PwmGpio for (Tb3, CCR6) {
    type Gpio = Pin<Port6, Pin5, Alternate1<Output>>;
    const ALT: Alt = Alt::Alt1;
}

/// Collection of uninitialized PWM pins derived from timer peripheral with 3 capture-compare registers
pub struct ThreeCCRnPins<T: ThreeCCRnTimer> {
    /// PWM pin 1 (derived from capture-compare register 1)
    pub pwm1: PwmUninit<T, CCR1>,
    /// PWM pin 2 (derived from capture-compare register 2)
    pub pwm2: PwmUninit<T, CCR2>,
}

impl<T: ThreeCCRnTimer> SealedDefault for ThreeCCRnPins<T> {
    fn default() -> Self {
        Self {
            pwm1: SealedDefault::default(),
            pwm2: SealedDefault::default(),
        }
    }
}

/// Collection of uninitialized PWM pins derived from timer peripheral with 7 capture-compare registers
pub struct SevenCCRnPins<T: SevenCCRnTimer> {
    /// PWM pin 1 (derived from capture-compare register 1)
    pub pwm1: PwmUninit<T, CCR1>,
    /// PWM pin 2 (derived from capture-compare register 2)
    pub pwm2: PwmUninit<T, CCR2>,
    /// PWM pin 3 (derived from capture-compare register 3)
    pub pwm3: PwmUninit<T, CCR3>,
    /// PWM pin 4 (derived from capture-compare register 4)
    pub pwm4: PwmUninit<T, CCR4>,
    /// PWM pin 5 (derived from capture-compare register 5)
    pub pwm5: PwmUninit<T, CCR5>,
    /// PWM pin 6 (derived from capture-compare register 6)
    pub pwm6: PwmUninit<T, CCR6>,
}

impl<T: SevenCCRnTimer> SealedDefault for SevenCCRnPins<T> {
    fn default() -> Self {
        Self {
            pwm1: SealedDefault::default(),
            pwm2: SealedDefault::default(),
            pwm3: SealedDefault::default(),
            pwm4: SealedDefault::default(),
            pwm5: SealedDefault::default(),
            pwm6: SealedDefault::default(),
        }
    }
}

/// Uninitialzied PWM pin
pub struct PwmUninit<T, C>(PhantomData<T>, PhantomData<C>);

impl<T: CapCmpPeriph<C>, C> PwmUninit<T, C>
where
    (T, C): PwmGpio,
{
    /// Initialized the PWM pin by passing in the appropriately configured GPIO pin
    pub fn init(self, pin: <(T, C) as PwmGpio>::Gpio) -> Pwm<T, C> {
        Pwm {
            _timer: PhantomData,
            _ccrn: PhantomData,
            pin,
        }
    }
}

impl<T, C> SealedDefault for PwmUninit<T, C> {
    fn default() -> Self {
        Self(PhantomData, PhantomData)
    }
}

/// A single PWM pin
pub struct Pwm<T: CapCmpPeriph<C>, C>
where
    (T, C): PwmGpio,
{
    _timer: PhantomData<T>,
    _ccrn: PhantomData<C>,
    pin: <(T, C) as PwmGpio>::Gpio,
}

/// Extension trait for creating PWM pins from timer peripherals
pub trait PwmExt: Sized + sealed::SealedPwmExt {
    #[doc(hidden)]
    type Timer: TimerPeriph + PwmConfigChannels;
    /// Collection of PWM pins
    type Pins: SealedDefault;

    /// Create new PWM pins out of timer
    #[inline]
    fn to_pwm(self, config: TimerConfig<Self::Timer>, period: u16) -> Self::Pins {
        let timer = unsafe { Self::Timer::steal() };
        config.write_regs(&timer);
        CCRn::<CCR0>::set_ccrn(timer, period);
        CCRn::<CCR0>::config_outmod(timer, Outmod::Toggle);
        timer.config_channels();
        // Start the timer to run PWM
        timer.upmode();
        Self::Pins::default()
    }
}

impl PwmExt for pac::TB0 {
    type Timer = Tb0;
    type Pins = ThreeCCRnPins<Self::Timer>;
}
impl PwmExt for pac::TB1 {
    type Timer = Tb1;
    type Pins = ThreeCCRnPins<Self::Timer>;
}
impl PwmExt for pac::TB2 {
    type Timer = Tb2;
    type Pins = ThreeCCRnPins<Self::Timer>;
}
impl PwmExt for pac::TB3 {
    type Timer = Tb3;
    type Pins = SevenCCRnPins<Self::Timer>;
}

impl<T: CapCmpPeriph<CCR0> + CapCmpPeriph<C>, C> PwmPin for Pwm<T, C>
where
    (T, C): PwmGpio,
{
    /// Number of cycles
    type Duty = u16;

    #[inline]
    fn set_duty(&mut self, duty: Self::Duty) {
        let timer = unsafe { T::steal() };
        CCRn::<C>::set_ccrn(timer, duty);
    }

    #[inline]
    fn get_duty(&self) -> Self::Duty {
        let timer = unsafe { T::steal() };
        CCRn::<C>::get_ccrn(timer)
    }

    /// Maximum valid duty is equal to the period. If number of duty cycles exceeds number of
    /// period cycles, then signal stays high (equivalent to 100% duty cycle).
    #[inline]
    fn get_max_duty(&self) -> Self::Duty {
        let timer = unsafe { T::steal() };
        CCRn::<CCR0>::get_ccrn(timer)
    }

    #[inline]
    fn disable(&mut self) {
        <(T, C) as PwmGpio>::to_gpio(&mut self.pin);
    }

    #[inline]
    fn enable(&mut self) {
        <(T, C) as PwmGpio>::to_alt(&mut self.pin);
    }
}
