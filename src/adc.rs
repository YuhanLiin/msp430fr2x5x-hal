//! Analog to Digital Converter (ADC)
//!
//! The ADC may read from any of the following pins:
//!
//! P1.0 - P1.7 (channels 0 to 7), P5.0 - P5.3 (channels 8 to 11)
//!

use crate::gpio::*;
use core::{convert::Infallible, marker::PhantomData};
use embedded_hal::adc::{Channel, OneShot};
use msp430fr2355::ADC;

/// How many ADCCLK cycles the ADC's sample-and-hold stage will last for.
/// 
/// Default: 8 cycles
#[derive(Default, Copy, Clone, PartialEq, Eq)]
pub enum SampleTime {
    /// Sample for 4 ADCCLK cycles
    _4 = 0b0000,
    /// Sample for 8 ADCCLK cycles
    #[default]
    _8 = 0b0001,
    /// Sample for 16 ADCCLK cycles
    _16 = 0b0010,
    /// Sample for 32 ADCCLK cycles
    _32 = 0b0011,
    /// Sample for 64 ADCCLK cycles
    _64 = 0b0100,
    /// Sample for 96 ADCCLK cycles
    _96 = 0b0101,
    /// Sample for 128 ADCCLK cycles
    _128 = 0b0110,
    /// Sample for 192 ADCCLK cycles
    _192 = 0b0111,
    /// Sample for 256 ADCCLK cycles
    _256 = 0b1000,
    /// Sample for 384 ADCCLK cycles
    _384 = 0b1001,
    /// Sample for 512 ADCCLK cycles
    _512 = 0b1010,
    /// Sample for 768 ADCCLK cycles
    _768 = 0b1011,
    /// Sample for 1024 ADCCLK cycles
    _1024 = 0b1100,
}

impl SampleTime {
    #[inline(always)]
    fn adcsht(self) -> u8 {
        self as u8
    }
}

/// How much the ADC input clock will be divided by after being divided by the predivider
/// 
/// Default: Divide by 1
#[derive(Default, Copy, Clone, PartialEq, Eq)]
pub enum ClockDivider {
    /// Divide the input clock by 1
    #[default]
    _1 = 0b000,
    /// Divide the input clock by 2
    _2 = 0b001,
    /// Divide the input clock by 3
    _3 = 0b010,
    /// Divide the input clock by 4
    _4 = 0b011,
    /// Divide the input clock by 5
    _5 = 0b100,
    /// Divide the input clock by 6
    _6 = 0b101,
    /// Divide the input clock by 7
    _7 = 0b110,
    /// Divide the input clock by 8
    _8 = 0b111,
}

impl ClockDivider {
    #[inline(always)]
    fn adcdiv(self) -> u8 {
        self as u8
    }
}

/// Which clock source the ADC uses as input.
/// 
/// Default: MODCLK
#[derive(Default, Copy, Clone, PartialEq, Eq)]
pub enum ClockSource {
    /// Use MODCLK as the ADC input clock
    #[default]
    MODCLK = 0b00,
    /// Use ACLK as the ADC input clock
    ACLK = 0b01,
    /// Use SMCLK as the ADC input clock
    SMCLK = 0b10,
}

impl ClockSource {
    #[inline(always)]
    fn adcssel(self) -> u8 {
        self as u8
    }
}

/// How much the ADC input clock will be divided by prior to being divided by the ADC clock divider
/// 
/// Default: Divide by 1
#[derive(Default, Copy, Clone, PartialEq, Eq)]
pub enum Predivider {
    /// Divide the input clock by 1
    #[default]
    _1 = 0b00,
    /// Divide the input clock by 4
    _4 = 0b01,
    /// Divide the input clock by 64
    _64 = 0b10,
}

impl Predivider {
    #[inline(always)]
    fn adcpdiv(self) -> u8 {
        self as u8
    }
}

/// The output resolution of the ADC conversion. Also determines how many ADCCLK cycles the conversion step takes.
/// 
/// Default: 10-bit resolution
#[derive(Default, Copy, Clone, PartialEq, Eq)]
pub enum Resolution {
    /// 8-bit ADC conversion result. The conversion step takes 10 ADCCLK cycles.
    _8BIT = 0b00,
    /// 10-bit ADC conversion result. The conversion step takes 12 ADCCLK cycles.
    #[default]
    _10BIT = 0b01,
    /// 12-bit ADC conversion result. The conversion step takes 14 ADCCLK cycles.
    _12BIT = 0b10,
}

