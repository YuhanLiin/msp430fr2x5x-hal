//! GPIO abstractions for configuring and controlling individual GPIO pins.
//!
//! To specify any pin, use the bounds `Pin<PORT: PortNum, PIN: PinNum>`.
//! To specify any pin on a port that supports interrupts, use the bounds `Pin<PORT: IntrPortNum, PIN: PinNum>`.
//! To specify any pin on port Px, use the bounds `Pin<Px, PIN: PinNum>`.
//!
//! Note that interrupts are only supported by the hardware on ports **1 to 4**, so interrupt-related
//! methods are only available on those pins.
//!
//! Pins can be converted to alternate functionalities 1 to 3, but the availability of these
//! conversions on each pin is limited by the hardware capabilities in the [`datasheet`], so not
//! every pin in every configuration can be converted to every alternate functionality.
//!
//! [`datasheet`]: http://www.ti.com/lit/ds/symlink/msp430fr2355.pdf

pub use crate::batch_gpio::*;
use crate::hw_traits::gpio::{GpioPeriph, IntrPeriph};
use crate::util::BitsExt;
use core::convert::Infallible;
use core::marker::PhantomData;
use crate::pac;
pub use pac::{P1, P2, P3, P4, P5, P6};

mod sealed {
    use super::*;

    pub trait SealedPinNum {}
    pub trait SealedGpioFunction {}

    impl SealedPinNum for Pin0 {}
    impl SealedPinNum for Pin1 {}
    impl SealedPinNum for Pin2 {}
    impl SealedPinNum for Pin3 {}
    impl SealedPinNum for Pin4 {}
    impl SealedPinNum for Pin5 {}
    impl SealedPinNum for Pin6 {}
    impl SealedPinNum for Pin7 {}

    impl SealedGpioFunction for Output {}
    impl<PULL> SealedGpioFunction for Input<PULL> {}
}

/// Trait that encompasses all `Pinx` types for specifying a pin number.
pub trait PinNum: sealed::SealedPinNum {
    // Pin number
    #[doc(hidden)]
    const NUM: u8;

    // Bitmask with all zeros except for the bit corresponding to the pin.
    #[doc(hidden)]
    const SET_MASK: u8 = 1 << Self::NUM;
    // Bitmask with all ones except for the bit corresponding to the pin.
    #[doc(hidden)]
    const CLR_MASK: u8 = !Self::SET_MASK;
}

// Don't need to seal, since GpioPeriph is private
/// Marker trait that encompasses all GPIO port types
pub trait PortNum: GpioPeriph {}
impl<PORT: GpioPeriph> PortNum for PORT {}

// Don't need to seal, since PortNum is already sealed
/// Marker trait for all Ports that support interrupts
pub trait IntrPortNum: IntrPeriph {}
impl<PORT: IntrPeriph> IntrPortNum for PORT {}

/// Pin number 0
pub struct Pin0;
impl PinNum for Pin0 {
    const NUM: u8 = 0;
}

/// Pin number 1
pub struct Pin1;
impl PinNum for Pin1 {
    const NUM: u8 = 1;
}

/// Pin number 2
pub struct Pin2;
impl PinNum for Pin2 {
    const NUM: u8 = 2;
}

/// Pin number 3
pub struct Pin3;
impl PinNum for Pin3 {
    const NUM: u8 = 3;
}

/// Pin number 4
pub struct Pin4;
impl PinNum for Pin4 {
    const NUM: u8 = 4;
}

/// Pin number 5
pub struct Pin5;
impl PinNum for Pin5 {
    const NUM: u8 = 5;
}

/// Pin number 6
pub struct Pin6;
impl PinNum for Pin6 {
    const NUM: u8 = 6;
}

/// Pin number 7
pub struct Pin7;
impl PinNum for Pin7 {
    const NUM: u8 = 7;
}

/// Marker trait for GPIO typestates representing pins in GPIO (non-alternate) state
pub trait GpioFunction: sealed::SealedGpioFunction {}

/// Direction typestate for GPIO output
pub struct Output;
impl GpioFunction for Output {}

/// Direction typestate for GPIO input.
/// The type parameter specifies pull direction of input.
pub struct Input<PULL>(PhantomData<PULL>);
impl<PULL> GpioFunction for Input<PULL> {}

/// Pull typestate for pullup inputs
pub struct Pullup;

