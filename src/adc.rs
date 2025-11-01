//! Analog to Digital Converter (ADC)
//!
//! Begin configuration by calling [`AdcConfig::new()`] or [`::default()`](AdcConfig::default()).
//! Once fully configured an [`Adc`] will be returned.
//!
//! [`Adc`] can read from a channel by calling [`read_count()`](Adc::read_count()) to return an ADC count.
//!
//! The [`count_to_mv()`](Adc::count_to_mv()) method is available to convert an ADC count to a voltage in millivolts,
//! given a reference voltage.
//!
//! As a convenience, [`read_voltage_mv()`](Adc::read_voltage_mv()) combines [`read_count()`](Adc::read_count()) and
//! [`count_to_mv()`](Adc::count_to_mv()).
//!
//! Currently the only supported ADC voltage reference is `AVCC`, the operating voltage of the MSP430.
//!
//! [`read_count()`](Adc::read_count()) takes a reference to the GPIO pin corresponding to the relevant ADC channel
//! to ensure it's been correctly configured. The ADC may read from any of the following pins:
//!
//! P1.0 - P1.7 (channels 0 to 7), P5.0 - P5.3 (channels 8 to 11).
//!
//! ADC channels 12 to 15 are not associated with external pins, so instead channels 12 and 13 can be read by passing a
//! reference to [`InternalVRef`] or [`InternalTempSensor`] respectively. Channels 14 and 15 require no prior
//! configuration, so the two functions below provide a reference that can be used to read from these channels.

use crate::{clock::{Aclk, Smclk}, gpio::*, pmm::{InternalTempSensor, InternalVRef}};
use core::convert::Infallible;
use crate::pac::ADC;

#[cfg(feature = "embedded-hal-02")]
pub use embedded_hal_02::adc::Channel;

#[cfg(not(feature = "embedded-hal-02"))]
/// A marker trait to identify MCU pins that can be used as inputs to an ADC channel.
///
/// This marker trait denotes an object, i.e. a GPIO pin, that is ready for use as an input to the
/// ADC. As ADCs channels can be supplied by multiple pins, this trait defines the relationship
/// between the physical interface and the ADC sampling buffer.
///
/// ```
/// # use std::marker::PhantomData;
/// # use embedded_hal::adc::Channel;
///
/// struct Adc1; // Example ADC with single bank of 8 channels
/// struct Gpio1Pin1<MODE>(PhantomData<MODE>);
/// struct Analog(()); // marker type to denote a pin in "analog" mode
///
/// // GPIO 1 pin 1 can supply an ADC channel when it is configured in Analog mode
/// impl Channel<Adc1> for Gpio1Pin1<Analog> {
///     fn channel() -> u8 { 7 } // GPIO pin 1 is connected to ADC channel 7
/// }
/// ```
pub trait Channel<ADC> {
    /// Type denoting the method used to identify ADC channels. This may be an integer (e.g. single ADC bank), or a tuple (multiple ADC banks), etc.
    /// On the MSP430 this is a `u8`, but this type remains generic for compatibility reasons with embedded-hal v0.2.7.
    type ID;
    /// Channel ID type
    ///
    /// A type used to identify this ADC channel. For example, if the ADC has eight channels, this
    /// might be a `u8`. If the ADC has multiple banks of channels, it could be a tuple, like
    /// `(u8: bank_id, u8: channel_id)`.
    /// Get the specific ID that identifies this channel, for example `0_u8` for the first ADC channel
    fn channel() -> u8;
}

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

