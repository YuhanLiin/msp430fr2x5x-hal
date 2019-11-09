use crate::bits::BitsExt;
use crate::hw_traits::gpio::{GpioPeriph, IntrPeriph};
use crate::pmm::Pmm;
use core::marker::PhantomData;
use embedded_hal::digital::v2::{InputPin, OutputPin, StatefulOutputPin, ToggleableOutputPin};
use msp430fr2355 as pac;
use pac::{p1, p2, p3, p4, p5, p6, P1, P2, P3, P4, P5, P6};

/// Trait that encompasses all `Portx<Pinx>` types for specifying a pin number on a GPIO port.
/// Use `Pin<P: PortPinNum, DIR>` to specify any GPIO pin on the chip.
pub trait PortPinNum {
    /// GPIO port type
    type Periph: GpioPeriph;

    /// GPIO pin number
    fn pin() -> u8;
}

/// Trait that encompasses all `Pinx` types for specifying a pin number.
/// Use `Pin<Port1<P: PinNum>, DIR>` to specify any P1 pin.
pub trait PinNum {
    /// Pin number
    fn num() -> u8;
}

/// Marker trait for pin numbers 0 to 6.
pub trait UnderSeven: PinNum {}
/// Marker trait for pin numbers 0 to 4.
pub trait UnderFive: PinNum {}

/// Pin number 0
pub struct Pin0;
impl PinNum for Pin0 {
    fn num() -> u8 {
        0
    }
}
impl UnderSeven for Pin0 {}
impl UnderFive for Pin0 {}

/// Pin number 1
pub struct Pin1;
impl PinNum for Pin1 {
    fn num() -> u8 {
        1
    }
}
impl UnderSeven for Pin1 {}
impl UnderFive for Pin1 {}

/// Pin number 2
pub struct Pin2;
impl PinNum for Pin2 {
    fn num() -> u8 {
        2
    }
}
impl UnderSeven for Pin2 {}
impl UnderFive for Pin2 {}

/// Pin number 3
pub struct Pin3;
impl PinNum for Pin3 {
    fn num() -> u8 {
        3
    }
}
impl UnderSeven for Pin3 {}
impl UnderFive for Pin3 {}

/// Pin number 4
pub struct Pin4;
impl PinNum for Pin4 {
    fn num() -> u8 {
        4
    }
}
impl UnderSeven for Pin4 {}
impl UnderFive for Pin4 {}

/// Pin number 5
pub struct Pin5;
impl PinNum for Pin5 {
    fn num() -> u8 {
        5
    }
}
impl UnderSeven for Pin5 {}

/// Pin number 6
pub struct Pin6;
impl PinNum for Pin6 {
    fn num() -> u8 {
        6
    }
}
impl UnderSeven for Pin6 {}

/// Pin number 7
pub struct Pin7;
impl PinNum for Pin7 {
    fn num() -> u8 {
        7
    }
}

pub trait Port5 {
    type pin0: PortPinNum;
    type pin1: PortPinNum;
    type pin2: PortPinNum;
    type pin3: PortPinNum;
    type pin4: PortPinNum;
}

pub trait Port7: Port5 {
    type pin5: PortPinNum;
    type pin6: PortPinNum;
}

pub trait Port: Port7 {
    type pin7: PortPinNum;
}

/// `PortPin` type for GPIO port P1, which contain pins 0 to 7.
pub struct Port1<P>(PhantomData<P>);
impl<P: PinNum> PortPinNum for Port1<P> {
    type Periph = pac::p1::RegisterBlock;

    fn pin() -> u8 {
        P::num()
    }
}

/// `PortPin` type for GPIO port P2, which contain pins 0 to 7.
pub struct Port2<P>(PhantomData<P>);
impl<P: PinNum> PortPinNum for Port2<P> {
    type Periph = pac::p2::RegisterBlock;

    fn pin() -> u8 {
        P::num()
    }
}

/// `PortPin` type for GPIO port P3, which contain pins 0 to 7.
pub struct Port3<P>(PhantomData<P>);
impl<P: PinNum> PortPinNum for Port3<P> {
    type Periph = pac::p3::RegisterBlock;

    fn pin() -> u8 {
        P::num()
    }
}