/// Pull typestate for pulldown inputs
pub struct Pulldown;

/// Pull typestate for floating inputs
pub struct Floating;

/// A single GPIO pin.
pub struct Pin<PORT: PortNum, PIN: PinNum, DIR> {
    _port: PhantomData<PORT>,
    _pin: PhantomData<PIN>,
    _dir: PhantomData<DIR>,
}

macro_rules! make_pin {
    () => {
        Pin {
            _port: PhantomData,
            _pin: PhantomData,
            _dir: PhantomData,
        }
    };

    ($dir:ty) => {
        Pin::<_, _, $dir> {
            _port: PhantomData,
            _pin: PhantomData,
            _dir: PhantomData,
        }
    };
}

impl<PORT: PortNum, PIN: PinNum, PULL> Pin<PORT, PIN, Input<PULL>> {
    /// Configures pin as pulldown input.
    /// This method requires a `Pxout` token because configuring pull direction requires setting
    /// the PxOUT register, which can race with setting an output pin on the same port.
    #[inline]
    pub fn pulldown(self) -> Pin<PORT, PIN, Input<Pulldown>> {
        let p = unsafe { PORT::steal() };
        p.pxout_clear(PIN::CLR_MASK);
        p.pxren_set(PIN::SET_MASK);
        make_pin!()
    }

    /// Configures pin as pullup input.
    /// This method requires a `Pxout` token because configuring pull direction requires setting
    /// the PxOUT register, which can race with setting an output pin on the same port.
    #[inline]
    pub fn pullup(self) -> Pin<PORT, PIN, Input<Pullup>> {
        let p = unsafe { PORT::steal() };
        p.pxout_set(PIN::SET_MASK);
        p.pxren_set(PIN::SET_MASK);
        make_pin!()
    }

    /// Configures pin as floating input
    #[inline]
    pub fn floating(self) -> Pin<PORT, PIN, Input<Floating>> {
        let p = unsafe { PORT::steal() };
        p.pxren_clear(PIN::CLR_MASK);
        make_pin!()
    }
}

impl<PORT: IntrPortNum, PIN: PinNum, PULL> Pin<PORT, PIN, Input<PULL>> {
    /// Set interrupt trigger to rising edge and clear interrupt flag.
    #[inline]
    pub fn select_rising_edge_trigger(&mut self) -> &mut Self {
        let p = unsafe { PORT::steal() };
        p.pxies_clear(PIN::CLR_MASK);
        p.pxifg_clear(PIN::CLR_MASK);
        self
    }

    /// Set interrupt trigger to falling edge, the default, and clear interrupt flag.
    #[inline]
    pub fn select_falling_edge_trigger(&mut self) -> &mut Self {
        let p = unsafe { PORT::steal() };
        p.pxies_set(PIN::SET_MASK);
        p.pxifg_clear(PIN::CLR_MASK);
        self
    }

    /// Enable interrupts on input pin.
    /// Note that changing other GPIO configurations while interrupts are enabled can cause
    /// spurious interrupts.
    #[inline]
    pub fn enable_interrupts(&mut self) -> &mut Self {
        let p = unsafe { PORT::steal() };
        p.pxie_set(PIN::SET_MASK);
        self
    }

    /// Disable interrupts on input pin.
    #[inline]
    pub fn disable_interrupt(&mut self) -> &mut Self {
        let p = unsafe { PORT::steal() };
        p.pxie_clear(PIN::CLR_MASK);
        self
    }

    /// Set interrupt flag high, triggering an ISR if interrupts are enabled.
    #[inline]
    pub fn set_ifg(&mut self) -> &mut Self {
        let p = unsafe { PORT::steal() };
        p.pxifg_set(PIN::SET_MASK);
        self
    }

    /// Clear interrupt flag.
    #[inline]
    pub fn clear_ifg(&mut self) -> &mut Self {
        let p = unsafe { PORT::steal() };
        p.pxifg_clear(PIN::CLR_MASK);
        self
    }

    /// Wait for interrupt flag to go high nonblockingly. Clear the flag if high.
    #[inline]
    pub fn wait_for_ifg(&mut self) -> nb::Result<(), Infallible> {
        let p = unsafe { PORT::steal() };
        if p.pxifg_rd().check(PIN::NUM) != 0 {
            p.pxifg_clear(PIN::CLR_MASK);
            Ok(())
        } else {
            Err(nb::Error::WouldBlock)
        }
    }
}

