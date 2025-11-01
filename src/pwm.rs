//! PWM ports
//!
//! Configures the board's TimerB peripherals into PWM ports. Each PWM port consists of multiple PWM
//! pins which all share the same period but have their own duty cycles.
//!
//! Each PWM pin starts off in an "uninitialized" state and must be initialized by passing in the
//! appropriate alternate-function GPIO pin. Only initialized pins can be used for PWM.

use crate::gpio::{
    Alternate1, Alternate2, ChangeSelectBits, Output, Pin, Pin0, Pin1, Pin2, Pin3, Pin4, Pin5,
    Pin6, Pin7, P1, P2, P5, P6,
};
use crate::hw_traits::timerb::{CCRn, Outmod};
use crate::timer::{CapCmpTimer3, CapCmpTimer7};
use core::marker::PhantomData;
use crate::pac;

pub use crate::timer::{
    CapCmp, TimerConfig, TimerDiv, TimerExDiv, TimerPeriph, CCR0, CCR1, CCR2, CCR3, CCR4, CCR5,
    CCR6,
};

#[doc(hidden)]
pub enum Alt {
    Alt1,
    Alt2,
}

// Sealed by CapCmp
/// Associates PWM pins with specific GPIO pins
pub trait PwmPeriph<C>: CapCmp<C> + CapCmp<CCR0> {
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

impl PwmPeriph<CCR1> for pac::TB0 {
    type Gpio = Pin<P1, Pin6, Alternate2<Output>>;
    const ALT: Alt = Alt::Alt2;
}
impl PwmPeriph<CCR2> for pac::TB0 {
    type Gpio = Pin<P1, Pin7, Alternate2<Output>>;
    const ALT: Alt = Alt::Alt2;
}

impl PwmPeriph<CCR1> for pac::TB1 {
    type Gpio = Pin<P2, Pin0, Alternate1<Output>>;
    const ALT: Alt = Alt::Alt1;
}
impl PwmPeriph<CCR2> for pac::TB1 {
    type Gpio = Pin<P2, Pin1, Alternate1<Output>>;
    const ALT: Alt = Alt::Alt1;
}

impl PwmPeriph<CCR1> for pac::TB2 {
    type Gpio = Pin<P5, Pin0, Alternate1<Output>>;
    const ALT: Alt = Alt::Alt1;
}
impl PwmPeriph<CCR2> for pac::TB2 {
    type Gpio = Pin<P5, Pin1, Alternate1<Output>>;
    const ALT: Alt = Alt::Alt1;
}

impl PwmPeriph<CCR1> for pac::TB3 {
    type Gpio = Pin<P6, Pin0, Alternate1<Output>>;
    const ALT: Alt = Alt::Alt1;
}
impl PwmPeriph<CCR2> for pac::TB3 {
    type Gpio = Pin<P6, Pin1, Alternate1<Output>>;
    const ALT: Alt = Alt::Alt1;
}
impl PwmPeriph<CCR3> for pac::TB3 {
    type Gpio = Pin<P6, Pin2, Alternate1<Output>>;
    const ALT: Alt = Alt::Alt1;
}
impl PwmPeriph<CCR4> for pac::TB3 {
    type Gpio = Pin<P6, Pin3, Alternate1<Output>>;
    const ALT: Alt = Alt::Alt1;
}
impl PwmPeriph<CCR5> for pac::TB3 {
    type Gpio = Pin<P6, Pin4, Alternate1<Output>>;
    const ALT: Alt = Alt::Alt1;
}
impl PwmPeriph<CCR6> for pac::TB3 {
    type Gpio = Pin<P6, Pin5, Alternate1<Output>>;
    const ALT: Alt = Alt::Alt1;
}

fn setup_pwm<T: TimerPeriph>(timer: &T, config: TimerConfig<T>, period: u16) {
    config.write_regs(timer);
    CCRn::<CCR0>::set_ccrn(timer, period);
    CCRn::<CCR0>::config_outmod(timer, Outmod::Toggle);
}

/// Collection of uninitialized PWM pins derived from timer peripheral with 3 capture-compare registers
pub struct PwmParts3<T: CapCmpTimer3> {
    /// PWM pin 1 (derived from capture-compare register 1)
    pub pwm1: PwmUninit<T, CCR1>,
    /// PWM pin 2 (derived from capture-compare register 2)
    pub pwm2: PwmUninit<T, CCR2>,
}

impl<T: CapCmpTimer3> PwmParts3<T> {
    /// Create uninitialized PWM pins with the same period
    pub fn new(timer: T, config: TimerConfig<T>, period: u16) -> Self {
        setup_pwm(&timer, config, period);
        // Configure PWM ports
        CCRn::<CCR1>::config_outmod(&timer, Outmod::ResetSet);
        CCRn::<CCR2>::config_outmod(&timer, Outmod::ResetSet);
        // Start the timer to run PWM
        timer.upmode();
        Self {
            pwm1: PwmUninit::new(),
            pwm2: PwmUninit::new(),
        }
    }
}

/// Collection of uninitialized PWM pins derived from timer peripheral with 7 capture-compare registers
pub struct PwmParts7<T: CapCmpTimer7> {
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

impl<T: CapCmpTimer7> PwmParts7<T> {
    /// Create uninitialized PWM pins with the same period
    pub fn new(timer: T, config: TimerConfig<T>, period: u16) -> Self {
        setup_pwm(&timer, config, period);
        // Configure PWM ports
        CCRn::<CCR1>::config_outmod(&timer, Outmod::ResetSet);
        CCRn::<CCR2>::config_outmod(&timer, Outmod::ResetSet);
        CCRn::<CCR3>::config_outmod(&timer, Outmod::ResetSet);
        CCRn::<CCR4>::config_outmod(&timer, Outmod::ResetSet);
        CCRn::<CCR5>::config_outmod(&timer, Outmod::ResetSet);
        CCRn::<CCR6>::config_outmod(&timer, Outmod::ResetSet);
        // Start the timer to run PWM
        timer.upmode();
        Self {
            pwm1: PwmUninit::new(),
            pwm2: PwmUninit::new(),
            pwm3: PwmUninit::new(),
            pwm4: PwmUninit::new(),
            pwm5: PwmUninit::new(),
            pwm6: PwmUninit::new(),
        }
    }
}

/// Uninitialized PWM pin
pub struct PwmUninit<T, C>(PhantomData<T>, PhantomData<C>);

impl<T: PwmPeriph<C>, C> PwmUninit<T, C> {
    /// Initializes the PWM pin by passing in the appropriately configured GPIO pin.
    #[inline]
    pub fn init(self, pin: T::Gpio) -> Pwm<T, C> {
        Pwm {
            _timer: PhantomData,
            _ccrn: PhantomData,
            pin,
        }
    }
}

impl<T, C> PwmUninit<T, C> {
    #[inline]
    fn new() -> Self {
        Self(PhantomData, PhantomData)
    }
}

/// An initialized Pwm pin
pub struct Pwm<T: PwmPeriph<C>, C> {
    _timer: PhantomData<T>,
    _ccrn: PhantomData<C>,
    pin: T::Gpio,
}

mod ehal1 {
    use super::*;
    use core::convert::Infallible;
    use embedded_hal::pwm::{ErrorType, SetDutyCycle};

