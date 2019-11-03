use crate::hw_traits::gpio::{GpioPeriph, IntrPeriph};
use crate::pmm::Pmm;
use core::marker::PhantomData;
use embedded_hal::digital::v2::{InputPin, OutputPin, StatefulOutputPin};
use msp430fr2355 as pac;
use pac::{p1, p2, p3, p4, p5, p6};

trait BitsExt {
    fn set(self, shift: u8) -> Self;
    fn clear(self, shift: u8) -> Self;
    fn check(self, shift: u8) -> Self;
    fn set_mask(self, mask: Self) -> Self;
    fn clear_mask(self, mask: Self) -> Self;
}

impl BitsExt for u8 {
    fn set(self, shift: u8) -> Self {
        self | (1 << shift)
    }

    fn clear(self, shift: u8) -> Self {
        self & !(1 << shift)
    }

    fn check(self, shift: u8) -> Self {
        self & (1 << shift)
    }

    fn set_mask(self, mask: Self) -> Self {
        self | mask
    }

    fn clear_mask(self, mask: Self) -> Self {
        self & !mask
    }
}

pub trait GpioPin {
    type Periph: GpioPeriph;

    fn pin() -> u8;
}

pub trait PinNumber {
    fn num() -> u8;
}
pub trait UnderSeven: PinNumber {}
pub trait UnderFive: PinNumber {}

pub struct Pin0;
impl PinNumber for Pin0 {
    fn num() -> u8 {
        0
    }
}
impl UnderSeven for Pin0 {}
impl UnderFive for Pin0 {}

pub struct Pin1;
impl PinNumber for Pin1 {
    fn num() -> u8 {
        1
    }
}
impl UnderSeven for Pin1 {}
impl UnderFive for Pin1 {}

pub struct Pin2;
impl PinNumber for Pin2 {
    fn num() -> u8 {
        2
    }
}
impl UnderSeven for Pin2 {}
impl UnderFive for Pin2 {}

pub struct Pin3;
impl PinNumber for Pin3 {
    fn num() -> u8 {
        3
    }
}
impl UnderSeven for Pin3 {}
impl UnderFive for Pin3 {}

pub struct Pin4;
impl PinNumber for Pin4 {
    fn num() -> u8 {
        4
    }
}
impl UnderSeven for Pin4 {}
impl UnderFive for Pin4 {}

pub struct Pin5;
impl PinNumber for Pin5 {
    fn num() -> u8 {
        5
    }
}
impl UnderSeven for Pin5 {}

pub struct Pin6;
impl PinNumber for Pin6 {
    fn num() -> u8 {
        6
    }
}
impl UnderSeven for Pin6 {}

pub struct Pin7;
impl PinNumber for Pin7 {
    fn num() -> u8 {
        7
    }
}

pub struct Port1<P>(PhantomData<P>);
impl<P: PinNumber> GpioPin for Port1<P> {
    type Periph = pac::p1::RegisterBlock;

    fn pin() -> u8 {
        P::num()
    }
}

pub struct Port2<P>(PhantomData<P>);
impl<P: PinNumber> GpioPin for Port2<P> {
    type Periph = pac::p2::RegisterBlock;

    fn pin() -> u8 {
        P::num()
    }
}

pub struct Port3<P>(PhantomData<P>);
impl<P: PinNumber> GpioPin for Port3<P> {
    type Periph = pac::p3::RegisterBlock;

    fn pin() -> u8 {
        P::num()
    }
}

pub struct Port4<P>(PhantomData<P>);
impl<P: PinNumber> GpioPin for Port4<P> {
    type Periph = pac::p4::RegisterBlock;

    fn pin() -> u8 {
        P::num()
    }
}

pub struct Port5<P>(PhantomData<P>);
impl<P: UnderFive> GpioPin for Port5<P> {
    type Periph = pac::p5::RegisterBlock;

    fn pin() -> u8 {
        P::num()
    }
}

pub struct Port6<P>(PhantomData<P>);
impl<P: UnderSeven> GpioPin for Port6<P> {
    type Periph = pac::p6::RegisterBlock;

    fn pin() -> u8 {
        P::num()
    }
}

// Conversion marker traits
pub trait ConvertToOutput {}
pub trait ConvertToInput {}

