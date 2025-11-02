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

/* I2C */
mod i2c {
    use crate::{pac::*, gpio::*, i2c::{impl_i2c_pin, I2cUsci}};

    /// I2C SCL pin for eUSCI B0
    pub struct UsciB0SCLPin;
    impl_i2c_pin!(UsciB0SCLPin, P1, Pin3);

    /// I2C SDA pin for eUSCI B0
    pub struct UsciB0SDAPin;
    impl_i2c_pin!(UsciB0SDAPin, P1, Pin2);

    /// UCLKI pin for eUSCI B0. Used as an external clock source.
    pub struct UsciB0UCLKIPin;
    impl_i2c_pin!(UsciB0UCLKIPin, P1, Pin1);

    /// I2C SCL pin for eUSCI B1
    pub struct UsciB1SCLPin;
    impl_i2c_pin!(UsciB1SCLPin, P4, Pin7);

    /// I2C SDA pin for eUSCI B1
    pub struct UsciB1SDAPin;
    impl_i2c_pin!(UsciB1SDAPin, P4, Pin6);

    /// UCLKI pin for eUSCI B1. Used as an external clock source.
    pub struct UsciB1UCLKIPin;
    impl_i2c_pin!(UsciB1UCLKIPin, P4, Pin5);

    impl I2cUsci for E_USCI_B0 {
        type ClockPin = UsciB0SCLPin;
        type DataPin = UsciB0SDAPin;
        type ExternalClockPin = UsciB0UCLKIPin;
    }
    impl I2cUsci for E_USCI_B1 {
        type ClockPin = UsciB1SCLPin;
        type DataPin = UsciB1SDAPin;
        type ExternalClockPin = UsciB1UCLKIPin;
    }
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

/* Serial */
mod serial {
    use crate::{pac::*, gpio::*, serial::*};

    impl SerialUsci for E_USCI_A0 {
        type ClockPin = UsciA0ClockPin;
        type TxPin = UsciA0TxPin;
        type RxPin = UsciA0RxPin;
    }
    impl SerialUsci for E_USCI_A1 {
        type ClockPin = UsciA1ClockPin;
        type TxPin = UsciA1TxPin;
        type RxPin = UsciA1RxPin;
    }
    /// UCLK pin for E_USCI_A0
    pub struct UsciA0ClockPin;
    impl_serial_pin!(UsciA0ClockPin, P1, Pin5);

    /// Tx pin for E_USCI_A0
    pub struct UsciA0TxPin;
    impl_serial_pin!(UsciA0TxPin, P1, Pin7);

    /// Rx pin for E_USCI_A0
    pub struct UsciA0RxPin;
    impl_serial_pin!(UsciA0RxPin, P1, Pin6);

    /// UCLK pin for E_USCI_A1
    pub struct UsciA1ClockPin;
    impl_serial_pin!(UsciA1ClockPin, P4, Pin1);

    /// Tx pin for E_USCI_A1
    pub struct UsciA1TxPin;
    impl_serial_pin!(UsciA1TxPin, P4, Pin3);

    /// Rx pin for E_USCI_A1
    pub struct UsciA1RxPin;
    impl_serial_pin!(UsciA1RxPin, P4, Pin2);
}

/* SPI */
mod spi {
    use crate::{pac::*, gpio::*, spi::*};
    impl SpiUsci for E_USCI_A0 {
        type MISO = UsciA0MISOPin;
        type MOSI = UsciA0MOSIPin;
        type SCLK = UsciA0SCLKPin;
        type STE = UsciA0STEPin;
    }

    impl SpiUsci for E_USCI_A1 {
        type MISO = UsciA1MISOPin;
        type MOSI = UsciA1MOSIPin;
        type SCLK = UsciA1SCLKPin;
        type STE = UsciA1STEPin;
    }

    impl SpiUsci for E_USCI_B0 {
        type MISO = UsciB0MISOPin;
        type MOSI = UsciB0MOSIPin;
        type SCLK = UsciB0SCLKPin;
        type STE = UsciB0STEPin;
    }

    impl SpiUsci for E_USCI_B1 {
        type MISO = UsciB1MISOPin;
        type MOSI = UsciB1MOSIPin;
        type SCLK = UsciB1SCLKPin;
        type STE = UsciB1STEPin;
    }
    /// SPI MISO pin for eUSCI A0 (P1.6)
    pub struct UsciA0MISOPin;
    impl_spi_pin!(UsciA0MISOPin, P1, Pin6);

    /// SPI MOSI pin for eUSCI A0 (P1.7)
    pub struct UsciA0MOSIPin;
    impl_spi_pin!(UsciA0MOSIPin, P1, Pin7);

    /// SPI SCLK pin for eUSCI A0 (P1.5)
    pub struct UsciA0SCLKPin;
    impl_spi_pin!(UsciA0SCLKPin, P1, Pin5);

    /// SPI STE pin for eUSCI A0 (P1.4)
    pub struct UsciA0STEPin;
    impl_spi_pin!(UsciA0STEPin, P1, Pin4);

    /// SPI MISO pin for eUSCI A1 (P4.2)
    pub struct UsciA1MISOPin;
    impl_spi_pin!(UsciA1MISOPin, P4, Pin2);

    /// SPI MOSI pin for eUSCI A1 (P4.3)
    pub struct UsciA1MOSIPin;
    impl_spi_pin!(UsciA1MOSIPin, P4, Pin3);

    /// SPI SCLK pin for eUSCI A1 (P4.1)
    pub struct UsciA1SCLKPin;
    impl_spi_pin!(UsciA1SCLKPin, P4, Pin1);
    /// SPI STE pin for eUSCI A1 (P4.0)
    pub struct UsciA1STEPin;
    impl_spi_pin!(UsciA1STEPin, P4, Pin0);

    /// SPI MISO pin for eUSCI B0 (P1.3)
    pub struct UsciB0MISOPin;
    impl_spi_pin!(UsciB0MISOPin, P1, Pin3);

    /// SPI MOSI pin for eUSCI B0 (P1.2)
    pub struct UsciB0MOSIPin;
    impl_spi_pin!(UsciB0MOSIPin, P1, Pin2);

    /// SPI SCLK pin for eUSCI B0 (P1.1)
    pub struct UsciB0SCLKPin;
    impl_spi_pin!(UsciB0SCLKPin, P1, Pin1);

    /// SPI STE pin for eUSCI B0 (P1.0)
    pub struct UsciB0STEPin;
    impl_spi_pin!(UsciB0STEPin, P1, Pin0);

    /// SPI MISO pin for eUSCI B1 (P4.7)
    pub struct UsciB1MISOPin;
    impl_spi_pin!(UsciB1MISOPin, P4, Pin7);

    /// SPI MOSI pin for eUSCI B1 (P4.6)
    pub struct UsciB1MOSIPin;
    impl_spi_pin!(UsciB1MOSIPin, P4, Pin6);

    /// SPI SCLK pin for eUSCI B1 (P4.5)
    pub struct UsciB1SCLKPin;
    impl_spi_pin!(UsciB1SCLKPin, P4, Pin5);

    /// SPI STE pin for eUSCI B1 (P4.4)
    pub struct UsciB1STEPin;
    impl_spi_pin!(UsciB1STEPin, P4, Pin4);
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