/// `PortPin` type for GPIO port P4, which contain pins 0 to 7.
pub struct Port4<P>(PhantomData<P>);
impl<P: PinNum> PortPinNum for Port4<P> {
    type Periph = pac::p4::RegisterBlock;

    fn pin() -> u8 {
        P::num()
    }
}

/// `PortPin` type for GPIO port P5, which contain pins 0 to 4.
/// To specify a pin on P5, use `Pin<Port5<P: UnderFive>, DIR>`.
pub struct Port5<P>(PhantomData<P>);
impl<P: UnderFive> PortPinNum for Port5<P> {
    type Periph = pac::p5::RegisterBlock;

    fn pin() -> u8 {
        P::num()
    }
}

/// `PortPin` type for GPIO port P6, which contain pins 0 to 6.
/// To specify a pin on P6, use `Pin<Port6<P: UnderSeven>, DIR>`.
pub struct Port6<P>(PhantomData<P>);
impl<P: UnderSeven> PortPinNum for Port6<P> {
    type Periph = pac::p6::RegisterBlock;

    fn pin() -> u8 {
        P::num()
    }
}

#[doc(hidden)]
pub trait ConvertToOutput {}
#[doc(hidden)]
pub trait ConvertToInput {}
#[doc(hidden)]
pub trait Known {}

/// Typestate for an unknown state
pub struct Unknown;
impl ConvertToInput for Unknown {}
impl ConvertToOutput for Unknown {}

/// Direction typestate for GPIO output
pub struct Output;
impl ConvertToInput for Output {}

/// Direction typestate for GPIO input.
/// The type parameter specifies pull direction of input.
pub struct Input<PULL>(PhantomData<PULL>);
impl<PULL> ConvertToOutput for Input<PULL> {}

/// Pull typestate for pullup inputs
pub struct Pullup;
impl Known for Pullup {}

/// Pull typestate for pulldown inputs
pub struct Pulldown;
impl Known for Pulldown {}

/// Pull typestate for floating inputs
pub struct Floating;
impl Known for Floating {}

/// PMM lock typestate for a locked GPIO
pub struct Locked;

/// PMM lock typestate for an unlocked GPIO
pub struct Unlocked;

/// A single GPIO pin on the chip.
pub struct Pin<PIN: PortPinNum, DIR> {
    _pin: PhantomData<PIN>,
    _dir: PhantomData<DIR>,
}

macro_rules! make_pin {
    () => {
        Pin {
            _pin: PhantomData,
            _dir: PhantomData,
        }
    };
}

/// Contention token for the PxOUT register.
/// Used to prevent races when accessing the PxOUT register from different pins on the same port.
pub struct Pxout<P: GpioPeriph>(PhantomData<P>);

/// Contention token for the PxDIR register.
/// Used to prevent races when accessing the PxDIR register from different pins on the same port.
pub struct Pxdir<P: GpioPeriph>(PhantomData<P>);

/// Contention token for the interrupt registers.
/// Used to prevent races when accessing the inerrupt registers from different pins on the same port.
pub struct Pxint<P: GpioPeriph>(PhantomData<P>);

impl<PIN: PortPinNum, PULL> Pin<PIN, Input<PULL>> {
    /// Configures pin as pulldown input
    /// This method requires a `Pxout` token because configuring pull direction requires setting
    /// the PxOUT register, which can race with setting an output pin on the same port.
    pub fn pulldown(self, _pxout: &mut Pxout<PIN::Periph>) -> Pin<PIN, Input<Pulldown>> {
        let p = PIN::Periph::steal();
        p.pxout_mod(|b| b.clear(PIN::pin()));
        p.pxren_mod(|b| b.set(PIN::pin()));
        make_pin!()
    }

    /// Configures pin as pullup input
    /// This method requires a `Pxout` token because configuring pull direction requires setting
    /// the PxOUT register, which can race with setting an output pin on the same port.
    pub fn pullup(self, _pxout: &mut Pxout<PIN::Periph>) -> Pin<PIN, Input<Pullup>> {
        let p = PIN::Periph::steal();
        p.pxout_mod(|b| b.set(PIN::pin()));
        p.pxren_mod(|b| b.set(PIN::pin()));
        make_pin!()
    }