// Unknown state
pub struct Unknown;
impl ConvertToInput for Unknown {}
impl ConvertToOutput for Unknown {}
pub trait Known {}

// Pin direction
pub struct Output;
impl ConvertToInput for Output {}
pub struct Input<PULL>(PhantomData<PULL>);
impl<PULL> ConvertToOutput for Input<PULL> {}

// Pin input pull state
pub struct Pullup;
impl Known for Pullup {}
pub struct Pulldown;
impl Known for Pulldown {}
pub struct Floating;
impl Known for Floating {}

// Pin PMM lock state
pub struct Locked;
pub struct Unlocked;

pub struct Pin<PIN: GpioPin, DIR, LOCK> {
    _pin: PhantomData<PIN>,
    _dir: PhantomData<DIR>,
    _lock: PhantomData<LOCK>,
}

macro_rules! make_pin {
    () => {
        Pin {
            _pin: PhantomData,
            _dir: PhantomData,
            _lock: PhantomData,
        }
    };
}

pub struct Pxout<P: GpioPeriph>(PhantomData<P>);
pub struct Pxdir<P: GpioPeriph>(PhantomData<P>);
pub struct Pxint<P: GpioPeriph>(PhantomData<P>);

impl<PIN: GpioPin, PULL, LOCK> Pin<PIN, Input<PULL>, LOCK> {
    pub fn pulldown(self, _pxout: &mut Pxout<PIN::Periph>) -> Pin<PIN, Input<Pulldown>, LOCK> {
        let p = PIN::Periph::steal();
        p.pxout_mod(|b| b.clear(PIN::pin()));
        p.pxren_mod(|b| b.set(PIN::pin()));
        make_pin!()
    }

    pub fn pullup(self, _pxout: &mut Pxout<PIN::Periph>) -> Pin<PIN, Input<Pullup>, LOCK> {
        let p = PIN::Periph::steal();
        p.pxout_mod(|b| b.set(PIN::pin()));
        p.pxren_mod(|b| b.set(PIN::pin()));
        make_pin!()
    }

    pub fn float(self, _pxout: &mut Pxout<PIN::Periph>) -> Pin<PIN, Input<Floating>, LOCK> {
        let p = PIN::Periph::steal();
        p.pxren_mod(|b| b.clear(PIN::pin()));
        make_pin!()
    }
}

impl<PIN: GpioPin, PULL: Known> Pin<PIN, Input<PULL>, Unlocked>
where
    PIN::Periph: IntrPeriph,
{
    pub fn enable_interrupt_rising_edge(&mut self, _pxint: &mut Pxint<PIN::Periph>) {
        let p = PIN::Periph::steal();
        p.pxies_mod(|b| b.clear(PIN::pin()));
        p.pxifg_mod(|b| b.clear(PIN::pin()));
        p.pxie_mod(|b| b.set(PIN::pin()));
    }

    pub fn enable_interrupt_falling_edge(&mut self, _pxint: &mut Pxint<PIN::Periph>) {
        let p = PIN::Periph::steal();
        p.pxies_mod(|b| b.set(PIN::pin()));
        p.pxifg_mod(|b| b.clear(PIN::pin()));
        p.pxie_mod(|b| b.set(PIN::pin()));
    }

    pub fn disable_interrupt(&mut self, _pxint: &mut Pxint<PIN::Periph>) {
        let p = PIN::Periph::steal();
        p.pxie_mod(|b| b.clear(PIN::pin()));
    }
}

impl<PIN: GpioPin, DIR: ConvertToOutput, LOCK> Pin<PIN, DIR, LOCK> {
    pub fn to_output(self, _pxdir: &mut Pxdir<PIN::Periph>) -> Pin<PIN, Output, LOCK> {
        let p = PIN::Periph::steal();
        p.pxdir_mod(|b| b.set(PIN::pin()));
        make_pin!()
    }
}

impl<PIN: GpioPin, DIR: ConvertToInput, LOCK> Pin<PIN, DIR, LOCK> {
    pub fn to_input(self, _pxdir: &mut Pxdir<PIN::Periph>) -> Pin<PIN, Output, LOCK> {
        let p = PIN::Periph::steal();
        p.pxdir_mod(|b| b.clear(PIN::pin()));
        make_pin!()
    }
}