/// Interrupt vector register used to determine which pin caused a port ISR
pub struct PxIV<PORT: PortNum>(PhantomData<PORT>);

impl<PORT: IntrPortNum> PxIV<PORT> {
    /// When called inside an ISR, returns the pin number of the highest priority interrupt flag
    /// that's currently enabled. Automatically clears the same flag. For a given port, the lowest
    /// numbered pin has the highest interrupt priority.
    #[inline]
    pub fn get_interrupt_vector(&mut self) -> GpioVector {
        let p = unsafe { PORT::steal() };
        match p.pxiv_rd() {
            0 => GpioVector::NoIsr,
            2 => GpioVector::Pin0Isr,
            4 => GpioVector::Pin1Isr,
            6 => GpioVector::Pin2Isr,
            8 => GpioVector::Pin3Isr,
            10 => GpioVector::Pin4Isr,
            12 => GpioVector::Pin5Isr,
            14 => GpioVector::Pin6Isr,
            16 => GpioVector::Pin7Isr,
            _ => unsafe { core::hint::unreachable_unchecked() },
        }
    }
}

/// Indicates which pin on the GPIO port caused the ISR.
pub enum GpioVector {
    /// No ISR
    NoIsr,
    /// ISR caused by pin 0
    Pin0Isr,
    /// ISR caused by pin 1
    Pin1Isr,
    /// ISR caused by pin 2
    Pin2Isr,
    /// ISR caused by pin 3
    Pin3Isr,
    /// ISR caused by pin 4
    Pin4Isr,
    /// ISR caused by pin 5
    Pin5Isr,
    /// ISR caused by pin 6
    Pin6Isr,
    /// ISR caused by pin 7
    Pin7Isr,
}

impl<PORT: PortNum, PIN: PinNum, PULL> Pin<PORT, PIN, Input<PULL>> {
    /// Configures pin as output
    #[inline]
    pub fn to_output(self) -> Pin<PORT, PIN, Output> {
        let p = unsafe { PORT::steal() };
        p.pxdir_set(PIN::SET_MASK);
        make_pin!()
    }
}

impl<PORT: PortNum, PIN: PinNum> Pin<PORT, PIN, Output> {
    /// Configures pin as floating input
    #[inline]
    pub fn to_input_floating(self) -> Pin<PORT, PIN, Input<Floating>> {
        let p = unsafe { PORT::steal() };
        p.pxdir_clear(PIN::CLR_MASK);
        make_pin!(Input<Floating>).floating()
    }

    /// Configures pin as floating input
    #[inline]
    pub fn to_input_pullup(self) -> Pin<PORT, PIN, Input<Pullup>> {
        let p = unsafe { PORT::steal() };
        p.pxdir_clear(PIN::CLR_MASK);
        make_pin!(Input<Floating>).pullup()
    }

    /// Configures pin as floating input
    #[inline]
    pub fn to_input_pulldown(self) -> Pin<PORT, PIN, Input<Pulldown>> {
        let p = unsafe { PORT::steal() };
        p.pxdir_clear(PIN::CLR_MASK);
        make_pin!(Input<Floating>).pulldown()
    }
}

/// GPIO parts for a specific port, including all 8 pins.
pub struct Parts<PORT: PortNum, DIR0, DIR1, DIR2, DIR3, DIR4, DIR5, DIR6, DIR7> {
    /// Pin0
    pub pin0: Pin<PORT, Pin0, DIR0>,
    /// Pin1
    pub pin1: Pin<PORT, Pin1, DIR1>,
    /// Pin2
    pub pin2: Pin<PORT, Pin2, DIR2>,
    /// Pin3
    pub pin3: Pin<PORT, Pin3, DIR3>,
    /// Pin4
    pub pin4: Pin<PORT, Pin4, DIR4>,
    /// Pin5
    pub pin5: Pin<PORT, Pin5, DIR5>,
    /// Pin6
    pub pin6: Pin<PORT, Pin6, DIR6>,
    /// Pin7
    pub pin7: Pin<PORT, Pin7, DIR7>,
    /// Interrupt vector register
    pub pxiv: PxIV<PORT>,
}

