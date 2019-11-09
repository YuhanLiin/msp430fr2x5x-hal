use crate::bits::BitsExt;
use crate::gpio::*;
use crate::hw_traits::gpio::{GpioPeriph, IntrPeriph};
use crate::pmm::Pmm;
use core::marker::PhantomData;
use msp430fr2355 as pac;
use pac::{p1, p2, p3, p4, p5, p6, P1, P2, P3, P4, P5, P6};

/// Proxy for a GPIO pin used for batch writes
pub struct PinProxy<PIN: PortPinNum, DIR> {
    _pin: PhantomData<PIN>,
    _dir: PhantomData<DIR>,
}

macro_rules! make_proxy {
    () => {
        PinProxy {
            _pin: PhantomData,
            _dir: PhantomData,
        }
    };
}

impl<PIN: PortPinNum, PULL> PinProxy<PIN, Input<PULL>> {
    /// Configures pin as pulldown input
    pub fn pulldown(self) -> PinProxy<PIN, Input<Pulldown>> {
        make_proxy!()
    }

    /// Configures pin as pullup input
    pub fn pullup(self) -> PinProxy<PIN, Input<Pullup>> {
        make_proxy!()
    }

    /// Configures pin as floating input
    pub fn floating(self) -> PinProxy<PIN, Input<Floating>> {
        make_proxy!()
    }
}

impl<PIN: PortPinNum, DIR: ConvertToOutput> PinProxy<PIN, DIR> {
    /// Configures pin as output
    pub fn to_output(self, _pxdir: &mut Pxdir<PIN::Periph>) -> PinProxy<PIN, Output> {
        make_proxy!()
    }
}

impl<PIN: PortPinNum, DIR: ConvertToInput> PinProxy<PIN, DIR> {
    /// Configures pin as output
    pub fn to_input(self, _pxdir: &mut Pxdir<PIN::Periph>) -> PinProxy<PIN, Input<Unknown>> {
        make_proxy!()
    }
}

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

impl<T> WritePxout for T {
    default fn pxout_on(&self) -> bool {
        false
    }
}

trait WritePxren {
    fn pxren_on(&self) -> bool;
}

impl<T> WritePxren for T {
    default fn pxren_on(&self) -> bool {
        false
    }
}

trait MaskRegister: WritePxdir + WritePxren + WritePxout {
    fn pxren_mask(&self) -> u8;
    fn pxout_mask(&self) -> u8;
    fn pxdir_mask(&self) -> u8;
}

impl<PIN: PortPinNum, DIR> MaskRegister for PinProxy<PIN, DIR> {
    fn pxdir_mask(&self) -> u8 {
        (self.pxdir_on() as u8) << PIN::pin()
    }

    fn pxout_mask(&self) -> u8 {
        (self.pxout_on() as u8) << PIN::pin()
    }

    fn pxren_mask(&self) -> u8 {
        (self.pxren_on() as u8) << PIN::pin()
    }
}

impl<PIN: PortPinNum> WritePxdir for PinProxy<PIN, Output> {
    fn pxdir_on(&self) -> bool {
        true
    }
}

impl<PIN: PortPinNum> WritePxout for PinProxy<PIN, Input<Pullup>> {
    fn pxout_on(&self) -> bool {
        true
    }
}

impl<PIN: PortPinNum> WritePxren for PinProxy<PIN, Input<Pullup>> {
    fn pxren_on(&self) -> bool {
        true
    }
}

impl<PIN: PortPinNum> WritePxren for PinProxy<PIN, Input<Pulldown>> {
    fn pxren_on(&self) -> bool {
        true
    }
}

/// Extension trait to split GPIO peripheral
pub trait GpioExt {
    /// The parts to split the GPIO into.
    type Parts;

    /// Split the GPIO into pins and contention tokens
    fn batch(self) -> Self::Parts;
}