impl Resolution {
    #[inline(always)]
    fn adcres(self) -> u8 {
        self as u8
    }
}

/// Selects the drive capability of the ADC reference buffer, which can increase the maximum sampling speed at the cost of increased power draw.
/// 
/// Default: 200ksps
#[derive(Default, Copy, Clone, PartialEq, Eq)]
pub enum SamplingRate {
    /// Maximum of 50 ksps. Lower power usage.
    _50KSPS,
    /// Maximum of 200 ksps. Higher power usage.
    #[default]
    _200KSPS,
}

impl SamplingRate {
    #[inline(always)]
    fn adcsr(self) -> bool {
        match self {
            SamplingRate::_200KSPS => false,
            SamplingRate::_50KSPS => true,
        }
    }
}

// Pins corresponding to an ADC channel. Pin types can have `::channel()` called on them to get their ADC channel index.
macro_rules! impl_adc_channel {
    ($port: ty, $pin: ty, $channel: literal ) => {
        impl<S: AdcState> Channel<Adc<S>> for Pin<$port, $pin, Alternate3<Input<Floating>>> {
            type ID = u8;

            fn channel() -> Self::ID {
                $channel
            }
        }
    };
}

impl_adc_channel!(P1, Pin0, 0);
impl_adc_channel!(P1, Pin1, 1);
impl_adc_channel!(P1, Pin2, 2);
impl_adc_channel!(P1, Pin3, 3);
impl_adc_channel!(P1, Pin4, 4);
impl_adc_channel!(P1, Pin5, 5);
impl_adc_channel!(P1, Pin6, 6);
impl_adc_channel!(P1, Pin7, 7);
impl_adc_channel!(P5, Pin0, 8);
impl_adc_channel!(P5, Pin1, 9);
impl_adc_channel!(P5, Pin2, 10);
impl_adc_channel!(P5, Pin3, 11);

/// Controls the onboard ADC
pub struct Adc<STATE: AdcState> {
    adc_reg: ADC,
    is_waiting: bool,
    _phantom: PhantomData<STATE>,
}

/// Configuration object for an ADC.
/// 
/// The default configuration is based on the default register values:
/// - MODCLK as input clock
/// - Predivider = 1 and clock divider = 1
/// - 10-bit resolution
/// - 8 cycle sample time
/// - Max 200 ksps sample rate
#[derive(Default, Clone, PartialEq, Eq)]
pub struct AdcConfig {
    /// Which clock source the ADC takes as an input. This clock will first be divided by the predivider, then the clock divider, to generate ADCCLK.
    pub clock_source: ClockSource,
    /// How much the input clock is divided by, after the predivider.
    pub clock_divider: ClockDivider,
    /// How much the input clock is initially divided by, before the clock divider.
    pub predivider: Predivider,
    /// How many bits the conversion result is. Also defines the number of ADCCLK cycles required to do the conversion step.
    pub resolution: Resolution,
    /// Sets the maximum sampling rate of the ADC. Lower values use less power.
    pub sampling_rate: SamplingRate,
    /// Determines the number of ADCCLK cycles the sampling time takes.
    pub sample_time: SampleTime,
}

impl AdcConfig {
    /// Creates an ADC configuration. A default implementation is also available through `::default()`
    pub fn new(
        clock_source: ClockSource,
        clock_divider: ClockDivider,
        predivider: Predivider,
        resolution: Resolution,
        sampling_rate: SamplingRate,
        sample_time: SampleTime,
    ) -> AdcConfig {
        AdcConfig {
            clock_source,
            clock_divider,
            predivider,
            resolution,
            sampling_rate,
            sample_time,
        }
    }

    /// Applies this ADC configuration to hardware registers, and returns an ADC.
    pub fn config_hw(self, mut adc_reg: ADC) -> Adc<Disabled> {
        // Disable the ADC before we set the other bits. Some can only be set while the ADC is disabled.
        disable_adc_reg(&mut adc_reg);

        let adcsht = self.sample_time.adcsht();
        adc_reg.adcctl0.write(|w| w.adcsht().bits(adcsht));

        let adcssel = self.clock_source.adcssel();
        let adcdiv = self.clock_divider.adcdiv();
        adc_reg.adcctl1.write(|w| {w
            .adcssel().bits(adcssel)
            .adcshp().adcshp_1()
            .adcdiv().bits(adcdiv)
        });

        let adcpdiv = self.predivider.adcpdiv();
        let adcres = self.resolution.adcres();
        let adcsr = self.sampling_rate.adcsr();
        adc_reg.adcctl2.write(|w| { w
            .adcpdiv().bits(adcpdiv)
            .adcres().bits(adcres)
            .adcsr().bit(adcsr)
        });

        Adc {
            adc_reg,
            is_waiting: false,
            _phantom: PhantomData,
        }
    }
}
/// Typestate for an enabled ADC. It is ready to begin conversions. The ADC must be disabled before it can be reconfigured.
pub struct Enabled;
/// Typestate for a disabled ADC. It is ready to be configured. The ADC must be enabled before it can begin conversions.
pub struct Disabled;
/// Typestate trait for the current state of the ADC. The ADC may be either `Enabled` or `Disabled.`
pub trait AdcState: private::Sealed {}
impl AdcState for Enabled {}
impl AdcState for Disabled {}

