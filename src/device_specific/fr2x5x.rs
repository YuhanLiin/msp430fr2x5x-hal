pub use msp430fr2355 as pac;
/*         GPIO          */
mod gpio {
    use crate::gpio::*;
    use crate::pac::{P1, P2, P3, P4, P5, P6};
    use crate::hw_traits::gpio::gpio_impl; 

    // Define alternate pin transitions

    // P1 alternate 1
    impl<PIN: PinNum, DIR> ToAlternate1 for Pin<P1, PIN, DIR> {}
    // P1 alternate 2
    impl<DIR>  ToAlternate2 for Pin<P1, Pin0, DIR> {}
    impl<DIR>  ToAlternate2 for Pin<P1, Pin1, DIR> {}
    impl<PULL> ToAlternate2 for Pin<P1, Pin2, Input<PULL>> {}
    impl<DIR>  ToAlternate2 for Pin<P1, Pin6, DIR> {}
    impl<DIR>  ToAlternate2 for Pin<P1, Pin7, DIR> {}
    // P1 alternate 3
    impl<PIN: PinNum, DIR> ToAlternate3 for Pin<P1, PIN, DIR> {}

    // P2 alternate 1
    impl<DIR>  ToAlternate1 for Pin<P2, Pin0, DIR> {}
    impl<DIR>  ToAlternate1 for Pin<P2, Pin1, DIR> {}
    impl<PULL> ToAlternate1 for Pin<P2, Pin2, Input<PULL>> {}
    impl<DIR>  ToAlternate1 for Pin<P2, Pin3, DIR> {}
    impl<DIR>  ToAlternate1 for Pin<P2, Pin6, DIR> {}
    impl<DIR>  ToAlternate1 for Pin<P2, Pin7, DIR> {}
    // P2 alternate 2
    impl ToAlternate2 for Pin<P2, Pin0, Output> {}
    impl ToAlternate2 for Pin<P2, Pin1, Output> {}
    impl<DIR> ToAlternate2 for Pin<P2, Pin6, DIR> {}
    impl<DIR> ToAlternate2 for Pin<P2, Pin7, DIR> {}
    // P2 alternate 3
    impl<DIR> ToAlternate3 for Pin<P2, Pin4, DIR> {}
    impl<DIR> ToAlternate3 for Pin<P2, Pin5, DIR> {}

    // P3 alternate 1
    impl<DIR> ToAlternate1 for Pin<P3, Pin0, DIR> {}
    impl<DIR> ToAlternate1 for Pin<P3, Pin4, DIR> {}
    // P3 alternate 3
    impl<DIR> ToAlternate3 for Pin<P3, Pin1, DIR> {}
    impl<DIR> ToAlternate3 for Pin<P3, Pin2, DIR> {}
    impl<DIR> ToAlternate3 for Pin<P3, Pin3, DIR> {}
    impl<DIR> ToAlternate3 for Pin<P3, Pin5, DIR> {}
    impl<DIR> ToAlternate3 for Pin<P3, Pin6, DIR> {}
    impl<DIR> ToAlternate3 for Pin<P3, Pin7, DIR> {}

    // P4 alternate 1
    impl<PIN: PinNum, DIR> ToAlternate1 for Pin<P4, PIN, DIR> {}
    // P4 alternate 2
    impl<DIR> ToAlternate2 for Pin<P4, Pin0, DIR> {}
    impl<DIR> ToAlternate2 for Pin<P4, Pin2, DIR> {}
    impl<DIR> ToAlternate2 for Pin<P4, Pin3, DIR> {}

    // P5 alternate 1
    impl<DIR> ToAlternate1 for Pin<P5, Pin0, DIR> {}
    impl<DIR> ToAlternate1 for Pin<P5, Pin1, DIR> {}
    impl<DIR> ToAlternate1 for Pin<P5, Pin2, DIR> {}
    impl<DIR> ToAlternate1 for Pin<P5, Pin3, DIR> {}
    // P5 alternate 2
    impl<DIR> ToAlternate2 for Pin<P5, Pin0, DIR> {}
    impl<DIR> ToAlternate2 for Pin<P5, Pin1, DIR> {}
    // P5 alternate 3
    impl<DIR> ToAlternate3 for Pin<P5, Pin0, DIR> {}
    impl<DIR> ToAlternate3 for Pin<P5, Pin1, DIR> {}
    impl<DIR> ToAlternate3 for Pin<P5, Pin2, DIR> {}
    impl<DIR> ToAlternate3 for Pin<P5, Pin3, DIR> {}