    /// Configures pin as floating input
    pub fn float(self, _pxout: &mut Pxout<PIN::Periph>) -> Pin<PIN, Input<Floating>> {
        let p = PIN::Periph::steal();
        p.pxren_mod(|b| b.clear(PIN::pin()));
        make_pin!()
    }
}

impl<PIN: PortPinNum, PULL: Known> Pin<PIN, Input<PULL>>
where
    PIN::Periph: IntrPeriph,
{
    /// Enable rising edge interrupts on the input pin.
    /// Note that changing other GPIO configurations while interrupts are enabled can cause
    /// spurious interrupts.
    pub fn enable_interrupt_rising_edge(&mut self, _pxint: &mut Pxint<PIN::Periph>) {
        let p = PIN::Periph::steal();
        p.pxies_mod(|b| b.clear(PIN::pin()));
        p.pxifg_mod(|b| b.clear(PIN::pin()));
        p.pxie_mod(|b| b.set(PIN::pin()));
    }

    /// Enable falling edge interrupts on the input pin.
    /// Note that changing other GPIO configurations while interrupts are enabled can cause
    /// spurious interrupts.
    pub fn enable_interrupt_falling_edge(&mut self, _pxint: &mut Pxint<PIN::Periph>) {
        let p = PIN::Periph::steal();
        p.pxies_mod(|b| b.set(PIN::pin()));
        p.pxifg_mod(|b| b.clear(PIN::pin()));
        p.pxie_mod(|b| b.set(PIN::pin()));
    }

    /// Disable interrupts on input pin.
    pub fn disable_interrupt(&mut self, _pxint: &mut Pxint<PIN::Periph>) {
        let p = PIN::Periph::steal();
        p.pxie_mod(|b| b.clear(PIN::pin()));
    }
}

impl<PIN: PortPinNum, DIR: ConvertToOutput> Pin<PIN, DIR> {
    /// Configures pin as output
    pub fn to_output(self, _pxdir: &mut Pxdir<PIN::Periph>) -> Pin<PIN, Output> {
        let p = PIN::Periph::steal();
        p.pxdir_mod(|b| b.set(PIN::pin()));
        make_pin!()
    }
}

impl<PIN: PortPinNum, DIR: ConvertToInput> Pin<PIN, DIR> {
    /// Configures pin as input
    pub fn to_input(self, _pxdir: &mut Pxdir<PIN::Periph>) -> Pin<PIN, Input<Unknown>> {
        let p = PIN::Periph::steal();
        p.pxdir_mod(|b| b.clear(PIN::pin()));
        make_pin!()
    }
}

impl<PIN: PortPinNum, DIR> Pin<PIN, DIR> {
    /// "Unlocks" the pin so that I/O can be performed on it.
    /// Unlocking with a `Pmm` ensures that I/O is only done on the pin after the LOCKLPM5 pin has
    /// been set. Otherwise I/O operations won't even work without setting LOCKLPM5.
    pub fn unlock(self, _pmm: &Pmm) -> Pin<PIN, DIR> {
        make_pin!()
    }
}

impl<PIN: PortPinNum> Pin<PIN, Output> {
    /// Use the `Pxout` token to create a output pin "proxy" on which output operations can be
    /// done. The token ensures that different output pin writes on the same port don't race with
    /// each other. We need to do this because unlike ARM, output writes on MSP430 require
    /// read-modify-write, which is not atomic.
    pub fn proxy<'out: 'a + 'b, 'a, 'b>(
        &'a mut self,
        _pxout: &'b mut Pxout<PIN::Periph>,
    ) -> OutputPinProxy<'out, PIN> {
        OutputPinProxy(PhantomData, PhantomData)
    }
}

impl<PIN: PortPinNum, PULL: Known> InputPin for Pin<PIN, Input<PULL>> {
    type Error = void::Void;

    fn is_high(&self) -> Result<bool, Self::Error> {
        let p = PIN::Periph::steal();
        Ok(p.pxin_rd().check(PIN::pin()) != 0)
    }

    fn is_low(&self) -> Result<bool, Self::Error> {
        self.is_high().map(|r| !r)
    }
}

