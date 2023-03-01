use embedded_hal::adc::{Channel, OneShot};
use msp430fr2355::ADC;

pub struct Adc<ADC> {
    adc_reg: ADC,
}

impl Adc<ADC> {
    pub fn new(adc: ADC) -> Adc<ADC> {
        Adc { adc_reg: adc }
    }

    pub fn adc_init(&self) {
        self.adc_reg
            .adcctl0
            .write(|w| w.adcenc().adcenc_0().adcon().adcon_0().adcsc().adcsc_0());
        self.adc_reg.adcie.write(|w| unsafe { w.bits(0) });
        self.adc_reg.adcifg.write(|w| unsafe { w.bits(0) });
        self.adc_reg.adcctl1.write(|w| unsafe { w.bits(0) });
        self.adc_reg.adcctl2.write(|w| unsafe { w.bits(0) });
    }

    pub fn adc_set_pin(&self, pin: u8) {
        self.adc_reg.adcmctl0.write(|w| w.adcinch().bits(pin));
    }

    pub fn adc_enable(&self) {
        self.adc_reg.adcctl0.write(|w| w.adcon().adcon_1());
    }

    pub fn adc_start_conversion(&self) {
        self.adc_reg.adcctl0.write(|w| w.adcenc().adcenc_1());
    }

    pub fn adc_is_busy(&self) -> bool {
        return self.adc_reg.adcctl1.read().adcbusy().bit_is_set();
    }

    pub fn adc_get_result(&self) -> u16 {
        return self.adc_reg.adcmem0.read().bits();
    }
}

impl<WORD, PIN> OneShot<Adc<ADC>, WORD, PIN> for Adc<ADC>
where
    WORD: From<u16>,
    PIN: Channel<Adc<ADC>, ID = u8>,
{
    type Error = ();

    fn read(&mut self, _pin: &mut PIN) -> nb::Result<WORD, Self::Error> {
        let chan = 1 << PIN::channel();
        //self.power_up();
        let result = 0xAA55_u16;
        //self.power_down();
        Ok(result.into())
    }
}