    // P6 alternate 1
    impl<PIN: PinNum, DIR> ToAlternate1 for Pin<P6, PIN, DIR> {}

    // GPIO port impls, PAC register methods, and marking ports as interrupt-capable
    gpio_impl!(p1: P1 => p1in, p1out, p1dir, p1ren, p1selc, p1sel0, p1sel1, [p1ies, p1ie, p1ifg, p1iv]);
    gpio_impl!(p2: P2 => p2in, p2out, p2dir, p2ren, p2selc, p2sel0, p2sel1, [p2ies, p2ie, p2ifg, p2iv]);
    gpio_impl!(p3: P3 => p3in, p3out, p3dir, p3ren, p3selc, p3sel0, p3sel1, [p3ies, p3ie, p3ifg, p3iv]);
    gpio_impl!(p4: P4 => p4in, p4out, p4dir, p4ren, p4selc, p4sel0, p4sel1, [p4ies, p4ie, p4ifg, p4iv]);
    gpio_impl!(p5: P5 => p5in, p5out, p5dir, p5ren, p5selc, p5sel0, p5sel1);
    gpio_impl!(p6: P6 => p6in, p6out, p6dir, p6ren, p6selc, p6sel0, p6sel1);
}

/* ADC */
mod adc {
    use crate::{gpio::*, adc::*};

    impl_adc_channel_pin!(P1, Pin0, 0);
    impl_adc_channel_pin!(P1, Pin1, 1);
    impl_adc_channel_pin!(P1, Pin2, 2);
    impl_adc_channel_pin!(P1, Pin3, 3);
    impl_adc_channel_pin!(P1, Pin4, 4);
    impl_adc_channel_pin!(P1, Pin5, 5);
    impl_adc_channel_pin!(P1, Pin6, 6);
    impl_adc_channel_pin!(P1, Pin7, 7);
    impl_adc_channel_pin!(P5, Pin0, 8);
    impl_adc_channel_pin!(P5, Pin1, 9);
    impl_adc_channel_pin!(P5, Pin2, 10);
    impl_adc_channel_pin!(P5, Pin3, 11);
}

/* Backup Memory */
pub const BAK_MEM_SIZE: usize = 32;

/* Capture */
mod capture {
    use crate::{pac::*, gpio::*, capture::CapturePeriph};

    impl CapturePeriph for TB0 {
        type Gpio1 = Pin<P1, Pin6, Alternate2<Input<Floating>>>;
        type Gpio2 = Pin<P1, Pin7, Alternate2<Input<Floating>>>;
        type Gpio3 = ();
        type Gpio4 = ();
        type Gpio5 = ();
        type Gpio6 = ();
    }

    impl CapturePeriph for TB1 {
        type Gpio1 = Pin<P2, Pin0, Alternate1<Input<Floating>>>;
        type Gpio2 = Pin<P2, Pin1, Alternate1<Input<Floating>>>;
        type Gpio3 = ();
        type Gpio4 = ();
        type Gpio5 = ();
        type Gpio6 = ();
    }

    impl CapturePeriph for TB2 {
        type Gpio1 = Pin<P5, Pin0, Alternate1<Input<Floating>>>;
        type Gpio2 = Pin<P5, Pin1, Alternate1<Input<Floating>>>;
        type Gpio3 = ();
        type Gpio4 = ();
        type Gpio5 = ();
        type Gpio6 = ();
    }