/// Proxy type for an output pin
pub struct OutputPinProxy<'out, PIN: PortPinNum>(PhantomData<&'out u8>, PhantomData<PIN>);

impl<'out, PIN: PortPinNum> OutputPin for OutputPinProxy<'out, PIN> {
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

impl<'out, PIN: PortPinNum> StatefulOutputPin for OutputPinProxy<'out, PIN> {
    fn is_set_high(&self) -> Result<bool, Self::Error> {
        let p = PIN::Periph::steal();
        Ok(p.pxout_rd().check(PIN::pin()) != 0)
    }

    fn is_set_low(&self) -> Result<bool, Self::Error> {
        self.is_set_high().map(|r| !r)
    }
}

impl<'out, PIN: PortPinNum> ToggleableOutputPin for OutputPinProxy<'out, PIN> {
    type Error = void::Void;

    fn toggle(&mut self) -> Result<(), Self::Error> {
        let p = PIN::Periph::steal();
        p.pxout_mod(|b| b ^ (1 << PIN::pin()));
        Ok(())
    }
}

macro_rules! impl_gpio_ext {
    ($Px:ident, $px:ident, $PxParts:ident, $Portx:ident $(, [$pin5:ident: $dir5:ident, $pin6:ident: $dir6:ident $(, $pin7:ident: $dir7:ident)?])?) => {
        /// GPIO parts
        pub struct $PxParts<DIR0, DIR1, DIR2, DIR3, DIR4 $(, $dir5, $dir6 $(, $dir7)?)?> {
            /// Pin0
            pub pin0: Pin<$Portx<Pin0>, DIR0>,
            /// Pin1
            pub pin1: Pin<$Portx<Pin1>, DIR1>,
            /// Pin2
            pub pin2: Pin<$Portx<Pin2>, DIR2>,
            /// Pin3
            pub pin3: Pin<$Portx<Pin3>, DIR3>,
            /// Pin4
            pub pin4: Pin<$Portx<Pin4>, DIR4>,
            $(
                /// Pin5
                pub $pin5: Pin<$Portx<Pin5>, DIR5>,
                /// Pin6
                pub $pin6: Pin<$Portx<Pin6>, DIR6>,
                $(
                    /// Pin7
                    pub $pin7: Pin<$Portx<Pin7>, DIR7>,
                )?
            )?

            /// Interrupt register contention token
            pub pxint: Pxint<$px::RegisterBlock>,
            /// PxOUT contention token
            pub pxout: Pxout<$px::RegisterBlock>,
            /// PxDIR contention token
            pub pxdir: Pxdir<$px::RegisterBlock>,
        }

        impl<DIR0, DIR1, DIR2, DIR3, DIR4 $(, $dir5, $dir6 $(, $dir7)?)?> $PxParts<DIR0, DIR1, DIR2, DIR3, DIR4 $(, $dir5, $dir6 $(, $dir7)?)?> {
            pub fn new() -> Self {
                Self {
                    pin0: make_pin!(),
                    pin1: make_pin!(),
                    pin2: make_pin!(),
                    pin3: make_pin!(),
                    pin4: make_pin!(),
                    $(
                        $pin5: make_pin!(),
                        $pin6: make_pin!(),
                        $(
                            $pin7: make_pin!(),
                        )?
                    )?

                    pxint: Pxint(PhantomData),
                    pxout: Pxout(PhantomData),
                    pxdir: Pxdir(PhantomData),
                }
            }
        }
    };
}

impl_gpio_ext!(P1, p1, P1Parts, Port1, [pin5: DIR5, pin6: DIR6, pin7: DIR7]);
impl_gpio_ext!(P2, p2, P2Parts, Port2, [pin5: DIR5, pin6: DIR6, pin7: DIR7]);
impl_gpio_ext!(P3, p3, P3Parts, Port3, [pin5: DIR5, pin6: DIR6, pin7: DIR7]);
impl_gpio_ext!(P4, p4, P4Parts, Port4, [pin5: DIR5, pin6: DIR6, pin7: DIR7]);
impl_gpio_ext!(P5, p5, P5Parts, Port5);
impl_gpio_ext!(P6, p6, P6Parts, Port6, [pin5: DIR5, pin6: DIR6]);
