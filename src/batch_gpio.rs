//! GPIO batch configuration.
//!
//! The `Batch` abstraction allows "collecting" changes to the configurations of different pins on
//! a GPIO port and writing the changes to the hardware all at once.
//! Changes to individual pins are performed **statically** using typestates, so the number of
//! register writes are minimized.
//!
//! For example, `P2.batch().config_pin3(|p| p.to_input_pullup()).config_pin1(|p| p.to_output()).split(&pmm)`
//! configures P2.3 as a pullup input pin and P2.1 as an output pin and then writes the
//! configuration to the hardware in a single set of writes.

use crate::gpio::*;
use crate::hw_traits::gpio::{GpioPeriph, IntrPeriph};
use crate::pmm::Pmm;
use crate::util::BitsExt;
use core::marker::PhantomData;

/// Proxy for a GPIO pin used for batch writes.
///
/// Configuring the proxy only changes the typestate of the proxy. Registers are only written once
/// all the proxies for the GPIO port are "committed".
pub struct PinProxy<PORT: PortNum, PIN: PinNum, DIR> {
    _port: PhantomData<PORT>,
    _pin: PhantomData<PIN>,
    _dir: PhantomData<DIR>,
}

macro_rules! make_proxy {
    () => {
        PinProxy {
            _port: PhantomData,
            _pin: PhantomData,
            _dir: PhantomData,
        }
    };
}

impl<PORT: PortNum, PIN: PinNum, PULL> PinProxy<PORT, PIN, Input<PULL>> {
    /// Configures pin as pulldown input
    #[inline(always)]
    pub fn pulldown(self) -> PinProxy<PORT, PIN, Input<Pulldown>> {
        make_proxy!()
    }

    /// Configures pin as pullup input
    #[inline(always)]
    pub fn pullup(self) -> PinProxy<PORT, PIN, Input<Pullup>> {
        make_proxy!()
    }

    /// Configures pin as floating input
    #[inline(always)]
    pub fn floating(self) -> PinProxy<PORT, PIN, Input<Floating>> {
        make_proxy!()
    }

    /// Configures pin as output
    #[inline(always)]
    pub fn to_output(self) -> PinProxy<PORT, PIN, Output> {
        make_proxy!()
    }
}

impl<PORT: PortNum, PIN: PinNum> PinProxy<PORT, PIN, Output> {
    /// Configures pin as floating input
    #[inline(always)]
    pub fn to_input_floating(self) -> PinProxy<PORT, PIN, Input<Floating>> {
        make_proxy!()
    }

    /// Configures pin as floating pullup
    #[inline(always)]
    pub fn to_input_pullup(self) -> PinProxy<PORT, PIN, Input<Pullup>> {
        make_proxy!()
    }

    /// Configures pin as floating pulldown
    #[inline(always)]
    pub fn to_input_pulldown(self) -> PinProxy<PORT, PIN, Input<Pulldown>> {
        make_proxy!()
    }
}

// Traits for deciding the value of a pin's registers
trait PxdirOn {}
trait PxoutOn {}
trait PxrenOn {}
trait Pxsel0On {}
trait Pxsel1On {}

trait WritePxdir {
    fn pxdir_on(&self) -> bool;
}
impl<T> WritePxdir for T {
    #[inline(always)]
    default fn pxdir_on(&self) -> bool {
        false
    }
}
impl<T: PxdirOn> WritePxdir for T {
    #[inline(always)]
    fn pxdir_on(&self) -> bool {
        true
    }
}

trait WritePxout {
    fn pxout_on(&self) -> bool;
}
impl<T> WritePxout for T {
    #[inline(always)]
    default fn pxout_on(&self) -> bool {
        false
    }
}
impl<T: PxoutOn> WritePxout for T {
    #[inline(always)]
    fn pxout_on(&self) -> bool {
        true
    }
}

trait WritePxren {
    fn pxren_on(&self) -> bool;
}
impl<T> WritePxren for T {
    #[inline(always)]
    default fn pxren_on(&self) -> bool {
        false
    }
}
impl<T: PxrenOn> WritePxren for T {
    #[inline(always)]
    fn pxren_on(&self) -> bool {
        true
    }
}