    impl CapturePeriph for TB3 {
        type Gpio1 = Pin<P6, Pin0, Alternate1<Input<Floating>>>;
        type Gpio2 = Pin<P6, Pin1, Alternate1<Input<Floating>>>;
        type Gpio3 = Pin<P6, Pin2, Alternate1<Input<Floating>>>;
        type Gpio4 = Pin<P6, Pin3, Alternate1<Input<Floating>>>;
        type Gpio5 = Pin<P6, Pin4, Alternate1<Input<Floating>>>;
        type Gpio6 = Pin<P6, Pin5, Alternate1<Input<Floating>>>;
    }
}

/* eCOMP */
pub mod ecomp {
    use core::convert::Infallible;

    use crate::{gpio::*, ecomp::*};
    use crate::hw_traits::{Steal, ecomp::*};
    use crate::pac::{E_COMP0, E_COMP1};
    #[cfg(feature = "sac")]
    use crate::{sac::Amplifier, pac::{SAC0, SAC1, SAC2, SAC3}};

    impl ECompInputs for E_COMP0 {
        type COMPx_0   = Pin<P1, Pin0, Alternate2<Input<Floating>>>;
        type COMPx_1   = Pin<P1, Pin1, Alternate2<Input<Floating>>>;
        type COMPx_Out = Pin<P2, Pin0, Alternate2<Output>>;
        #[cfg(feature = "sac")]
        type SACp = Amplifier<SAC0>;
        #[cfg(feature = "sac")]
        type SACn = Amplifier<SAC2>;
        
        type DeviceSpecific0    = (); // Internal 1.2V reference. No type required.
        type DeviceSpecific1    = Infallible; // Not used
        type DeviceSpecific2Pos = Infallible; // Not used
        type DeviceSpecific2Neg = Infallible; // Not used
        type DeviceSpecific3Pos = Pin<P1, Pin1, Alternate2<Input<Floating>>>;
        type DeviceSpecific3Neg = Pin<P3, Pin1, Alternate2<Input<Floating>>>;
    }
    impl ECompInputs for E_COMP1 {
        type COMPx_0 = Pin<P2, Pin5, Alternate2<Input<Floating>>>;
        type COMPx_1 = Pin<P2, Pin4, Alternate2<Input<Floating>>>;
        type COMPx_Out = Pin<P2, Pin1, Alternate2<Output>>;
        #[cfg(feature = "sac")]
        type SACp = Amplifier<SAC1>;
        #[cfg(feature = "sac")]
        type SACn = Amplifier<SAC3>;

        type DeviceSpecific0    = (); // Internal 1.2V reference. No type required.
        type DeviceSpecific1    = Infallible; // Not used
        type DeviceSpecific2Pos = Infallible; // Not used
        type DeviceSpecific2Neg = Infallible; // Not used
        type DeviceSpecific3Pos = Pin<P1, Pin5, Alternate2<Input<Floating>>>;
        type DeviceSpecific3Neg = Pin<P3, Pin5, Alternate2<Input<Floating>>>;
    }

        /// List of possible inputs to the positive input of an eCOMP comparator.
    /// The amplifier output and DAC options take a reference to ensure they have been configured.
    #[allow(non_camel_case_types)]
    pub enum PositiveInput<'a, COMP: ECompInputs> {
        /// COMPx.0. P1.0 for COMP0, P2.5 for COMP1
        COMPx_0(COMP::COMPx_0),
        /// COMPx.1. P1.1 for COMP0, P2.4 for COMP1
        COMPx_1(COMP::COMPx_1),
        /// Internal 1.2V reference
        _1V2,
        #[cfg(feature = "sac")]
        /// Output of amplifier SAC0 for eCOMP0, SAC2 for eCOMP1.
        ///
        /// Requires a reference to ensure that it has been configured.
        OAxO(&'a COMP::SACp),
        /// This eCOMP's internal 6-bit DAC
        ///
        /// Requires a reference to ensure that it has been configured.
        Dac(&'a dyn CompDacPeriph<COMP>),
    }
    impl<COMP: ECompInputs> PositiveInput<'_, COMP> {
        #[inline(always)]
        pub(crate) fn cppsel(&self) -> u8 {
            match self {
                PositiveInput::COMPx_0(_)   => 0b000,
                PositiveInput::COMPx_1(_)   => 0b001,
                PositiveInput::_1V2         => 0b010,
                #[cfg(feature = "sac")]
                PositiveInput::OAxO(_)      => 0b101,
                PositiveInput::Dac(_)       => 0b110,
            }
        }
    }