#[derive(Default, Copy, Clone, PartialEq, Eq)]
enum ClockSource {
    /// Use MODCLK as the ADC input clock
    #[default]
    ModClk = 0b00,
    /// Use ACLK as the ADC input clock
    AClk = 0b01,
    /// Use SMCLK as the ADC input clock
    SmClk = 0b10,
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
macro_rules! impl_adc_channel_pin {
    ($port: ty, $pin: ty, $channel: literal ) => {
        impl Channel<Adc> for Pin<$port, $pin, Alternate3<Input<Floating>>> {
            type ID = u8;

            fn channel() -> Self::ID {
                $channel
            }
        }
    };
}

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

// A few ADC channels don't correspond to pins.
macro_rules! impl_adc_channel_extra {
    ($type: ty, $channel: literal ) => {
        impl Channel<Adc> for $type {
            type ID = u8;

            fn channel() -> Self::ID {
                $channel
            }
        }
    };
}

impl_adc_channel_extra!(InternalTempSensor<'_>, 12);
impl_adc_channel_extra!(InternalVRef, 13);

// Users needn't deal with the structs themselves so it just adds noise to the docs. We instead document the functions below.
#[doc(hidden)]
pub struct AdcVssChannel;
impl_adc_channel_extra!(AdcVssChannel, 14);
/// ADC channel 14, tied to VSS. Pass this function's output to `read_count()`.
#[inline(always)]
pub fn adc_ch14_vss() -> AdcVssChannel { AdcVssChannel }

#[doc(hidden)]
pub struct AdcVccChannel;
impl_adc_channel_extra!(AdcVccChannel, 15);
/// ADC channel 15, tied to VCC. Pass this function's output to `read_count()`.
#[inline(always)]
pub fn adc_ch15_vcc() -> AdcVccChannel { AdcVccChannel }

/// Typestate for an ADC configuration with no clock source selected
pub struct NoClockSet;
/// Typestate for an ADC configuration with a clock source selected
pub struct ClockSet(ClockSource);

/// Configuration object for an ADC.
///
/// Currently the only supported voltage reference is AVCC.
///
/// The default configuration is based on the default register values:
/// - Predivider = 1 and clock divider = 1
/// - 10-bit resolution
/// - 8 cycle sample time
/// - Max 200 ksps sample rate
#[derive(Clone, PartialEq, Eq)]
pub struct AdcConfig<STATE> {
    state: STATE,
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

// Only implement Default for NoClockSet
impl Default for AdcConfig<NoClockSet> {
    fn default() -> Self {
        Self {
            state: NoClockSet,
            clock_divider: Default::default(),
            predivider: Default::default(),
            resolution: Default::default(),
            sampling_rate: Default::default(),
            sample_time: Default::default(),
        }
    }
}

impl AdcConfig<NoClockSet> {
    /// Creates an ADC configuration. A default implementation is also available through `::default()`
    pub fn new(
        clock_divider: ClockDivider,
        predivider: Predivider,
        resolution: Resolution,
        sampling_rate: SamplingRate,
        sample_time: SampleTime,
    ) -> AdcConfig<NoClockSet> {
        AdcConfig {
            state: NoClockSet,
            clock_divider,
            predivider,
            resolution,
            sampling_rate,
            sample_time,
        }
    }
    /// Configure the ADC to use SMCLK
    pub fn use_smclk(self, _smclk: &Smclk) -> AdcConfig<ClockSet> {
        AdcConfig {
            state: ClockSet(ClockSource::SmClk),
            clock_divider: self.clock_divider,
            predivider: self.predivider,
            resolution: self.resolution,
            sampling_rate: self.sampling_rate,
            sample_time: self.sample_time,
        }
    }
    /// Configure the ADC to use ACLK
    pub fn use_aclk(self, _aclk: &Aclk) -> AdcConfig<ClockSet> {
        AdcConfig {
            state: ClockSet(ClockSource::AClk),
            clock_divider: self.clock_divider,
            predivider: self.predivider,
            resolution: self.resolution,
            sampling_rate: self.sampling_rate,
            sample_time: self.sample_time,
        }
    }
    /// Configure the ADC to use MODCLK
    pub fn use_modclk(self) -> AdcConfig<ClockSet> {
        AdcConfig {
            state: ClockSet(ClockSource::ModClk),
            clock_divider: self.clock_divider,
            predivider: self.predivider,
            resolution: self.resolution,
            sampling_rate: self.sampling_rate,
            sample_time: self.sample_time,
        }
    }
}
impl AdcConfig<ClockSet> {
    /// Applies this ADC configuration to hardware registers, and returns an ADC.
    pub fn configure(self, mut adc_reg: ADC) -> Adc {
        // Disable the ADC before we set the other bits. Some can only be set while the ADC is disabled.
        disable_adc_reg(&mut adc_reg);

        let adcsht = self.sample_time.adcsht();
        adc_reg.adcctl0.write(|w| w.adcsht().bits(adcsht));

        let adcssel = self.state.0.adcssel();
        let adcdiv = self.clock_divider.adcdiv();
        adc_reg.adcctl1.write(|w| { w
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
        }
    }
}

/// Controls the onboard ADC. The `read()` method is available through the embedded_hal `OneShot` trait.
pub struct Adc {
    adc_reg: ADC,
    is_waiting: bool,
}

impl Adc {
    /// Whether the ADC is currently sampling or converting.
    pub fn adc_is_busy(&self) -> bool {
        self.adc_reg.adcctl1.read().adcbusy().bit_is_set()
    }

    /// Gets the latest ADC conversion result.
    pub fn adc_get_result(&self) -> u16 {
        self.adc_reg.adcmem0.read().bits()
    }

    /// Enables this ADC, ready to start conversions.
    pub fn enable(&mut self) {
        unsafe {
            self.adc_reg.adcctl0.set_bits(|w| w.adcon().set_bit());
        }
    }

    /// Disables this ADC to save power.
    pub fn disable(&mut self) {
        disable_adc_reg(&mut self.adc_reg);
    }

    /// Selects which pin to sample.
    fn set_pin<PIN>(&mut self, _pin: &PIN)
    where
        PIN: Channel<Self, ID = u8>,
    {
        self.adc_reg
            .adcmctl0
            .modify(|_, w| w.adcinch().bits(PIN::channel()));
    }

    /// Starts an ADC conversion.
    fn start_conversion(&mut self) {
        unsafe {
            self.adc_reg.adcctl0.set_bits(|w| w
                .adcenc().set_bit()
                .adcsc().set_bit());
        }
    }

    /// Begins a single ADC conversion if one isn't already underway, enabling the ADC in the process.
    ///
    /// If the result is ready it is returned as an ADC count, otherwise returns `WouldBlock`
    pub fn read_count<PIN>(&mut self, pin: &mut PIN) -> nb::Result<u16, Infallible>
    where
        PIN: Channel<Self, ID = u8>,
    {
        if self.is_waiting {
            if self.adc_is_busy() {
                return Err(nb::Error::WouldBlock);
            } else {
                self.is_waiting = false;
                return Ok(self.adc_get_result());
            }
        }
        self.disable();
        self.set_pin(pin);
        self.enable();

        self.start_conversion();
        self.is_waiting = true;
        Err(nb::Error::WouldBlock)
    }

    /// Convert an ADC count to a voltage value in millivolts.
    ///
    /// `ref_voltage_mv` is the reference voltage of the ADC in millivolts.
    pub fn count_to_mv(&self, count: u16, ref_voltage_mv: u16) -> u16 {
        use crate::pac::adc::adcctl2::ADCRES_A;
        let resolution = match self.adc_reg.adcctl2.read().adcres().variant() {
            ADCRES_A::ADCRES_0 => 256,  //  8-bit
            ADCRES_A::ADCRES_1 => 1024, // 10-bit
            ADCRES_A::ADCRES_2 => 4096, // 12-bit
            ADCRES_A::ADCRES_3 => 4096, // Reserved, unreachable
        };
        ((count as u32 * ref_voltage_mv as u32) / resolution) as u16
    }

    /// Begins a single ADC conversion if one isn't already underway, enabling the ADC in the process.
    ///
    /// If the result is ready it is returned as a voltage in millivolts based on `ref_voltage_mv`, otherwise returns `WouldBlock`.
    ///
    /// If you instead want a raw count you should use the `.read_count()` method.
    pub fn read_voltage_mv<PIN: Channel<Self, ID = u8>>(&mut self, pin: &mut PIN, ref_voltage_mv: u16) -> nb::Result<u16, Infallible> {
        self.read_count(pin).map(|count| self.count_to_mv(count, ref_voltage_mv))
    }
}

fn disable_adc_reg(adc: &mut ADC) {
    unsafe {
        adc.adcctl0.clear_bits(|w| w
            .adcon().clear_bit()
            .adcenc().clear_bit());
    }
}

#[cfg(feature = "embedded-hal-02")]
mod ehal02 {
    use super::*;
    use embedded_hal_02::adc::{Channel, OneShot};

    impl<PIN> OneShot<Adc, u16, PIN> for Adc
    where
        PIN: Channel<Self, ID = u8>,
    {
        type Error = Infallible; // Only returns WouldBlock

        /// Begins a single ADC conversion if one isn't already underway, enabling the ADC in the process.
        ///
        /// If the result is ready it is returned as an ADC count, otherwise returns `WouldBlock`
        #[inline(always)]
        fn read(&mut self, pin: &mut PIN) -> nb::Result<u16, Self::Error> {
            self.read_count(pin)
        }
    }
}
