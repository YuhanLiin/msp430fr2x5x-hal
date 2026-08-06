pub use msp430fr25x2 as pac;

/// PAC with standardised peripheral names. For the FR25x2 this is just the PAC.
pub use msp430fr25x2 as _pac;

/*         GPIO          */
pub mod gpio {
    // Make PAC GPIO available as a re-export
    pub use crate::pac::{P1, P2};

    use crate::gpio::*;
    use crate::hw_traits::gpio::gpio_impl;

    // Define alternate pin transitions

    // P1 alternate 1
    impl<PIN: PinNum, DIR> ToAlternate1 for Pin<P1, PIN, DIR> {}
    // P1 alternate 2
    impl<DIR> ToAlternate2 for Pin<P1, Pin1, DIR> {}
    impl<DIR> ToAlternate2 for Pin<P1, Pin2, DIR> {}
    impl<DIR> ToAlternate2 for Pin<P1, Pin3, DIR> {}
    impl<DIR> ToAlternate2 for Pin<P1, Pin4, DIR> {}
    impl<DIR> ToAlternate2 for Pin<P1, Pin5, DIR> {}
    impl<DIR> ToAlternate2 for Pin<P1, Pin6, DIR> {}
    // P1 alternate 3
    impl<PIN: PinNum, DIR> ToAlternate3 for Pin<P1, PIN, DIR> {}

    // P2 alternate 1
    impl<DIR> ToAlternate1 for Pin<P2, Pin0, DIR> {}
    impl<DIR> ToAlternate1 for Pin<P2, Pin1, DIR> {}
    impl<DIR> ToAlternate1 for Pin<P2, Pin2, DIR> {}
    impl<DIR> ToAlternate1 for Pin<P2, Pin3, DIR> {}
    impl<DIR> ToAlternate1 for Pin<P2, Pin4, DIR> {}
    impl<DIR> ToAlternate1 for Pin<P2, Pin5, DIR> {}
    impl<DIR> ToAlternate1 for Pin<P2, Pin6, DIR> {}
    // P2 alternate 2
    impl<DIR> ToAlternate2 for Pin<P2, Pin0, DIR> {}
    impl<DIR> ToAlternate2 for Pin<P2, Pin1, DIR> {}
    impl<DIR> ToAlternate2 for Pin<P2, Pin2, DIR> {}
    impl<DIR> ToAlternate2 for Pin<P2, Pin3, DIR> {}
    impl<DIR> ToAlternate2 for Pin<P2, Pin4, DIR> {}

    // GPIO port impls, PAC register methods, and marking ports as interrupt-capable
    gpio_impl!(p1: P1 => p1in, p1out, p1dir, p1ren, p1selc, p1sel0, p1sel1, [p1ies, p1ie, p1ifg, p1iv]);
    gpio_impl!(p2: P2 => p2in, p2out, p2dir, p2ren, p2selc, p2sel0, p2sel1, [p2ies, p2ie, p2ifg, p2iv]);
}

/* ADC */
mod adc {
    use crate::{adc::*, gpio::*};

    impl_adc_channel_pin!(P1, Pin0, Alternate3 => 0);
    impl_adc_channel_pin!(P1, Pin1, Alternate3 => 1);
    impl_adc_channel_pin!(P1, Pin2, Alternate3 => 2);
    impl_adc_channel_pin!(P1, Pin3, Alternate3 => 3);
    impl_adc_channel_pin!(P2, Pin2, Alternate3 => 4);
    impl_adc_channel_pin!(P2, Pin3, Alternate3 => 5);
    impl_adc_channel_pin!(P2, Pin4, Alternate3 => 6);
    impl_adc_channel_pin!(P2, Pin5, Alternate3 => 7);
}

/* Backup Memory */
/// Size of the Backup Memory segment on this device, in bytes
pub const BAK_MEM_SIZE: usize = 32;

/* Capture */
mod capture {
    use crate::{capture::CapturePeriph, gpio::*, pac::*};

