use msp430fr2355::ADC;

pub struct Adc(ADC);

impl Adc {
    pub fn new(adc: ADC) -> Adc {
        Adc(adc)
    }


}