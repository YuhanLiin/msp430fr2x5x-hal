//! PWM ports
//!
//! PWM ports are created from timers. TB0, TB1, and TB2 create 2-channel ports and TB3 create
//! 6-channel ports. Each channel has its own configurable duty cycle, but share the same period as
//! other channels in the same port.

use crate::hw_traits::timerb::{
    CCRn, Outmod, TimerB, TimerSteal, CCR0, CCR1, CCR2, CCR3, CCR4, CCR5, CCR6,
};
use crate::timer::{SevenCCRnTimer, ThreeCCRnTimer, TimerPeriph};
use core::marker::PhantomData;
use embedded_hal::PwmPin;
use msp430fr2355 as pac;

pub use crate::timer::{TimerConfig, TimerDiv, TimerExDiv};

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
        let timer = unsafe { Self::steal() };
        CCRn::<CCR1>::config_outmod(timer, Outmod::ResetSet);
        CCRn::<CCR2>::config_outmod(timer, Outmod::ResetSet);
    }
}

impl PwmConfigChannels for Tb1 {
    #[inline]
    fn config_channels(&self) {
        let timer = unsafe { Self::steal() };
        CCRn::<CCR1>::config_outmod(timer, Outmod::ResetSet);
        CCRn::<CCR2>::config_outmod(timer, Outmod::ResetSet);
    }
}

impl PwmConfigChannels for Tb2 {
    #[inline]
    fn config_channels(&self) {
        let timer = unsafe { Self::steal() };
        CCRn::<CCR1>::config_outmod(timer, Outmod::ResetSet);
        CCRn::<CCR2>::config_outmod(timer, Outmod::ResetSet);
    }
}

impl PwmConfigChannels for Tb3 {
    #[inline]
    fn config_channels(&self) {
        let timer = unsafe { Self::steal() };
        CCRn::<CCR1>::config_outmod(timer, Outmod::ResetSet);
        CCRn::<CCR2>::config_outmod(timer, Outmod::ResetSet);
        CCRn::<CCR3>::config_outmod(timer, Outmod::ResetSet);
        CCRn::<CCR4>::config_outmod(timer, Outmod::ResetSet);
        CCRn::<CCR5>::config_outmod(timer, Outmod::ResetSet);
        CCRn::<CCR6>::config_outmod(timer, Outmod::ResetSet);
    }
}

//trait PwmGpio {
//type Pin;
//}

//impl PwmGpio for (Tb0, CCR0) {
//type Pin = Pwm
//}

/// Collection of PWM channels derived from timer peripheral with 3 capture-compare registers
pub struct ThreeCCRnPins<T: ThreeCCRnTimer> {
    /// PWM pin 1 (derived from capture-compare register 1)
    pub pwm1: Pwm<T, CCR1>,
    /// PWM pin 2 (derived from capture-compare register 2)
    pub pwm2: Pwm<T, CCR2>,
}

impl<T: ThreeCCRnTimer> Default for ThreeCCRnPins<T> {
    fn default() -> Self {
        Self {
            pwm1: Default::default(),
            pwm2: Default::default(),
        }
    }
}

/// Collection of PWM channels derived from timer peripheral with 7 capture-compare registers
pub struct SevenCCRnPins<T: SevenCCRnTimer> {
    /// PWM pin 1 (derived from capture-compare register 1)
    pub pwm1: Pwm<T, CCR1>,
    /// PWM pin 2 (derived from capture-compare register 2)
    pub pwm2: Pwm<T, CCR2>,
    /// PWM pin 3 (derived from capture-compare register 3)
    pub pwm3: Pwm<T, CCR3>,
    /// PWM pin 4 (derived from capture-compare register 4)
    pub pwm4: Pwm<T, CCR4>,
    /// PWM pin 5 (derived from capture-compare register 5)
    pub pwm5: Pwm<T, CCR5>,
    /// PWM pin 6 (derived from capture-compare register 6)
    pub pwm6: Pwm<T, CCR6>,
}

impl<T: SevenCCRnTimer> Default for SevenCCRnPins<T> {
    fn default() -> Self {
        Self {
            pwm1: Default::default(),
            pwm2: Default::default(),
            pwm3: Default::default(),
            pwm4: Default::default(),
            pwm5: Default::default(),
            pwm6: Default::default(),
        }
    }
}

//pub struct PwmUninit<T: CCRn<C>, C>(PhantomData<T>, PhantomData<C>);

/// A single PWM channel
pub struct Pwm<T: CCRn<C>, C>(PhantomData<T>, PhantomData<C>);

impl<T: TimerPeriph + CCRn<C>, C> Default for Pwm<T, C> {
    fn default() -> Self {
        Self(PhantomData, PhantomData)
    }
}

/// Extension trait for creating PWM pins from timer peripherals
pub trait PwmExt: Sized {
    #[doc(hidden)]
    type Timer: TimerPeriph + PwmConfigChannels + CCRn<CCR0>;
    /// Collection of PWM pins
    type Pins: Default;

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

impl<T: CCRn<CCR0> + CCRn<C>, C> PwmPin for Pwm<T, C> {
    /// Number of cycles
    type Duty = u16;

    #[inline]
    /// Sets duty cycle of the PWM. Has no effect while PWM is disabled.
    fn set_duty(&mut self, duty: Self::Duty) {
        let timer = unsafe { T::steal() };
        if !self.is_disabled() {
            CCRn::<C>::set_ccrn(timer, duty);
        }
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
        let timer = unsafe { T::steal() };
        // Set duty cycle to 0
        CCRn::<C>::set_ccrn(timer, 0);
    }

    #[inline]
    fn enable(&mut self) {
        let timer = unsafe { T::steal() };
        if self.is_disabled() {
            // Set duty cycle to 1
            CCRn::<C>::set_ccrn(timer, 1);
        }
    }
}

impl<T: CCRn<C>, C> Pwm<T, C> {
    /// Check whether the PWM is disabled or not
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        let timer = unsafe { T::steal() };
        timer.get_ccrn() == 0
    }
}