macro_rules! impl_batch {
    ($Px:ident, $px:ident, $PxBatch:ident, $PxParts:ident, $Portx:ident $(, [$pin5:ident: $dir5:ident=$U5:ident, $pin6:ident: $dir6:ident=$U6:ident $(, $pin7:ident: $dir7:ident=$U7:ident)?])?) => {
        pub struct $PxBatch<DIR0, DIR1, DIR2, DIR3, DIR4 $(, $dir5, $dir6, $($dir7)?)?> {
            pub pin0: PinProxy<$Portx<Pin0>, DIR0>,
            pub pin1: PinProxy<$Portx<Pin1>, DIR1>,
            pub pin2: PinProxy<$Portx<Pin2>, DIR2>,
            pub pin3: PinProxy<$Portx<Pin3>, DIR3>,
            pub pin4: PinProxy<$Portx<Pin4>, DIR4>,
            $(
                /// Pin5
                pub pin5: PinProxy<$Portx<Pin5>, $dir5>,
                /// Pin6
                pub pin6: PinProxy<$Portx<Pin6>, $dir6>,
                $(
                    /// Pin7
                    pub pin7: PinProxy<$Portx<Pin7>, $dir7>,
                )?
            )?
        }

        impl<DIR0, DIR1, DIR2, DIR3, DIR4 $(, $dir5, $dir6, $($dir7)?)?> $PxBatch<DIR0, DIR1, DIR2, DIR3, DIR4 $(, $dir5, $dir6, $($dir7)?)?> {
            pub fn new() -> Self {
                $PxBatch {
                    pin0: make_proxy!(),
                    pin1: make_proxy!(),
                    pin2: make_proxy!(),
                    pin3: make_proxy!(),
                    pin4: make_proxy!(),
                    $(
                        $pin5: make_proxy!(),
                        $pin6: make_proxy!(),
                        $(
                            $pin7: make_proxy!(),
                        )?
                    )?
                }
            }

            fn write_regs(&self) {
                let mut pxdir = 0u8.set_mask(self.pin0.pxdir_mask())
                                   .set_mask(self.pin1.pxdir_mask())
                                   .set_mask(self.pin2.pxdir_mask())
                                   .set_mask(self.pin3.pxdir_mask())
                                   .set_mask(self.pin4.pxdir_mask());

                let mut pxout = 0u8.set_mask(self.pin0.pxout_mask())
                                   .set_mask(self.pin1.pxout_mask())
                                   .set_mask(self.pin2.pxout_mask())
                                   .set_mask(self.pin3.pxout_mask())
                                   .set_mask(self.pin4.pxout_mask());

                let mut pxren = 0u8.set_mask(self.pin0.pxren_mask())
                                   .set_mask(self.pin1.pxren_mask())
                                   .set_mask(self.pin2.pxren_mask())
                                   .set_mask(self.pin3.pxren_mask())
                                   .set_mask(self.pin4.pxren_mask());

                $(
                    pxdir = pxdir.set_mask(self.$pin5.pxdir_mask()).set_mask(self.$pin6.pxdir_mask());
                    pxout = pxout.set_mask(self.$pin5.pxout_mask()).set_mask(self.$pin6.pxout_mask());
                    pxren = pxren.set_mask(self.$pin5.pxren_mask()).set_mask(self.$pin6.pxren_mask());
                    $(
                        pxdir = pxdir.set_mask(self.$pin7.pxdir_mask());
                        pxout = pxout.set_mask(self.$pin7.pxout_mask());
                        pxren = pxren.set_mask(self.$pin7.pxren_mask());
                    )?
                )?

                let p = pac::$px::RegisterBlock::steal();
                p.pxout_wr(pxout);
                p.pxdir_wr(pxdir);
                p.pxren_wr(pxren);
                p.pxifg_wr(0x00);
            }

            pub fn configure(self) -> $PxParts<DIR0, DIR1, DIR2, DIR3, DIR4 $(, $dir5, $dir6 $(, $dir7)?)?> {
                $PxParts::new()
            }
        }

        impl GpioExt for $Px {
            type Parts = $PxBatch<Unknown, Unknown, Unknown, Unknown, Unknown $(, $U5, $U6 $(, $U7)?)?>;

            fn batch(self) -> Self::Parts {
                $PxBatch::new()
            }
        }

        impl<DIR0, DIR1, DIR2, DIR3, DIR4 $(, $dir5, $dir6 $(, $dir7)?)?> $PxParts<DIR0, DIR1, DIR2, DIR3, DIR4 $(, $dir5, $dir6 $(, $dir7)?)?> {
            pub fn batch(self) -> $PxBatch<DIR0, DIR1, DIR2, DIR3, DIR4 $(, $dir5, $dir6 $(, $dir7)?)?> {
                $PxBatch::new()
            }
        }
    };
}

impl_batch!(
    P1,
    p1,
    P1Batch,
    P1Parts,
    Port1,
    [
        pin5: DIR5 = Unknown,
        pin6: DIR6 = Unknown,
        pin7: DIR7 = Unknown
    ]
);