impl<PORT: PortNum, DIR0, DIR1, DIR2, DIR3, DIR4, DIR5, DIR6, DIR7>
    Parts<PORT, DIR0, DIR1, DIR2, DIR3, DIR4, DIR5, DIR6, DIR7>
{
    /// Converts all parts into a GPIO batch so the entire port can be configured at once
    #[inline]
    pub fn batch(self) -> Batch<PORT, DIR0, DIR1, DIR2, DIR3, DIR4, DIR5, DIR6, DIR7> {
        Batch::create()
    }

    #[inline]
    pub(super) fn new() -> Self {
        Self {
            pin0: make_pin!(),
            pin1: make_pin!(),
            pin2: make_pin!(),
            pin3: make_pin!(),
            pin4: make_pin!(),
            pin5: make_pin!(),
            pin6: make_pin!(),
            pin7: make_pin!(),
            pxiv: PxIV(PhantomData),
        }
    }
}

// Trait will not be used as a bound outside the HAL, since it's only used as an associated type
// bound inside the HAL, so just keep it hidden
#[doc(hidden)]
pub trait ChangeSelectBits {
    fn set_sel0(&mut self);
    fn set_sel1(&mut self);
    fn clear_sel0(&mut self);
    fn clear_sel1(&mut self);
    fn flip_selc(&mut self);
}

// Methods for managing sel1, sel0, and selc registers
impl<PORT: PortNum, PIN: PinNum, DIR> ChangeSelectBits for Pin<PORT, PIN, DIR> {
    #[inline]
    fn set_sel0(&mut self) {
        let p = unsafe { PORT::steal() };
        p.pxsel0_set(PIN::SET_MASK);
    }

    #[inline]
    fn set_sel1(&mut self) {
        let p = unsafe { PORT::steal() };
        p.pxsel1_set(PIN::SET_MASK);
    }

    #[inline]
    fn clear_sel0(&mut self) {
        let p = unsafe { PORT::steal() };
        p.pxsel0_clear(PIN::CLR_MASK);
    }

    #[inline]
    fn clear_sel1(&mut self) {
        let p = unsafe { PORT::steal() };
        p.pxsel1_clear(PIN::CLR_MASK);
    }

    #[inline]
    fn flip_selc(&mut self) {
        let p = unsafe { PORT::steal() };
        // Change both sel0 and sel1 bits at once
        p.pxselc_wr(0u8.set(PIN::NUM));
    }
}

/// Typestate for GPIO alternate function 1
pub struct Alternate1<DIR>(PhantomData<DIR>);

/// Typestate for GPIO alternate function 2
pub struct Alternate2<DIR>(PhantomData<DIR>);

/// Typestate for GPIO alternate function 3
pub struct Alternate3<DIR>(PhantomData<DIR>);

// Sealing these traits takes a lot of work, and I'll never add any items in the future, so they
// are unsealed
/// Marker trait for all Pins that have alternate function 1 available
pub trait ToAlternate1 {}
/// Marker trait for all Pins that have alternate function 2 available
pub trait ToAlternate2 {}
/// Marker trait for all Pins that have alternate function 3 available
pub trait ToAlternate3 {}

impl<PORT: PortNum, PIN: PinNum, DIR: GpioFunction> Pin<PORT, PIN, DIR>
where
    Self: ToAlternate1,
{
    /// Convert pin to GPIO alternate function 1
    #[inline]
    pub fn to_alternate1(mut self) -> Pin<PORT, PIN, Alternate1<DIR>> {
        self.set_sel0();
        make_pin!()
    }
}

impl<PORT: PortNum, PIN: PinNum, DIR: GpioFunction> Pin<PORT, PIN, DIR>
where
    Self: ToAlternate2,
{
    /// Convert pin to GPIO alternate function 2
    #[inline]
    pub fn to_alternate2(mut self) -> Pin<PORT, PIN, Alternate2<DIR>> {
        self.set_sel1();
        make_pin!()
    }
}

impl<PORT: PortNum, PIN: PinNum, DIR: GpioFunction> Pin<PORT, PIN, DIR>
where
    Self: ToAlternate3,
{
    /// Convert pin to GPIO alternate function 3
    #[inline]
    pub fn to_alternate3(mut self) -> Pin<PORT, PIN, Alternate3<DIR>> {
        self.flip_selc();
        make_pin!()
    }
}

