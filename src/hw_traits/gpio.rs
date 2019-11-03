use msp430fr2355 as pac;

macro_rules! modify {
    ($rd:ident, $wr:ident, $mod:ident, $num:ty) => {
        fn $mod<F: Fn($num) -> $num>(&self, f: F) -> $num {
            let n = f(self.$rd());
            self.$wr(n);
            n
        }
    };

    ($rd:ident, $wr:ident, $mod:ident) => {
        modify!($rd, $wr, $mod, u8);
    }
}

pub trait GpioPeriph {
    fn steal<'a>() -> &'a Self;

    fn pxin_rd(&self) -> u8;

    fn pxout_rd(&self) -> u8;
    fn pxout_wr(&self, bits: u8);
    modify!(pxout_rd, pxout_wr, pxout_mod);

    fn pxdir_rd(&self) -> u8;
    fn pxdir_wr(&self, bits: u8);
    modify!(pxdir_rd, pxdir_wr, pxdir_mod);

    fn pxren_rd(&self) -> u8;
    fn pxren_wr(&self, bits: u8);
    modify!(pxren_rd, pxren_wr, pxren_mod);

    fn pxselc_rd(&self) -> u8;
    fn pxselc_wr(&self, bits: u8);
    modify!(pxselc_rd, pxselc_wr, pxselc_mod);

    fn pxsel0_rd(&self) -> u8;
    fn pxsel0_wr(&self, bits: u8);
    modify!(pxsel0_rd, pxsel0_wr, pxsel0_mod);

    fn pxsel1_rd(&self) -> u8;
    fn pxsel1_wr(&self, bits: u8);
    modify!(pxsel1_rd, pxsel1_wr, pxsel1_mod);
}

pub trait IntrPeriph: GpioPeriph {
    fn pxies_rd(&self) -> u8;
    fn pxies_wr(&self, bits: u8);
    modify!(pxies_rd, pxies_wr, pxies_mod);

    fn pxie_rd(&self) -> u8;
    fn pxie_wr(&self, bits: u8);
    modify!(pxie_rd, pxie_wr, pxie_mod);

    fn pxifg_rd(&self) -> u8;
    fn pxifg_wr(&self, bits: u8);
    modify!(pxifg_rd, pxifg_wr, pxifg_mod);

    fn pxiv_rd(&self) -> u16;
}

macro_rules! gpio_impl {
    ($px:ident: $Px:ident =>
     $pxin:ident, $pxout:ident, $pxdir:ident, $pxren:ident, $pxselc:ident, $pxsel0:ident, $pxsel1:ident
     $(, [$pxies:ident, $pxie:ident, $pxifg:ident, $pxiv:ident])?
    ) => {
        mod $px {
            use super::*;

            impl GpioPeriph for pac::$px::RegisterBlock {
                fn steal<'a>() -> &'a Self {
                    unsafe{ &*pac::$Px::ptr() }
                }

                fn pxin_rd(&self) -> u8 {
                    self.$pxin.read().bits()
                }

                fn pxout_rd(&self) -> u8 {
                    self.$pxout.read().bits()
                }
                fn pxout_wr(&self, bits: u8) {
                    self.$pxout.write(|w| unsafe { w.bits(bits) })
                }

                fn pxdir_rd(&self) -> u8 {
                    self.$pxdir.read().bits()
                }
                fn pxdir_wr(&self, bits: u8) {
                    self.$pxdir.write(|w| unsafe { w.bits(bits) })
                }

                fn pxren_rd(&self) -> u8 {
                    self.$pxren.read().bits()
                }
                fn pxren_wr(&self, bits: u8) {
                    self.$pxren.write(|w| unsafe { w.bits(bits) })
                }

                fn pxselc_rd(&self) -> u8 {
                    self.$pxselc.read().bits()
                }
                fn pxselc_wr(&self, bits: u8) {
                    self.$pxselc.write(|w| unsafe { w.bits(bits) })
                }

                fn pxsel0_rd(&self) -> u8 {
                    self.$pxsel0.read().bits()
                }
                fn pxsel0_wr(&self, bits: u8) {
                    self.$pxsel0.write(|w| unsafe { w.bits(bits) })
                }

                fn pxsel1_rd(&self) -> u8 {
                    self.$pxsel1.read().bits()
                }
                fn pxsel1_wr(&self, bits: u8) {
                    self.$pxsel1.write(|w| unsafe { w.bits(bits) })
                }
            }

            $(
                impl IntrPeriph for pac::$px::RegisterBlock {
                    fn pxies_rd(&self) -> u8 {
                        self.$pxies.read().bits()
                    }
                    fn pxies_wr(&self, bits: u8) {
                        self.$pxies.write(|w| unsafe { w.bits(bits) })
                    }

                    fn pxie_rd(&self) -> u8 {
                        self.$pxie.read().bits()
                    }
                    fn pxie_wr(&self, bits: u8) {
                        self.$pxie.write(|w| unsafe { w.bits(bits) })
                    }

                    fn pxifg_rd(&self) -> u8 {
                        self.$pxifg.read().bits()
                    }
                    fn pxifg_wr(&self, bits: u8) {
                        self.$pxifg.write(|w| unsafe { w.bits(bits) })
                    }

                    fn pxiv_rd(&self) -> u16 {
                        self.$pxiv.read().bits()
                    }
                }
            )?
        }
    };
}

gpio_impl!(p1: P1 => p1in, p1out, p1dir, p1ren, p1selc, p1sel0, p1sel1, [p1ies, p1ie, p1ifg, p1iv]);
gpio_impl!(p2: P2 => p2in, p2out, p2dir, p2ren, p2selc, p2sel0, p2sel1, [p2ies, p2ie, p2ifg, p2iv]);
gpio_impl!(p3: P3 => p3in, p3out, p3dir, p3ren, p3selc, p3sel0, p3sel1, [p3ies, p3ie, p3ifg, p3iv]);
gpio_impl!(p4: P4 => p4in, p4out, p4dir, p4ren, p4selc, p4sel0, p4sel1, [p4ies, p4ie, p4ifg, p4iv]);
gpio_impl!(p5: P5 => p5in, p5out, p5dir, p5ren, p5selc, p5sel0, p5sel1);
gpio_impl!(p6: P6 => p6in, p6out, p6dir, p6ren, p6selc, p6sel0, p6sel1);