    impl<T: PwmPeriph<C>, C> ErrorType for Pwm<T, C> {
        type Error = Infallible;
    }

    impl<T: PwmPeriph<C>, C> SetDutyCycle for Pwm<T, C> {
        #[inline]
        fn max_duty_cycle(&self) -> u16 {
            let timer = unsafe { T::steal() };
            CCRn::<CCR0>::get_ccrn(&timer)
        }

        /// Set the duty cycle to `duty / max_duty`.
        ///
        /// The caller is responsible for ensuring that the duty cycle value is less than or equal to the maximum duty cycle value,
        /// as reported by `max_duty_cycle`.
        ///
        /// As the error type is `Infallible` this can be safely unwrapped.
        #[inline]
        fn set_duty_cycle(&mut self, duty: u16) -> Result<(), Self::Error> {
            let timer = unsafe { T::steal() };
            CCRn::<C>::set_ccrn(&timer, duty);
            Ok(())
        }
    }
}

#[cfg(feature = "embedded-hal-02")]
mod ehal02 {
    use super::*;
    use embedded_hal_02::PwmPin;

    impl<T: PwmPeriph<C>, C> PwmPin for Pwm<T, C> {
        /// Number of cycles
        type Duty = u16;

        #[inline]
        fn set_duty(&mut self, duty: Self::Duty) {
            let timer = unsafe { T::steal() };
            CCRn::<C>::set_ccrn(&timer, duty);
        }

        #[inline]
        fn get_duty(&self) -> Self::Duty {
            let timer = unsafe { T::steal() };
            CCRn::<C>::get_ccrn(&timer)
        }

        /// Maximum valid duty is equal to the period. If number of duty cycles exceeds number of
        /// period cycles, then signal stays high (equivalent to 100% duty cycle).
        #[inline]
        fn get_max_duty(&self) -> Self::Duty {
            let timer = unsafe { T::steal() };
            CCRn::<CCR0>::get_ccrn(&timer)
        }

        #[inline]
        fn disable(&mut self) {
            T::to_gpio(&mut self.pin);
        }

        #[inline]
        fn enable(&mut self) {
            T::to_alt(&mut self.pin);
        }
    }
}