// Seal this supertrait so users can still refer to AdcState, but they can't add other implementations besides `Enabled` and `Disabled`.
mod private {
    pub trait Sealed {}
    impl Sealed for super::Enabled {}
    impl Sealed for super::Disabled {}
}

impl<S: AdcState> Adc<S> {
    /// Whether the ADC is currently sampling or converting.
    pub fn adc_is_busy(&self) -> bool {
        self.adc_reg.adcctl1.read().adcbusy().bit_is_set()
    }

    /// Gets the latest ADC conversion result.
    pub fn adc_get_result(&self) -> u16 {
        self.adc_reg.adcmem0.read().bits()
    }
}

impl Adc<Disabled> {
    /// Enables this ADC, ready to start a conversion.
    pub fn into_enabled(mut self) -> Adc<Enabled> {
        enable_adc_reg(&mut self.adc_reg);
        Adc {
            adc_reg: self.adc_reg,
            is_waiting: self.is_waiting,
            _phantom: PhantomData,
        }
    }

    /// Selects which pin to sample.
    pub fn set_pin<PIN>(&mut self, _pin: &PIN)
    where
        PIN: Channel<Self, ID = u8>,
    {
        self.adc_reg
            .adcmctl0
            .modify(|_, w| w.adcinch().bits(PIN::channel()));
    }
}

impl Adc<Enabled> {
    /// Disables this ADC to save power.
    pub fn into_disabled(mut self) -> Adc<Disabled> {
        disable_adc_reg(&mut self.adc_reg);
        Adc {
            adc_reg: self.adc_reg,
            is_waiting: self.is_waiting,
            _phantom: PhantomData,
        }
    }

    /// Starts an ADC conversion.
    pub fn start_conversion(&mut self) {
        unsafe {
            self.adc_reg.adcctl0.set_bits(|w| w
                .adcenc().set_bit()
                .adcsc().set_bit());
        }
    }

    // We use this fn to implement OneShot, as otherwise we'd need to consume the Adc to change state.
    /// Disables the ADC, configures the input channel, then re-enables the ADC.
    pub fn reset_and_set_pin<PIN>(&mut self, _pin: &PIN)
    where
        PIN: Channel<Self, ID = u8>,
    {
        disable_adc_reg(&mut self.adc_reg);
        self.adc_reg
            .adcmctl0
            .modify(|_, w| w.adcinch().bits(PIN::channel()));

        enable_adc_reg(&mut self.adc_reg);
    }
}

fn disable_adc_reg(adc: &mut ADC) {
    unsafe {
        adc.adcctl0.clear_bits(|w| w
            .adcon().clear_bit()
            .adcenc().clear_bit());
    }
}

fn enable_adc_reg(adc: &mut ADC) {
    unsafe {
        adc.adcctl0.set_bits(|w| w.adcon().set_bit());
    }
}

impl<WORD, PIN> OneShot<Adc<Enabled>, WORD, PIN> for Adc<Enabled>
where
    WORD: From<u16>,
    PIN: Channel<Adc<Enabled>, ID = u8>,
{
    type Error = Infallible; // Only returns WouldBlock

    /// Begins a single ADC conversion if one is not already underway.
    ///
    /// If the result is ready it is returned, otherwise returns `WouldBlock`
    fn read(&mut self, pin: &mut PIN) -> nb::Result<WORD, Self::Error> {
        if self.is_waiting {
            if self.adc_is_busy() {
                return Err(nb::Error::WouldBlock);
            } else {
                self.is_waiting = false;
                return Ok(self.adc_get_result().into());
            }
        }

        self.reset_and_set_pin(pin);

        self.start_conversion();
        self.is_waiting = true;
        Err(nb::Error::WouldBlock)
    }
}