trait WritePxsel0 {
    fn pxsel0_on(&self) -> bool;
}
impl<T> WritePxsel0 for T {
    #[inline(always)]
    default fn pxsel0_on(&self) -> bool {
        false
    }
}
impl<T: Pxsel0On> WritePxsel0 for T {
    #[inline(always)]
    fn pxsel0_on(&self) -> bool {
        true
    }
}

trait WritePxsel1 {
    fn pxsel1_on(&self) -> bool;
}
impl<T> WritePxsel1 for T {
    #[inline(always)]
    default fn pxsel1_on(&self) -> bool {
        false
    }
}
impl<T: Pxsel1On> WritePxsel1 for T {
    #[inline(always)]
    fn pxsel1_on(&self) -> bool {
        true
    }
}

// Register marker trait implementations
impl<PORT: PortNum, PIN: PinNum> PxdirOn for PinProxy<PORT, PIN, Output> {}
impl<PORT: PortNum, PIN: PinNum> PxdirOn for PinProxy<PORT, PIN, Alternate1<Output>> {}
impl<PORT: PortNum, PIN: PinNum> PxdirOn for PinProxy<PORT, PIN, Alternate2<Output>> {}
impl<PORT: PortNum, PIN: PinNum> PxdirOn for PinProxy<PORT, PIN, Alternate3<Output>> {}

impl<PORT: PortNum, PIN: PinNum> PxrenOn for PinProxy<PORT, PIN, Input<Pullup>> {}
impl<PORT: PortNum, PIN: PinNum> PxrenOn for PinProxy<PORT, PIN, Input<Pulldown>> {}
impl<PORT: PortNum, PIN: PinNum> PxrenOn for PinProxy<PORT, PIN, Alternate1<Input<Pullup>>> {}
impl<PORT: PortNum, PIN: PinNum> PxrenOn for PinProxy<PORT, PIN, Alternate1<Input<Pulldown>>> {}
impl<PORT: PortNum, PIN: PinNum> PxrenOn for PinProxy<PORT, PIN, Alternate2<Input<Pullup>>> {}
impl<PORT: PortNum, PIN: PinNum> PxrenOn for PinProxy<PORT, PIN, Alternate2<Input<Pulldown>>> {}
impl<PORT: PortNum, PIN: PinNum> PxrenOn for PinProxy<PORT, PIN, Alternate3<Input<Pullup>>> {}
impl<PORT: PortNum, PIN: PinNum> PxrenOn for PinProxy<PORT, PIN, Alternate3<Input<Pulldown>>> {}

impl<PORT: PortNum, PIN: PinNum> PxoutOn for PinProxy<PORT, PIN, Input<Pullup>> {}
impl<PORT: PortNum, PIN: PinNum> PxoutOn for PinProxy<PORT, PIN, Alternate1<Input<Pullup>>> {}
impl<PORT: PortNum, PIN: PinNum> PxoutOn for PinProxy<PORT, PIN, Alternate2<Input<Pullup>>> {}
impl<PORT: PortNum, PIN: PinNum> PxoutOn for PinProxy<PORT, PIN, Alternate3<Input<Pullup>>> {}

impl<PORT: PortNum, PIN: PinNum, DIR> Pxsel0On for PinProxy<PORT, PIN, Alternate1<DIR>> {}
impl<PORT: PortNum, PIN: PinNum, DIR> Pxsel0On for PinProxy<PORT, PIN, Alternate3<DIR>> {}

impl<PORT: PortNum, PIN: PinNum, DIR> Pxsel1On for PinProxy<PORT, PIN, Alternate2<DIR>> {}
impl<PORT: PortNum, PIN: PinNum, DIR> Pxsel1On for PinProxy<PORT, PIN, Alternate3<DIR>> {}

// Derive bitmasks for different GPIO registers from pin numbers and register trait implementations
trait MaskRegisters {
    fn pxout_mask(&self) -> u8;
    fn pxdir_mask(&self) -> u8;
    fn pxren_mask(&self) -> u8;
    fn pxsel0_mask(&self) -> u8;
    fn pxsel1_mask(&self) -> u8;
}

