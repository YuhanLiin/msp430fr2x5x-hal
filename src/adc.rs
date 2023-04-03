use core::{u8};
use crate::gpio::*;
use embedded_hal::{adc::{Channel, OneShot}};
use msp430fr2355::ADC;

pub enum SampleTime {
    _4,
    _8,
    _16,
    _32,
    _64,
    _96,
    _128,
    _192,
    _256,
    _384,
    _512,
    _768,
    _1024,
}

impl SampleTime {
    fn adcsht(self) -> u8 {
        match self {
            SampleTime::_4 => 0b000,
            SampleTime::_8 => 0b001,
            SampleTime::_16 => 0b010,
            SampleTime::_32 => 0b011,
            SampleTime::_64 => 0b100,
            SampleTime::_96 => 0b101,
            SampleTime::_128 => 0b110,
            SampleTime::_192 => 0b111,
            SampleTime::_256 => 0b1000,
            SampleTime::_384 => 0b1001,
            SampleTime::_512 => 0b1010,
            SampleTime::_768 => 0b1011,
            SampleTime::_1024 => 0b1100,
        }
    }
}

pub enum ClockDivider {
    _1,
    _2,
    _3,
    _4,
    _5,
    _6,
    _7,
    _8,
}

impl ClockDivider {
    fn adcdiv(self) -> u8 {
        match self {
            ClockDivider::_1 => 0b000,
            ClockDivider::_2 => 0b001,
            ClockDivider::_3 => 0b010,
            ClockDivider::_4 => 0b011,
            ClockDivider::_5 => 0b100,
            ClockDivider::_6 => 0b101,
            ClockDivider::_7 => 0b110,
            ClockDivider::_8 => 0b111,
        }
    }
}

pub enum ClockSource {
    MODCLK,
    ACLK,
    SMCLK,
}

impl ClockSource {
    fn adcssel(self) -> u8 {
        match self {
            ClockSource::MODCLK => 0b00,
            ClockSource::ACLK => 0b01,
            ClockSource::SMCLK => 0b10,
        }
    }
}

pub enum Predivider {
    _1,
    _4,
    _64,
}

impl Predivider {
    fn adcpdiv(self) -> u8 {
        match self {
            Predivider::_1 => 0b00,
            Predivider::_4 => 0b01,
            Predivider::_64 => 0b10,
        }
    }
}

pub enum Resolution {
    _8BIT,
    _10BIT,
    _12BIT,
}

impl Resolution {
    fn adcres(self) -> u8 {
        match self {
            Resolution::_8BIT => 0b00,
            Resolution::_10BIT => 0b01,
            Resolution::_12BIT => 0b10,
        }
    }
}

pub enum SamplingRate {
    _50KSPS,
    _200KSPS,
}

impl SamplingRate {
    fn adcsr(self) -> bool {
        match self {
            SamplingRate::_200KSPS => false,
            SamplingRate::_50KSPS => true,
        }
    }
}

impl Channel<Adc<ADC>> for Pin<P1, Pin0, Alternate3<Input<Floating>>> {
    type ID = u8;

    fn channel() -> Self::ID { 0 }
}

impl Channel<Adc<ADC>> for Pin<P1, Pin1, Alternate3<Input<Floating>>> {
    type ID = u8;

    fn channel() -> Self::ID { 1 }
}

impl Channel<Adc<ADC>> for Pin<P1, Pin2, Alternate3<Input<Floating>>> {
    type ID = u8;

    fn channel() -> Self::ID { 2 }
}

impl Channel<Adc<ADC>> for Pin<P1, Pin3, Alternate3<Input<Floating>>> {
    type ID = u8;

    fn channel() -> Self::ID { 3 }
}

impl Channel<Adc<ADC>> for Pin<P1, Pin4, Alternate3<Input<Floating>>> {
    type ID = u8;

    fn channel() -> Self::ID { 4 }
}

impl Channel<Adc<ADC>> for Pin<P1, Pin5, Alternate3<Input<Floating>>> {
    type ID = u8;

    fn channel() -> Self::ID { 5 }
}

impl Channel<Adc<ADC>> for Pin<P1, Pin6, Alternate3<Input<Floating>>> {
    type ID = u8;

    fn channel() -> Self::ID { 6 }
}

impl Channel<Adc<ADC>> for Pin<P1, Pin7, Alternate3<Input<Floating>>> {
    type ID = u8;

    fn channel() -> Self::ID { 7 }
}