    /// List of possible inputs to the negative input of an eCOMP comparator.
    /// The amplifier output and DAC options take a reference to ensure they have been configured.
    #[allow(non_camel_case_types)]
    pub enum NegativeInput<'a, COMP: ECompInputs> {
        /// COMPx.0. P1.0 for COMP0, P2.5 for COMP1
        COMPx_0(COMP::COMPx_0),
        /// COMPx.1. P1.1 for COMP0, P2.4 for COMP1
        COMPx_1(COMP::COMPx_1),
        /// Internal 1.2V reference
        _1V2,
        #[cfg(feature = "sac")]
        /// Output of amplifier SAC1 for eCOMP0, SAC3 for eCOMP1.
        OAxO(&'a COMP::SACn),
        /// This eCOMP's internal 6-bit DAC
        Dac(&'a dyn CompDacPeriph<COMP>),
    }
    impl<COMP: ECompInputs> NegativeInput<'_, COMP> {
        #[inline(always)]
        pub(crate) fn cpnsel(&self) -> u8 {
            match self {
                NegativeInput::COMPx_0(_)   => 0b000,
                NegativeInput::COMPx_1(_)   => 0b001,
                NegativeInput::_1V2         => 0b010,
                #[cfg(feature = "sac")]
                NegativeInput::OAxO(_)      => 0b101,
                NegativeInput::Dac(_)       => 0b110,
            }
        }
    }

    impl_ecomp!(
        E_COMP0,
        cpctl0, cpctl1,
        cpdacctl, cpdacdata,
        cpint, cpiv
    );

    impl_ecomp!(
        E_COMP1,
        cp1ctl0, cp1ctl1,
        cp1dacctl, cp1dacdata,
        cp1int, cp1iv
    );
}

/* Information Memory */
pub const INFO_MEM_SIZE: usize = 512;

/* PWM */
mod pwm {
    use crate::{pac::*, gpio::*, pwm::*};

    // TB0
    impl PwmPeriph<CCR1> for TB0 {
        type Gpio = Pin<P1, Pin6, Alternate2<Output>>;
        const ALT: Alt = Alt::Alt2;
    }
    impl PwmPeriph<CCR2> for TB0 {
        type Gpio = Pin<P1, Pin7, Alternate2<Output>>;
        const ALT: Alt = Alt::Alt2;
    }

    // TB1
    impl PwmPeriph<CCR1> for TB1 {
        type Gpio = Pin<P2, Pin0, Alternate1<Output>>;
        const ALT: Alt = Alt::Alt1;
    }
    impl PwmPeriph<CCR2> for TB1 {
        type Gpio = Pin<P2, Pin1, Alternate1<Output>>;
        const ALT: Alt = Alt::Alt1;
    }

    // TB2
    impl PwmPeriph<CCR1> for TB2 {
        type Gpio = Pin<P5, Pin0, Alternate1<Output>>;
        const ALT: Alt = Alt::Alt1;
    }
    impl PwmPeriph<CCR2> for TB2 {
        type Gpio = Pin<P5, Pin1, Alternate1<Output>>;
        const ALT: Alt = Alt::Alt1;
    }

    // TB3
    impl PwmPeriph<CCR1> for TB3 {
        type Gpio = Pin<P6, Pin0, Alternate1<Output>>;
        const ALT: Alt = Alt::Alt1;
    }
    impl PwmPeriph<CCR2> for TB3 {
        type Gpio = Pin<P6, Pin1, Alternate1<Output>>;
        const ALT: Alt = Alt::Alt1;
    }
    impl PwmPeriph<CCR3> for TB3 {
        type Gpio = Pin<P6, Pin2, Alternate1<Output>>;
        const ALT: Alt = Alt::Alt1;
    }
    impl PwmPeriph<CCR4> for TB3 {
        type Gpio = Pin<P6, Pin3, Alternate1<Output>>;
        const ALT: Alt = Alt::Alt1;
    }
    impl PwmPeriph<CCR5> for TB3 {
        type Gpio = Pin<P6, Pin4, Alternate1<Output>>;
        const ALT: Alt = Alt::Alt1;
    }
    impl PwmPeriph<CCR6> for TB3 {
        type Gpio = Pin<P6, Pin5, Alternate1<Output>>;
        const ALT: Alt = Alt::Alt1;
    }
}