// sel0 = 1, sel1 = 0
impl<PORT: PortNum, PIN: PinNum, DIR> Pin<PORT, PIN, Alternate1<DIR>> {
    /// Convert pin to GPIO function
    #[inline]
    pub fn to_gpio(mut self) -> Pin<PORT, PIN, DIR> {
        self.clear_sel0();
        make_pin!()
    }
}

impl<PORT: PortNum, PIN: PinNum, DIR> Pin<PORT, PIN, Alternate1<DIR>>
where
    Self: ToAlternate2,
{
    /// Convert pin to alternate function 2
    #[inline]
    pub fn to_alternate2(mut self) -> Pin<PORT, PIN, Alternate2<DIR>> {
        self.flip_selc();
        make_pin!()
    }
}

impl<PORT: PortNum, PIN: PinNum, DIR> Pin<PORT, PIN, Alternate1<DIR>>
where
    Self: ToAlternate3,
{
    /// Convert pin to alternate function 3
    #[inline]
    pub fn to_alternate3(mut self) -> Pin<PORT, PIN, Alternate3<DIR>> {
        self.set_sel1();
        make_pin!()
    }
}

// sel0 = 0, sel1 = 1
impl<PORT: PortNum, PIN: PinNum, DIR> Pin<PORT, PIN, Alternate2<DIR>> {
    /// Convert pin to GPIO function
    #[inline]
    pub fn to_gpio(mut self) -> Pin<PORT, PIN, DIR> {
        self.clear_sel1();
        make_pin!()
    }
}

impl<PORT: PortNum, PIN: PinNum, DIR> Pin<PORT, PIN, Alternate2<DIR>>
where
    Self: ToAlternate1,
{
    /// Convert pin to alternate function 1
    #[inline]
    pub fn to_alternate1(mut self) -> Pin<PORT, PIN, Alternate1<DIR>> {
        self.flip_selc();
        make_pin!()
    }
}

impl<PORT: PortNum, PIN: PinNum, DIR> Pin<PORT, PIN, Alternate2<DIR>>
where
    Self: ToAlternate3,
{
    /// Convert pin to alternate function 3
    #[inline]
    pub fn to_alternate3(mut self) -> Pin<PORT, PIN, Alternate3<DIR>> {
        self.set_sel0();
        make_pin!()
    }
}

// sel0 = 1, sel1 = 1
impl<PORT: PortNum, PIN: PinNum, DIR> Pin<PORT, PIN, Alternate3<DIR>> {
    /// Convert pin to GPIO function
    #[inline]
    pub fn to_gpio(mut self) -> Pin<PORT, PIN, DIR> {
        self.flip_selc();
        make_pin!()
    }
}

impl<PORT: PortNum, PIN: PinNum, DIR> Pin<PORT, PIN, Alternate3<DIR>>
where
    Self: ToAlternate1,
{
    /// Convert pin to alternate function 1
    #[inline]
    pub fn to_alternate1(mut self) -> Pin<PORT, PIN, Alternate1<DIR>> {
        self.clear_sel1();
        make_pin!()
    }
}

impl<PORT: PortNum, PIN: PinNum, DIR> Pin<PORT, PIN, Alternate3<DIR>>
where
    Self: ToAlternate2,
{
    /// Convert pin to alternate function 2
    #[inline]
    pub fn to_alternate2(mut self) -> Pin<PORT, PIN, Alternate2<DIR>> {
        self.clear_sel0();
        make_pin!()
    }
}

// P1 alternate 1
impl<PIN: PinNum, DIR> ToAlternate1 for Pin<P1, PIN, DIR> {}
// P1 alternate 2
impl<DIR> ToAlternate2 for Pin<P1, Pin0, DIR> {}
impl<DIR> ToAlternate2 for Pin<P1, Pin1, DIR> {}
impl<PULL> ToAlternate2 for Pin<P1, Pin2, Input<PULL>> {}
impl<DIR> ToAlternate2 for Pin<P1, Pin6, DIR> {}
impl<DIR> ToAlternate2 for Pin<P1, Pin7, DIR> {}
// P1 alternate 3
impl<PIN: PinNum, DIR> ToAlternate3 for Pin<P1, PIN, DIR> {}