    impl CapturePeriph for Ta0 {
        type Gpio0 = ();
        type Gpio1 = Pin<P1, Pin4, Alternate2<Input<Floating>>>;
        type Gpio2 = Pin<P1, Pin5, Alternate2<Input<Floating>>>;
        type Gpio3 = ();
        type Gpio4 = ();
        type Gpio5 = ();
        type Gpio6 = ();
    }

    impl CapturePeriph for Ta1 {
        type Gpio0 = ();
        type Gpio1 = Pin<P2, Pin2, Alternate1<Input<Floating>>>;
        type Gpio2 = Pin<P2, Pin3, Alternate1<Input<Floating>>>;
        type Gpio3 = ();
        type Gpio4 = ();
        type Gpio5 = ();
        type Gpio6 = ();
    }
}

/* Clocks */
/// MODCLK frequency
pub const MODCLK_FREQ_HZ: u32 = 5_000_000;

/* eUSCI */
mod eusci {
    use crate::{
        hw_traits::{eusci::*, Steal},
        pac::*,
    };

    eusci_steal_impl!(EUsciA0);
    eusci_steal_impl!(EUsciB0);
}

/* I2C */
mod i2c {
    use crate::{
        gpio::*,
        hw_traits::eusci::*,
        i2c::{impl_i2c_pin, I2cUsci},
        pac::*,
        pin_mapping::*,
    };

    eusci_i2c_impl!(
        EUsciB0,
        ucb0ctlw0,
        ucb0ctlw1,
        ucb0brw,
        ucb0statw,
        ucb0tbcnt,
        ucb0rxbuf,
        ucb0txbuf,
        ucb0i2coa0,
        ucb0i2coa1,
        ucb0i2coa2,
        ucb0i2coa3,
        ucb0addrx,
        ucb0addmask,
        ucb0i2csa,
        ucb0ie,
        ucb0ifg,
        ucb0iv,
        crate::pac::e_usci_b0::ucb0ifg::R,
    );

    /// I2C SCL pin for eUSCI B0 (default mapping)
    pub struct UsciB0SCLPinDefault;
    impl_i2c_pin!(UsciB0SCLPinDefault, P1, Pin3);

    /// I2C SCL pin for eUSCI B0 (remapped mapping)
    pub struct UsciB0SCLPinRemapped;
    impl_i2c_pin!(UsciB0SCLPinRemapped, P2, Pin6);

    /// I2C SDA pin for eUSCI B0 (default mapping)
    pub struct UsciB0SDAPinDefault;
    impl_i2c_pin!(UsciB0SDAPinDefault, P1, Pin2);

    /// I2C SDA pin for eUSCI B0 (remapped mapping)
    pub struct UsciB0SDAPinRemapped;
    impl_i2c_pin!(UsciB0SDAPinRemapped, P2, Pin5);

    /// UCLKI pin for eUSCI B0. Used as an external clock source. (default mapping)
    pub struct UsciB0UCLKIPinDefault;
    impl_i2c_pin!(UsciB0UCLKIPinDefault, P1, Pin1);

    /// UCLKI pin for eUSCI B0. Used as an external clock source. (remapped mapping)
    pub struct UsciB0UCLKIPinRemapped;
    impl_i2c_pin!(UsciB0UCLKIPinRemapped, P2, Pin4);

    impl I2cUsci<DefaultMapping> for EUsciB0 {
        type ClockPin = UsciB0SCLPinDefault;
        type DataPin = UsciB0SDAPinDefault;
        type ExternalClockPin = UsciB0UCLKIPinDefault;

        fn configure_pin_mapping() {
            let sys = unsafe { crate::_pac::Sys::steal() };
            sys.syscfg2().write(|w| w.uscibrmp().clear_bit());
        }
    }
    impl I2cUsci<RemappedMapping> for EUsciB0 {
        type ClockPin = UsciB0SCLPinRemapped;
        type DataPin = UsciB0SDAPinRemapped;
        type ExternalClockPin = UsciB0UCLKIPinRemapped;