/* SAC */
mod sac {
    use crate::{gpio::*, hw_traits::{Steal, sac::*}};
    use crate::pac::{SAC0, SAC1, SAC2, SAC3};

    impl_sac_periph!(
        SAC0, 
        P1, Pin3, // Positive input pin
        P1, Pin2, // Negative input pin
        P1, Pin1, // Output pin
        sac0oa, sac0pga, sac0dac, sac0dat
    );
    impl_sac_periph!(
        SAC1, 
        P1, Pin7, 
        P1, Pin6, 
        P1, Pin5,
        sac1oa, sac1pga, sac1dac, sac1dat
    );
    impl_sac_periph!(
        SAC2, 
        P3, Pin3, 
        P3, Pin2, 
        P3, Pin1,
        sac2oa, sac2pga, sac2dac, sac2dat
    );
    impl_sac_periph!(
        SAC3, 
        P3, Pin7, 
        P3, Pin6, 
        P3, Pin5,
        sac3oa, sac3pga, sac3dac, sac3dat
    );
}

/* Timer */
mod timer {
    use crate::{pac, gpio::*, timer::*, hw_traits::{Steal, timerb::*}};

    timerb_impl!(
        TB0,
        tb0,
        tb0ctl,
        tb0ex0,
        tb0iv,
        tb0r,
        [CCR0, tb0cctl0, tb0ccr0],
        [CCR1, tb0cctl1, tb0ccr1],
        [CCR2, tb0cctl2, tb0ccr2]
    );

    timerb_impl!(
        TB1,
        tb1,
        tb1ctl,
        tb1ex0,
        tb1iv,
        tb1r,
        [CCR0, tb1cctl0, tb1ccr0],
        [CCR1, tb1cctl1, tb1ccr1],
        [CCR2, tb1cctl2, tb1ccr2]
    );

    timerb_impl!(
        TB2,
        tb2,
        tb2ctl,
        tb2ex0,
        tb2iv,
        tb2r,
        [CCR0, tb2cctl0, tb2ccr0],
        [CCR1, tb2cctl1, tb2ccr1],
        [CCR2, tb2cctl2, tb2ccr2]
    );

    timerb_impl!(
        TB3,
        tb3,
        tb3ctl,
        tb3ex0,
        tb3iv,
        tb3r,
        [CCR0, tb3cctl0, tb3ccr0],
        [CCR1, tb3cctl1, tb3ccr1],
        [CCR2, tb3cctl2, tb3ccr2],
        [CCR3, tb3cctl3, tb3ccr3],
        [CCR4, tb3cctl4, tb3ccr4],
        [CCR5, tb3cctl5, tb3ccr5],
        [CCR6, tb3cctl6, tb3ccr6]
    );
    
    impl TimerPeriph for TB0 {
        type Tbxclk = Pin<P2, Pin7, Alternate1<Input<Floating>>>;
    }
    impl CapCmpTimer3 for TB0 {}

    impl TimerPeriph for TB1 {
        type Tbxclk = Pin<P2, Pin2, Alternate1<Input<Floating>>>;
    }
    impl CapCmpTimer3 for TB1 {}

    impl TimerPeriph for TB2 {
        type Tbxclk = Pin<P5, Pin2, Alternate1<Input<Floating>>>;
    }
    impl CapCmpTimer3 for TB2 {}

    impl TimerPeriph for TB3 {
        type Tbxclk = Pin<P6, Pin6, Alternate1<Input<Floating>>>;
    }
    impl CapCmpTimer7 for TB3 {}
}