impl<PORT: PortNum, PIN: PinNum, DIR> MaskRegisters for PinProxy<PORT, PIN, DIR> {
    #[inline(always)]
    fn pxout_mask(&self) -> u8 {
        (self.pxout_on() as u8) << PIN::NUM
    }

    #[inline(always)]
    fn pxdir_mask(&self) -> u8 {
        (self.pxdir_on() as u8) << PIN::NUM
    }

    #[inline(always)]
    fn pxren_mask(&self) -> u8 {
        (self.pxren_on() as u8) << PIN::NUM
    }

    #[inline(always)]
    fn pxsel0_mask(&self) -> u8 {
        (self.pxsel0_on() as u8) << PIN::NUM
    }

    #[inline(always)]
    fn pxsel1_mask(&self) -> u8 {
        (self.pxsel1_on() as u8) << PIN::NUM
    }
}

trait InterruptOperations {
    fn maybe_set_pxie(&self, b: u8);
}

impl<P: GpioPeriph> InterruptOperations for P {
    #[inline(always)]
    default fn maybe_set_pxie(&self, _b: u8) {}
}

impl<P: IntrPeriph> InterruptOperations for P {
    #[inline(always)]
    fn maybe_set_pxie(&self, b: u8) {
        self.pxie_set(b);
    }
}

impl<P: PortNum>
    Batch<
        P,
        Input<Floating>,
        Input<Floating>,
        Input<Floating>,
        Input<Floating>,
        Input<Floating>,
        Input<Floating>,
        Input<Floating>,
        Input<Floating>,
    >
{
    /// Split into a batch of individual GPIO pin proxies
    pub fn new(_port: P) -> Self {
        Self::create()
    }
}

/// Collection of proxies for pins 0 to 7 of a specific port, used to commit configurations for
/// all pins in a single step.
pub struct Batch<PORT: PortNum, DIR0, DIR1, DIR2, DIR3, DIR4, DIR5, DIR6, DIR7> {
    pin0: PinProxy<PORT, Pin0, DIR0>,
    pin1: PinProxy<PORT, Pin1, DIR1>,
    pin2: PinProxy<PORT, Pin2, DIR2>,
    pin3: PinProxy<PORT, Pin3, DIR3>,
    pin4: PinProxy<PORT, Pin4, DIR4>,
    pin5: PinProxy<PORT, Pin5, DIR5>,
    pin6: PinProxy<PORT, Pin6, DIR6>,
    pin7: PinProxy<PORT, Pin7, DIR7>,
}