        fn configure_pin_mapping() {
            let sys = unsafe { crate::_pac::Sys::steal() };
            sys.syscfg2().write(|w| w.uscibrmp().set_bit());
        }
    }
}

/* Information Memory */
/// Size of the Information Memory segment on this device, in bytes
pub const INFO_MEM_SIZE: usize = 512;

/* PWM */
mod pwm {
    use crate::{gpio::*, pac::*, pwm::*};

    // TA0
    impl PwmPeriph<CCR1> for Ta0 {
        type Gpio = Pin<P1, Pin4, Alternate2<Output>>;
        const ALT: Alt = Alt::Alt2;
    }
    impl PwmPeriph<CCR2> for Ta0 {
        type Gpio = Pin<P1, Pin5, Alternate2<Output>>;
        const ALT: Alt = Alt::Alt2;
    }

    // TA1
    impl PwmPeriph<CCR1> for Ta1 {
        type Gpio = Pin<P2, Pin2, Alternate1<Output>>;
        const ALT: Alt = Alt::Alt1;
    }
    impl PwmPeriph<CCR2> for Ta1 {
        type Gpio = Pin<P2, Pin3, Alternate1<Output>>;
        const ALT: Alt = Alt::Alt1;
    }
}

/* Serial */
mod serial {
    use crate::{gpio::*, hw_traits::eusci::*, pac::*, pin_mapping::*, serial::*};

    eusci_uart_impl!(
        EUsciA0,
        uca0ctlw0,
        uca0ctlw1,
        uca0brw,
        uca0mctlw,
        uca0statw,
        uca0rxbuf,
        uca0txbuf,
        uca0ie,
        uca0ifg,
        uca0iv,
        crate::pac::e_usci_a0::uca0statw::R
    );

    impl SerialUsci<DefaultMapping> for EUsciA0 {
        type ClockPin = UsciA0ClockPinDefault;
        type TxPin = UsciA0TxPinDefault;
        type RxPin = UsciA0RxPinDefault;

        fn configure_pin_mapping() {
            let sys = unsafe { crate::_pac::Sys::steal() };
            sys.syscfg3().write(|w| w.usciarmp().clear_bit());
        }
    }
    impl SerialUsci<RemappedMapping> for EUsciA0 {
        type ClockPin = UsciA0ClockPinRemapped;
        type TxPin = UsciA0TxPinRemapped;
        type RxPin = UsciA0RxPinRemapped;

        fn configure_pin_mapping() {
            let sys = unsafe { crate::_pac::Sys::steal() };
            sys.syscfg3().write(|w| w.usciarmp().set_bit());
        }
    }

    /// UCLK pin for E_USCI_A0 (default mapping)
    pub struct UsciA0ClockPinDefault;
    impl_serial_pin!(UsciA0ClockPinDefault, P1, Pin6);

    /// UCLK pin for E_USCI_A0 (remapped mapping)
    pub struct UsciA0ClockPinRemapped;
    impl_serial_pin!(UsciA0ClockPinRemapped, P1, Pin6);

    /// Tx pin for E_USCI_A0 (default mapping)
    pub struct UsciA0TxPinDefault;
    impl_serial_pin!(UsciA0TxPinDefault, P1, Pin4);

    /// Tx pin for E_USCI_A0 (remapped mapping)
    pub struct UsciA0TxPinRemapped;
    impl_serial_pin!(UsciA0TxPinRemapped, P2, Pin0);

    /// Rx pin for E_USCI_A0 (default mapping)
    pub struct UsciA0RxPinDefault;
    impl_serial_pin!(UsciA0RxPinDefault, P1, Pin5);

    /// Rx pin for E_USCI_A0 (remapped mapping)
    pub struct UsciA0RxPinRemapped;
    impl_serial_pin!(UsciA0RxPinRemapped, P2, Pin1);
}

