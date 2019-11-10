use crate::bits::BitsExt;
use crate::gpio::*;
use crate::hw_traits::gpio::{GpioPeriph, IntrPeriph};
use crate::pmm::Pmm;
use core::marker::PhantomData;

/// Proxy for a GPIO pin used for batch writes.
/// Configuring the proxy only changes the typestate of the proxy. Registers are only written once
/// the proxies for the GPIO port are "committed".
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
    pub fn pulldown(self) -> PinProxy<PORT, PIN, Input<Pulldown>> {
        make_proxy!()
    }

    /// Configures pin as pullup input
    pub fn pullup(self) -> PinProxy<PORT, PIN, Input<Pullup>> {
        make_proxy!()
    }

    /// Configures pin as floating input
    pub fn floating(self) -> PinProxy<PORT, PIN, Input<Floating>> {
        make_proxy!()
    }
}

impl<PORT: PortNum, PIN: PinNum, DIR> PinProxy<PORT, PIN, DIR> {
    /// Configures pin as output
    pub fn to_output(self) -> PinProxy<PORT, PIN, Output> {
        make_proxy!()
    }

    /// Configures pin as output
    pub fn to_input(self) -> PinProxy<PORT, PIN, Input<Floating>> {
        make_proxy!()
    }
}

// Traits for deciding the value of a pin's registers
trait WritePxdir {
    fn pxdir_on(&self) -> bool;
}

impl<T> WritePxdir for T {
    default fn pxdir_on(&self) -> bool {
        false
    }
}

trait WritePxout {
    fn pxout_on(&self) -> bool;
}

impl<PORT: PortNum, PIN: PinNum, DIR> WritePxout for PinProxy<PORT, PIN, DIR> {
    default fn pxout_on(&self) -> bool {
        false
    }
}

trait WritePxren {
    fn pxren_on(&self) -> bool;
}

impl<PORT: PortNum, PIN: PinNum, DIR> WritePxren for PinProxy<PORT, PIN, DIR> {
    default fn pxren_on(&self) -> bool {
        false
    }
}

// Register value trait implementations
impl<PORT: PortNum, PIN: PinNum> WritePxdir for PinProxy<PORT, PIN, Output> {
    fn pxdir_on(&self) -> bool {
        true
    }
}

impl<PORT: PortNum, PIN: PinNum> WritePxren for PinProxy<PORT, PIN, Input<Pullup>> {
    fn pxren_on(&self) -> bool {
        true
    }
}
impl<PORT: PortNum, PIN: PinNum> WritePxren for PinProxy<PORT, PIN, Input<Pulldown>> {
    fn pxren_on(&self) -> bool {
        true
    }
}
impl<PORT: PortNum, PIN: PinNum> WritePxout for PinProxy<PORT, PIN, Input<Pullup>> {
    fn pxout_on(&self) -> bool {
        true
    }
}

trait MaskRegisters {
    fn pxout_mask(&self) -> u8;
    fn pxdir_mask(&self) -> u8;
    fn pxren_mask(&self) -> u8;
}

impl<PORT: PortNum, PIN: PinNum, DIR> MaskRegisters for PinProxy<PORT, PIN, DIR> {
    fn pxout_mask(&self) -> u8 {
        (self.pxout_on() as u8) << PIN::pin()
    }

    fn pxdir_mask(&self) -> u8 {
        (self.pxdir_on() as u8) << PIN::pin()
    }

    fn pxren_mask(&self) -> u8 {
        (self.pxren_on() as u8) << PIN::pin()
    }
}

trait MaybeClearInterrupts {
    fn maybe_clear(&self);
}

impl<P: GpioPeriph> MaybeClearInterrupts for P {
    default fn maybe_clear(&self) {}
}

impl<P: IntrPeriph> MaybeClearInterrupts for P {
    fn maybe_clear(&self) {
        self.pxifg_wr(0x00);
    }
}

/// Extension trait to split GPIO peripheral
pub trait GpioExt {
    /// The parts to split the GPIO into.
    type Batch;

    /// Split the GPIO into pins and contention tokens
    fn batch(self) -> Self::Batch;
}

impl<P: GpioPort> GpioExt for P {
    type Batch = Batch<
        P::PortNum,
        Input<Floating>,
        Input<Floating>,
        Input<Floating>,
        Input<Floating>,
        Input<Floating>,
        Input<Floating>,
        Input<Floating>,
        Input<Floating>,
    >;

    fn batch(self) -> Self::Batch {
        Self::Batch::new()
    }
}

/// Collection of proxies for pins 0 to 7 of a specific port, used to commit configurations for
/// all pins in a single step, reducing the total number of register accesses.
pub struct Batch<PORT: PortNum, DIR0, DIR1, DIR2, DIR3, DIR4, DIR5, DIR6, DIR7> {
    /// Pin 0 proxy
    pub pin0: PinProxy<PORT, Pin0, DIR0>,
    /// Pin 1 proxy
    pub pin1: PinProxy<PORT, Pin1, DIR1>,
    /// Pin 2 proxy
    pub pin2: PinProxy<PORT, Pin2, DIR2>,
    /// Pin 3 proxy
    pub pin3: PinProxy<PORT, Pin3, DIR3>,
    /// Pin 4 proxy
    pub pin4: PinProxy<PORT, Pin4, DIR4>,
    /// Pin 5 proxy
    pub pin5: PinProxy<PORT, Pin5, DIR5>,
    /// Pin 6 proxy
    pub pin6: PinProxy<PORT, Pin6, DIR6>,
    /// Pin 7 proxy
    pub pin7: PinProxy<PORT, Pin7, DIR7>,
}

impl<PORT: PortNum, DIR0, DIR1, DIR2, DIR3, DIR4, DIR5, DIR6, DIR7>
    Batch<PORT, DIR0, DIR1, DIR2, DIR3, DIR4, DIR5, DIR6, DIR7>
{
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

        let p = PORT::Port::steal();
        p.pxout_wr(pxout);
        p.pxdir_wr(pxdir);
        p.pxren_wr(pxren);
        // Clear interrupts
        p.maybe_clear();
    }

    pub(super) fn new() -> Self {
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

    /// Commits all pin configurations to GPIO registers and returns GPIO parts.
    /// GPIO input/output operations only work after the LOCKLPM5 bit has been set, which is
    /// ensured when passing `&Pmm` into the method, since a `Pmm` is created only be setting
    /// LOCKLPM5.
    pub fn split(self, _pmm: &Pmm) -> Parts<PORT, DIR0, DIR1, DIR2, DIR3, DIR4, DIR5, DIR6, DIR7> {
        self.write_regs();
        Parts::new()
    }

    /// Edit configuration of pin 0
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