impl<PORT: PortNum, DIR0, DIR1, DIR2, DIR3, DIR4, DIR5, DIR6, DIR7>
    Batch<PORT, DIR0, DIR1, DIR2, DIR3, DIR4, DIR5, DIR6, DIR7>
{
    #[inline]
    fn write_regs(&self) {
        let pxdir = 0u8
            .set_mask(self.pin0.pxdir_mask())
            .set_mask(self.pin1.pxdir_mask())
            .set_mask(self.pin2.pxdir_mask())
            .set_mask(self.pin3.pxdir_mask())
            .set_mask(self.pin4.pxdir_mask())
            .set_mask(self.pin5.pxdir_mask())
            .set_mask(self.pin6.pxdir_mask())
            .set_mask(self.pin7.pxdir_mask());

        let pxout = 0u8
            .set_mask(self.pin0.pxout_mask())
            .set_mask(self.pin1.pxout_mask())
            .set_mask(self.pin2.pxout_mask())
            .set_mask(self.pin3.pxout_mask())
            .set_mask(self.pin4.pxout_mask())
            .set_mask(self.pin5.pxout_mask())
            .set_mask(self.pin6.pxout_mask())
            .set_mask(self.pin7.pxout_mask());

        let pxren = 0u8
            .set_mask(self.pin0.pxren_mask())
            .set_mask(self.pin1.pxren_mask())
            .set_mask(self.pin2.pxren_mask())
            .set_mask(self.pin3.pxren_mask())
            .set_mask(self.pin4.pxren_mask())
            .set_mask(self.pin5.pxren_mask())
            .set_mask(self.pin6.pxren_mask())
            .set_mask(self.pin7.pxren_mask());

        let pxsel0 = 0u8
            .set_mask(self.pin0.pxsel0_mask())
            .set_mask(self.pin1.pxsel0_mask())
            .set_mask(self.pin2.pxsel0_mask())
            .set_mask(self.pin3.pxsel0_mask())
            .set_mask(self.pin4.pxsel0_mask())
            .set_mask(self.pin5.pxsel0_mask())
            .set_mask(self.pin6.pxsel0_mask())
            .set_mask(self.pin7.pxsel0_mask());

        let pxsel1 = 0u8
            .set_mask(self.pin0.pxsel1_mask())
            .set_mask(self.pin1.pxsel1_mask())
            .set_mask(self.pin2.pxsel1_mask())
            .set_mask(self.pin3.pxsel1_mask())
            .set_mask(self.pin4.pxsel1_mask())
            .set_mask(self.pin5.pxsel1_mask())
            .set_mask(self.pin6.pxsel1_mask())
            .set_mask(self.pin7.pxsel1_mask());

        let p = unsafe { PORT::steal() };
        // Turn off interrupts first so nothing fires during subsequent register writes
        p.maybe_set_pxie(0);
        p.pxsel0_wr(pxsel0);
        p.pxsel1_wr(pxsel1);
        p.pxout_wr(pxout);
        p.pxdir_wr(pxdir);
        p.pxren_wr(pxren);
    }

    #[inline(always)]
    pub(super) fn create() -> Self {
        Self {
            pin0: make_proxy!(),
            pin1: make_proxy!(),
            pin2: make_proxy!(),
            pin3: make_proxy!(),
            pin4: make_proxy!(),
            pin5: make_proxy!(),
            pin6: make_proxy!(),
            pin7: make_proxy!(),
        }
    }

    /// Commits all pin configurations to GPIO registers and returns GPIO parts and turns off all
    /// interrupt enable bits.
    ///
    /// Note that the pin's interrupt flags may become set as a result of
    /// this operation.
    ///
    /// GPIO input/output operations only work after the LOCKLPM5 bit has been set, which is
    /// ensured when passing `&Pmm` into the method, since a `Pmm` is created only by setting
    /// LOCKLPM5.
    #[inline]
    pub fn split(self, _pmm: &Pmm) -> Parts<PORT, DIR0, DIR1, DIR2, DIR3, DIR4, DIR5, DIR6, DIR7> {
        self.write_regs();
        Parts::new()
    }

    /// Edit configuration of pin 0
    #[inline(always)]
    pub fn config_pin0<NEW, F: FnOnce(PinProxy<PORT, Pin0, DIR0>) -> PinProxy<PORT, Pin0, NEW>>(
        self,
        f: F,
    ) -> Batch<PORT, NEW, DIR1, DIR2, DIR3, DIR4, DIR5, DIR6, DIR7> {
        Batch {
            pin0: f(self.pin0),
            pin1: make_proxy!(),
            pin2: make_proxy!(),
            pin3: make_proxy!(),
            pin4: make_proxy!(),
            pin5: make_proxy!(),
            pin6: make_proxy!(),
            pin7: make_proxy!(),
        }
    }

    /// Edit configuration of pin 1
    #[inline(always)]
    pub fn config_pin1<NEW, F: FnOnce(PinProxy<PORT, Pin1, DIR1>) -> PinProxy<PORT, Pin1, NEW>>(
        self,
        f: F,
    ) -> Batch<PORT, DIR0, NEW, DIR2, DIR3, DIR4, DIR5, DIR6, DIR7> {
        Batch {
            pin0: make_proxy!(),
            pin1: f(self.pin1),
            pin2: make_proxy!(),
            pin3: make_proxy!(),
            pin4: make_proxy!(),
            pin5: make_proxy!(),
            pin6: make_proxy!(),
            pin7: make_proxy!(),
        }
    }

    /// Edit configuration of pin 2
    #[inline(always)]
    pub fn config_pin2<NEW, F: FnOnce(PinProxy<PORT, Pin2, DIR2>) -> PinProxy<PORT, Pin2, NEW>>(
        self,
        f: F,
    ) -> Batch<PORT, DIR0, DIR1, NEW, DIR3, DIR4, DIR5, DIR6, DIR7> {
        Batch {
            pin0: make_proxy!(),
            pin1: make_proxy!(),
            pin2: f(self.pin2),
            pin3: make_proxy!(),
            pin4: make_proxy!(),
            pin5: make_proxy!(),
            pin6: make_proxy!(),
            pin7: make_proxy!(),
        }
    }

    /// Edit configuration of pin 3
    #[inline(always)]
    pub fn config_pin3<NEW, F: FnOnce(PinProxy<PORT, Pin3, DIR3>) -> PinProxy<PORT, Pin3, NEW>>(
        self,
        f: F,
    ) -> Batch<PORT, DIR0, DIR1, DIR2, NEW, DIR4, DIR5, DIR6, DIR7> {
        Batch {
            pin0: make_proxy!(),
            pin1: make_proxy!(),
            pin2: make_proxy!(),
            pin3: f(self.pin3),
            pin4: make_proxy!(),
            pin5: make_proxy!(),
            pin6: make_proxy!(),
            pin7: make_proxy!(),
        }
    }

    /// Edit configuration of pin 4
    #[inline(always)]
    pub fn config_pin4<NEW, F: FnOnce(PinProxy<PORT, Pin4, DIR4>) -> PinProxy<PORT, Pin4, NEW>>(
        self,
        f: F,
    ) -> Batch<PORT, DIR0, DIR1, DIR2, DIR3, NEW, DIR5, DIR6, DIR7> {
        Batch {
            pin0: make_proxy!(),
            pin1: make_proxy!(),
            pin2: make_proxy!(),
            pin3: make_proxy!(),
            pin4: f(self.pin4),
            pin5: make_proxy!(),
            pin6: make_proxy!(),
            pin7: make_proxy!(),
        }
    }

    /// Edit configuration of pin 5
    #[inline(always)]
    pub fn config_pin5<NEW, F: FnOnce(PinProxy<PORT, Pin5, DIR5>) -> PinProxy<PORT, Pin5, NEW>>(
        self,
        f: F,
    ) -> Batch<PORT, DIR0, DIR1, DIR2, DIR3, DIR4, NEW, DIR6, DIR7> {
        Batch {
            pin0: make_proxy!(),
            pin1: make_proxy!(),
            pin2: make_proxy!(),
            pin3: make_proxy!(),
            pin4: make_proxy!(),
            pin5: f(self.pin5),
            pin6: make_proxy!(),
            pin7: make_proxy!(),
        }
    }

    /// Edit configuration of pin 6
    #[inline(always)]
    pub fn config_pin6<NEW, F: FnOnce(PinProxy<PORT, Pin6, DIR6>) -> PinProxy<PORT, Pin6, NEW>>(
        self,
        f: F,
    ) -> Batch<PORT, DIR0, DIR1, DIR2, DIR3, DIR4, DIR5, NEW, DIR7> {
        Batch {
            pin0: make_proxy!(),
            pin1: make_proxy!(),
            pin2: make_proxy!(),
            pin3: make_proxy!(),
            pin4: make_proxy!(),
            pin5: make_proxy!(),
            pin6: f(self.pin6),
            pin7: make_proxy!(),
        }
    }

    /// Edit configuration of pin 7
    #[inline(always)]
    pub fn config_pin7<NEW, F: FnOnce(PinProxy<PORT, Pin7, DIR7>) -> PinProxy<PORT, Pin7, NEW>>(
        self,
        f: F,
    ) -> Batch<PORT, DIR0, DIR1, DIR2, DIR3, DIR4, DIR5, DIR6, NEW> {
        Batch {
            pin0: make_proxy!(),
            pin1: make_proxy!(),
            pin2: make_proxy!(),
            pin3: make_proxy!(),
            pin4: make_proxy!(),
            pin5: make_proxy!(),
            pin6: make_proxy!(),
            pin7: f(self.pin7),
        }
    }
}