/* SPI */
mod spi {
    use crate::{gpio::*, hw_traits::eusci::*, pac::*, pin_mapping::*, spi::*};

    eusci_spi_impl!(
        EUsciA0,
        uca0ctlw0_spi,
        uca0brw,
        uca0statw_spi,
        uca0rxbuf,
        uca0txbuf,
        uca0ie_spi,
        uca0ifg_spi,
        uca0iv,
        crate::pac::e_usci_a0::uca0statw_spi::R
    );
    eusci_spi_impl!(
        EUsciB0,
        ucb0ctlw0_spi,
        ucb0brw,
        ucb0statw_spi,
        ucb0rxbuf,
        ucb0txbuf,
        ucb0ie_spi,
        ucb0ifg_spi,
        ucb0iv,
        crate::pac::e_usci_b0::ucb0statw_spi::R
    );

    impl SpiUsci<DefaultMapping> for EUsciA0 {
        type MISO = UsciA0MISOPinDefault;
        type MOSI = UsciA0MOSIPinDefault;
        type SCLK = UsciA0SCLKPinDefault;
        type STE = UsciA0STEPinDefault;

        fn configure_pin_mapping() {
            let sys = unsafe { crate::_pac::Sys::steal() };
            sys.syscfg3().write(|w| w.usciarmp().clear_bit());
        }
    }

    impl SpiUsci<RemappedMapping> for EUsciA0 {
        type MISO = UsciA0MISOPinRemapped;
        type MOSI = UsciA0MOSIPinRemapped;
        type SCLK = UsciA0SCLKPinRemapped;
        type STE = UsciA0STEPinRemapped;

        fn configure_pin_mapping() {
            let sys = unsafe { crate::_pac::Sys::steal() };
            sys.syscfg3().write(|w| w.usciarmp().set_bit());
        }
    }

    impl SpiUsci<DefaultMapping> for EUsciB0 {
        type MISO = UsciB0MISOPinDefault;
        type MOSI = UsciB0MOSIPinDefault;
        type SCLK = UsciB0SCLKPinDefault;
        type STE = UsciB0STEPinDefault;

        fn configure_pin_mapping() {
            let sys = unsafe { crate::_pac::Sys::steal() };
            sys.syscfg2().write(|w| w.uscibrmp().clear_bit());
        }
    }

    impl SpiUsci<RemappedMapping> for EUsciB0 {
        type MISO = UsciB0MISOPinRemapped;
        type MOSI = UsciB0MOSIPinRemapped;
        type SCLK = UsciB0SCLKPinRemapped;
        type STE = UsciB0STEPinRemapped;

        fn configure_pin_mapping() {
            let sys = unsafe { crate::_pac::Sys::steal() };
            sys.syscfg2().write(|w| w.uscibrmp().set_bit());
        }
    }

    /// SPI MISO pin for eUSCI A0 (P1.5) (default mapping)
    pub struct UsciA0MISOPinDefault;
    impl_spi_pin!(UsciA0MISOPinDefault, P1, Pin5);

    /// SPI MISO pin for eUSCI A0 (P2.1) (remapped mapping)
    pub struct UsciA0MISOPinRemapped;
    impl_spi_pin!(UsciA0MISOPinRemapped, P2, Pin1);

    /// SPI MOSI pin for eUSCI A0 (P1.4) (default mapping)
    pub struct UsciA0MOSIPinDefault;
    impl_spi_pin!(UsciA0MOSIPinDefault, P1, Pin4);

    /// SPI MOSI pin for eUSCI A0 (P2.0) (remapped mapping)
    pub struct UsciA0MOSIPinRemapped;
    impl_spi_pin!(UsciA0MOSIPinRemapped, P2, Pin0);

    /// SPI SCLK pin for eUSCI A0 (P1.6) (default mapping)
    pub struct UsciA0SCLKPinDefault;
    impl_spi_pin!(UsciA0SCLKPinDefault, P1, Pin6);