impl Channel<Adc<ADC>> for Pin<P5, Pin0, Alternate3<Input<Floating>>> {
    type ID = u8;

    fn channel() -> Self::ID { 8 }
}

impl Channel<Adc<ADC>> for Pin<P5, Pin1, Alternate3<Input<Floating>>> {
    type ID = u8;

    fn channel() -> Self::ID { 9 }
}

impl Channel<Adc<ADC>> for Pin<P5, Pin2, Alternate3<Input<Floating>>> {
    type ID = u8;

    fn channel() -> Self::ID { 10 }
}

impl Channel<Adc<ADC>> for Pin<P5, Pin3, Alternate3<Input<Floating>>> {
    type ID = u8;

    fn channel() -> Self::ID { 11 }
}

pub struct Adc<ADC> {
    adc_reg: ADC,
}

pub struct AdcConfig {
    adc: ADC,
    clock_source: ClockSource,
    clock_divider: ClockDivider,
    predivider: Predivider,
    resolution: Resolution,
    sampling_rate: SamplingRate,
    sample_time: SampleTime,
}

impl AdcConfig {
    pub fn new(
        adc: ADC,
        clock_source: ClockSource,
        clock_divider: ClockDivider,
        predivider: Predivider,
        resolution: Resolution,
        sampling_rate: SamplingRate,
        sample_time: SampleTime,
    ) -> AdcConfig {
        AdcConfig {
            adc,
            clock_source: clock_source,
            clock_divider: clock_divider,
            predivider: predivider,
            resolution: resolution,
            sampling_rate: sampling_rate,
            sample_time: sample_time,
        }
    }

    pub fn config_hw(self) -> Adc<ADC> {
        let adc_reg = self.adc;

        adc_reg.adcctl0.modify(|_, w| w.adcenc().adcenc_0()
                                                .adcon().adcon_0()
                                                .adcsc().adcsc_0());

        let adcsht = self.sample_time.adcsht();
        adc_reg.adcctl0.modify(|_, w| w.adcsht().bits(adcsht));

        let adcssel = self.clock_source.adcssel();
        adc_reg.adcctl1.modify(|_, w| w.adcssel().bits(adcssel));

        let adcdiv = self.clock_divider.adcdiv();
        adc_reg.adcctl1.modify(|_, w| w.adcdiv().bits(adcdiv));

        let adcpdiv = self.predivider.adcpdiv();
        adc_reg.adcctl2.modify(|_, w| w.adcpdiv().bits(adcpdiv));

        let adcres = self.resolution.adcres();
        adc_reg.adcctl2.modify(|_, w| w.adcres().bits(adcres));

        let adcsr = self.sampling_rate.adcsr();
        adc_reg.adcctl2.modify(|_, w| w.adcsr().bit(adcsr));

        Adc { adc_reg }
    }
}

impl Adc<ADC> {
    pub fn new(adc: ADC) -> Adc<ADC> {
        Adc { adc_reg: adc }
    }

    pub fn adc_enable(&self) {
        self.adc_reg.adcctl0.modify(|_, w| w.adcon().adcon_1());
    }

    pub fn adc_disable(&self) {
        self.adc_reg.adcctl0.modify(|_, w| 
            w.adcon().adcon_0()
            .adcenc().adcenc_0());
    }

    pub fn adc_start_conversion(&self) {
        self.adc_reg
            .adcctl0
            .modify(|_, w| w.adcenc().adcenc_1().adcsc().adcsc_1());
    }

    pub fn adc_is_busy(&self) -> bool {
        return self.adc_reg.adcctl1.read().adcbusy().bit_is_set();
    }

    pub fn adc_get_result(&self) -> u16 {
        return self.adc_reg.adcmem0.read().bits();
    }

    pub fn adc_set_pin<PIN>(&self, _pin: &PIN)
    where PIN: Channel<Adc<ADC>, ID=u8> {
        self.adc_reg.adcmctl0.modify(|_, w| w.adcinch().bits(PIN::channel()));
    }
}

impl<WORD, PIN> OneShot<Adc<ADC>, WORD, PIN> for Adc<ADC>
where
    WORD: From<u16>,
    PIN: Channel<Adc<ADC>, ID = u8>,
{
    type Error = ();

    fn read(&mut self, _pin: &mut PIN ) -> nb::Result<WORD, Self::Error> {
        self.adc_disable();
        self.adc_set_pin(_pin);
        self.adc_enable();

        self.adc_start_conversion();
        while self.adc_is_busy() {}
        let result = self.adc_get_result();

        Ok(result.into())
    }
}