impl<PIN: GpioPin, DIR> Pin<PIN, DIR, Locked> {
    pub fn unlock(self, _pmm: &Pmm) -> Pin<PIN, DIR, Unlocked> {
        make_pin!()
    }
}

impl<PIN: GpioPin> Pin<PIN, Output, Unlocked> {
    pub fn proxy<'out: 'a + 'b, 'a, 'b>(
        &'a mut self,
        _pxout: &'b mut Pxout<PIN::Periph>,
    ) -> OutputPinProxy<'out, PIN> {
        OutputPinProxy(PhantomData, PhantomData)
    }
}

impl<PIN: GpioPin, PULL: Known> InputPin for Pin<PIN, Input<PULL>, Unlocked> {
    type Error = void::Void;

    fn is_high(&self) -> Result<bool, Self::Error> {
        let p = PIN::Periph::steal();
        Ok(p.pxin_rd().check(PIN::pin()) != 0)
    }

    fn is_low(&self) -> Result<bool, Self::Error> {
        let p = PIN::Periph::steal();
        Ok(p.pxin_rd().check(PIN::pin()) == 0)
    }
}

pub struct OutputPinProxy<'out, PIN: GpioPin>(PhantomData<&'out u8>, PhantomData<PIN>);

impl<'out, PIN: GpioPin> OutputPin for OutputPinProxy<'out, PIN> {
    type Error = void::Void;

    fn set_low(&mut self) -> Result<(), Self::Error> {
        let p = PIN::Periph::steal();
        p.pxout_mod(|b| b.clear(PIN::pin()));
        Ok(())
    }

    fn set_high(&mut self) -> Result<(), Self::Error> {
        let p = PIN::Periph::steal();
        p.pxout_mod(|b| b.set(PIN::pin()));
        Ok(())
    }
}

impl<'out, PIN: GpioPin> StatefulOutputPin for OutputPinProxy<'out, PIN> {
    fn is_set_high(&self) -> Result<bool, Self::Error> {
        let p = PIN::Periph::steal();
        Ok(p.pxout_rd().check(PIN::pin()) != 0)
    }

    fn is_set_low(&self) -> Result<bool, Self::Error> {
        let p = PIN::Periph::steal();
        Ok(p.pxout_rd().check(PIN::pin()) == 0)
    }
}

pub trait GpioExt {
    type Parts;

    fn constrain(self) -> Self::Parts;
}

macro_rules! impl_gpio_ext {
    ($px:ident, $PxParts:ident, $Portx:ident) => {
        pub struct $PxParts {
            pub pin0: $Portx<Pin0>,
            pub pin1: $Portx<Pin1>,
            pub pin2: $Portx<Pin2>,
            pub pin3: $Portx<Pin3>,
            pub pin4: $Portx<Pin4>,
            pub pin5: $Portx<Pin5>,
            pub pin6: $Portx<Pin6>,
            pub pin7: $Portx<Pin7>,

            pub pxint: Pxint<$px::RegisterBlock>,
            pub pxout: Pxout<$px::RegisterBlock>,
            pub pxdir: Pxdir<$px::RegisterBlock>,
        }

        impl GpioExt for $px::RegisterBlock {
            type Parts = $PxParts;

            fn constrain(self) -> Self::Parts {
                Self::Parts {
                    pin0: $Portx(PhantomData),
                    pin1: $Portx(PhantomData),
                    pin2: $Portx(PhantomData),
                    pin3: $Portx(PhantomData),
                    pin4: $Portx(PhantomData),
                    pin5: $Portx(PhantomData),
                    pin6: $Portx(PhantomData),
                    pin7: $Portx(PhantomData),

                    pxint: Pxint(PhantomData),
                    pxout: Pxout(PhantomData),
                    pxdir: Pxdir(PhantomData),
                }
            }
        }
    };
}

impl_gpio_ext!(p1, P1Parts, Port1);
impl_gpio_ext!(p2, P2Parts, Port2);
impl_gpio_ext!(p3, P3Parts, Port3);
impl_gpio_ext!(p4, P4Parts, Port4);
impl_gpio_ext!(p5, P5Parts, Port5);
impl_gpio_ext!(p6, P6Parts, Port6);