// P2 alternate 1
impl<DIR> ToAlternate1 for Pin<P2, Pin0, DIR> {}
impl<DIR> ToAlternate1 for Pin<P2, Pin1, DIR> {}
impl<PULL> ToAlternate1 for Pin<P2, Pin2, Input<PULL>> {}
impl<DIR> ToAlternate1 for Pin<P2, Pin3, DIR> {}
impl<DIR> ToAlternate1 for Pin<P2, Pin6, DIR> {}
impl<DIR> ToAlternate1 for Pin<P2, Pin7, DIR> {}
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

mod ehal1 {
    use super::*;
    use core::convert::Infallible;
    use embedded_hal::digital::{ErrorType, InputPin, OutputPin, StatefulOutputPin};

    impl<PORT: PortNum, PIN: PinNum, DIR> ErrorType for Pin<PORT, PIN, DIR> {
        type Error = Infallible;
    }

    impl<PORT: PortNum, PIN: PinNum, PULL> InputPin for Pin<PORT, PIN, Input<PULL>> {
        #[inline]
        fn is_high(&mut self) -> Result<bool, Self::Error> {
            let p = unsafe { PORT::steal() };
            Ok(p.pxin_rd().check(PIN::NUM) != 0)
        }

        #[inline]
        fn is_low(&mut self) -> Result<bool, Self::Error> {
            self.is_high().map(|r| !r)
        }
    }

    impl<PORT: PortNum, PIN: PinNum> OutputPin for Pin<PORT, PIN, Output> {
        #[inline]
        fn set_low(&mut self) -> Result<(), Self::Error> {
            let p = unsafe { PORT::steal() };
            p.pxout_clear(PIN::CLR_MASK);
            Ok(())
        }

        #[inline]
        fn set_high(&mut self) -> Result<(), Self::Error> {
            let p = unsafe { PORT::steal() };
            p.pxout_set(PIN::SET_MASK);
            Ok(())
        }
    }

    impl<PORT: PortNum, PIN: PinNum> StatefulOutputPin for Pin<PORT, PIN, Output> {
        #[inline]
        fn is_set_high(&mut self) -> Result<bool, Self::Error> {
            let p = unsafe { PORT::steal() };
            Ok(p.pxout_rd().check(PIN::NUM) != 0)
        }

        #[inline]
        fn is_set_low(&mut self) -> Result<bool, Self::Error> {
            self.is_set_high().map(|r| !r)
        }
    }
}

#[cfg(feature = "embedded-hal-02")]
mod ehal02 {
    use super::*;
    use embedded_hal_02::digital::v2::{InputPin, OutputPin, StatefulOutputPin, ToggleableOutputPin};

    impl<PORT: PortNum, PIN: PinNum, PULL> InputPin for Pin<PORT, PIN, Input<PULL>> {
        type Error = void::Void;

        #[inline]
        fn is_high(&self) -> Result<bool, Self::Error> {
            let p = unsafe { PORT::steal() };
            Ok(p.pxin_rd().check(PIN::NUM) != 0)
        }

        #[inline]
        fn is_low(&self) -> Result<bool, Self::Error> {
            self.is_high().map(|r| !r)
        }
    }

    impl<PORT: PortNum, PIN: PinNum> OutputPin for Pin<PORT, PIN, Output> {
        type Error = void::Void;

        #[inline]
        fn set_low(&mut self) -> Result<(), Self::Error> {
            let p = unsafe { PORT::steal() };
            p.pxout_clear(PIN::CLR_MASK);
            Ok(())
        }

        #[inline]
        fn set_high(&mut self) -> Result<(), Self::Error> {
            let p = unsafe { PORT::steal() };
            p.pxout_set(PIN::SET_MASK);
            Ok(())
        }
    }

    impl<PORT: PortNum, PIN: PinNum> StatefulOutputPin for Pin<PORT, PIN, Output> {
        #[inline]
        fn is_set_high(&self) -> Result<bool, Self::Error> {
            let p = unsafe { PORT::steal() };
            Ok(p.pxout_rd().check(PIN::NUM) != 0)
        }

        #[inline]
        fn is_set_low(&self) -> Result<bool, Self::Error> {
            self.is_set_high().map(|r| !r)
        }
    }

    impl<PORT: PortNum, PIN: PinNum> ToggleableOutputPin for Pin<PORT, PIN, Output> {
        type Error = void::Void;

        #[inline]
        fn toggle(&mut self) -> Result<(), Self::Error> {
            let p = unsafe { PORT::steal() };
            p.pxout_toggle(PIN::SET_MASK);
            Ok(())
        }
    }
}