    /// SPI SCLK pin for eUSCI A0 (P1.6) (remapped mapping)
    pub struct UsciA0SCLKPinRemapped;
    impl_spi_pin!(UsciA0SCLKPinRemapped, P1, Pin6);

    /// SPI STE pin for eUSCI A0 (P1.7) (default mapping)
    pub struct UsciA0STEPinDefault;
    impl_spi_pin!(UsciA0STEPinDefault, P1, Pin7);

    /// SPI STE pin for eUSCI A0 (P1.7) (remapped mapping)
    pub struct UsciA0STEPinRemapped;
    impl_spi_pin!(UsciA0STEPinRemapped, P1, Pin7);

    /// SPI MISO pin for eUSCI B0 (P1.3) (default mapping)
    pub struct UsciB0MISOPinDefault;
    impl_spi_pin!(UsciB0MISOPinDefault, P1, Pin3);

    /// SPI MISO pin for eUSCI B0 (P2.6) (remapped mapping)
    pub struct UsciB0MISOPinRemapped;
    impl_spi_pin!(UsciB0MISOPinRemapped, P2, Pin6);

    /// SPI MOSI pin for eUSCI B0 (P1.2) (default mapping)
    pub struct UsciB0MOSIPinDefault;
    impl_spi_pin!(UsciB0MOSIPinDefault, P1, Pin2);

    /// SPI MOSI pin for eUSCI B0 (P2.5) (remapped mapping)
    pub struct UsciB0MOSIPinRemapped;
    impl_spi_pin!(UsciB0MOSIPinRemapped, P2, Pin5);

    /// SPI SCLK pin for eUSCI B0 (P1.1) (default mapping)
    pub struct UsciB0SCLKPinDefault;
    impl_spi_pin!(UsciB0SCLKPinDefault, P1, Pin1);

    /// SPI SCLK pin for eUSCI B0 (P2.4) (remapped mapping)
    pub struct UsciB0SCLKPinRemapped;
    impl_spi_pin!(UsciB0SCLKPinRemapped, P2, Pin4);

    /// SPI STE pin for eUSCI B0 (P1.0) (default mapping)
    pub struct UsciB0STEPinDefault;
    impl_spi_pin!(UsciB0STEPinDefault, P1, Pin0);

    /// SPI STE pin for eUSCI B0 (P2.3) (remapped mapping)
    pub struct UsciB0STEPinRemapped;
    impl_spi_pin!(UsciB0STEPinRemapped, P2, Pin3);
}

/* Timer */
mod timer {
    use crate::{
        gpio::*,
        hw_traits::{timer_a::*, Steal},
        pac::*,
        timer::*,
    };

    timer_a_impl!(
        Ta0,
        ta0,
        ta0ctl,
        ta0ex0,
        ta0iv,
        ta0r,
        taclr,
        taifg,
        taidex,
        taie,
        tassel,
        [CCR0, ta0cctl0, ta0ccr0],
        [CCR1, ta0cctl1, ta0ccr1],
        [CCR2, ta0cctl2, ta0ccr2]
    );

    timer_a_impl!(
        Ta1,
        ta1,
        ta1ctl,
        ta1ex0,
        ta1iv,
        ta1r,
        taclr,
        taifg,
        taidex,
        taie,
        tassel,
        [CCR0, ta1cctl0, ta1ccr0],
        [CCR1, ta1cctl1, ta1ccr1],
        [CCR2, ta1cctl2, ta1ccr2]
    );

    impl TimerPeriph for Ta0 {
        type Tbxclk = Pin<P1, Pin6, Alternate2<Input<Floating>>>;
    }
    impl CapCmpTimer3 for Ta0 {}

    impl TimerPeriph for Ta1 {
        type Tbxclk = Pin<P2, Pin4, Alternate1<Input<Floating>>>;
    }
    impl CapCmpTimer3 for Ta1 {}
}
