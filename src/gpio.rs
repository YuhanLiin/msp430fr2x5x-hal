use crate::hw_traits::gpio::GpioPeriph;
use core::marker::PhantomData;
use msp430fr2355 as pac;
use pac::generic::{Readable, Reg, Writable};

pub trait GpioPin {
    type Periph: GpioPeriph;

    fn pin() -> u8;

    fn pin_mask() -> u8 {
        1 << Self::pin()
    }
}

trait PinNumber {
    fn num() -> u8;
}
trait UnderSeven: PinNumber {}
trait UnderFive: PinNumber {}

struct Pin0;
impl PinNumber for Pin0 {
    fn num() -> u8 {
        0
    }
}
impl UnderSeven for Pin0 {}
impl UnderFive for Pin0 {}

struct Pin1;
impl PinNumber for Pin1 {
    fn num() -> u8 {
        1
    }
}
impl UnderSeven for Pin1 {}
impl UnderFive for Pin1 {}

struct Pin2;
impl PinNumber for Pin2 {
    fn num() -> u8 {
        2
    }
}
impl UnderSeven for Pin2 {}
impl UnderFive for Pin2 {}

struct Pin3;
impl PinNumber for Pin3 {
    fn num() -> u8 {
        3
    }
}
impl UnderSeven for Pin3 {}
impl UnderFive for Pin3 {}

struct Pin4;
impl PinNumber for Pin4 {
    fn num() -> u8 {
        4
    }
}
impl UnderSeven for Pin4 {}
impl UnderFive for Pin4 {}

struct Pin5;
impl PinNumber for Pin5 {
    fn num() -> u8 {
        5
    }
}
impl UnderSeven for Pin5 {}

struct Pin6;
impl PinNumber for Pin6 {
    fn num() -> u8 {
        6
    }
}
impl UnderSeven for Pin6 {}

struct Pin7;
impl PinNumber for Pin7 {
    fn num() -> u8 {
        7
    }
}

struct Port1<P>(PhantomData<P>);
impl<P: PinNumber> GpioPin for Port1<P> {
    type Periph = pac::p1::RegisterBlock;

    fn pin() -> u8 {
        P::num()
    }
}

struct Port2<P>(PhantomData<P>);
impl<P: PinNumber> GpioPin for Port2<P> {
    type Periph = pac::p2::RegisterBlock;

    fn pin() -> u8 {
        P::num()
    }
}

struct Port3<P>(PhantomData<P>);
impl<P: PinNumber> GpioPin for Port3<P> {
    type Periph = pac::p3::RegisterBlock;

    fn pin() -> u8 {
        P::num()
    }
}

struct Port4<P>(PhantomData<P>);
impl<P: PinNumber> GpioPin for Port4<P> {
    type Periph = pac::p4::RegisterBlock;

    fn pin() -> u8 {
        P::num()
    }
}

struct Port5<P>(PhantomData<P>);
impl<P: UnderFive> GpioPin for Port5<P> {
    type Periph = pac::p5::RegisterBlock;

    fn pin() -> u8 {
        P::num()
    }
}

struct Port6<P>(PhantomData<P>);
impl<P: UnderSeven> GpioPin for Port6<P> {
    type Periph = pac::p6::RegisterBlock;

    fn pin() -> u8 {
        P::num()
    }
}

// Pin direction
pub struct Output;
pub struct Input<PULL>(PhantomData<PULL>);
pub struct Unknown;

// Pin input pull state
pub struct Pullup;
pub struct Pulldown;
pub struct Floating;

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

impl<PIN: GpioPin, PULL, LOCK> Pin<PIN, Input<PULL>, LOCK> {
    pub fn pulldown(self) -> Pin<PIN, Input<Pulldown>, LOCK> {
        let p = PIN::Periph::steal();
        p.pxout_wr(p.pxout_rd() & !PIN::pin_mask());
        make_pin!()
    }
}
