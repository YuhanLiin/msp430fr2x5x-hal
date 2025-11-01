use super::Steal;
use crate::pac;

pub trait GpioPeriph: Steal {
    fn pxin_rd(&self) -> u8;

    fn pxout_rd(&self) -> u8;
    fn pxout_wr(&self, bits: u8);
    fn pxout_set(&self, bits: u8);
    fn pxout_clear(&self, bits: u8);
    fn pxout_toggle(&self, bits: u8);

    fn pxdir_rd(&self) -> u8;
    fn pxdir_wr(&self, bits: u8);
    fn pxdir_set(&self, bits: u8);
    fn pxdir_clear(&self, bits: u8);

    fn pxren_rd(&self) -> u8;
    fn pxren_wr(&self, bits: u8);
    fn pxren_set(&self, bits: u8);
    fn pxren_clear(&self, bits: u8);

    fn pxselc_wr(&self, bits: u8);

    fn pxsel0_rd(&self) -> u8;
    fn pxsel0_wr(&self, bits: u8);
    fn pxsel0_set(&self, bits: u8);
    fn pxsel0_clear(&self, bits: u8);

    fn pxsel1_rd(&self) -> u8;
    fn pxsel1_wr(&self, bits: u8);
    fn pxsel1_set(&self, bits: u8);
    fn pxsel1_clear(&self, bits: u8);
}

pub trait IntrPeriph: GpioPeriph {
    fn pxies_rd(&self) -> u8;
    fn pxies_wr(&self, bits: u8);
    fn pxies_set(&self, bits: u8);
    fn pxies_clear(&self, bits: u8);

    fn pxie_rd(&self) -> u8;
    fn pxie_wr(&self, bits: u8);
    fn pxie_set(&self, bits: u8);
    fn pxie_clear(&self, bits: u8);

    fn pxifg_rd(&self) -> u8;
    fn pxifg_wr(&self, bits: u8);
    fn pxifg_set(&self, bits: u8);
    fn pxifg_clear(&self, bits: u8);

    fn pxiv_rd(&self) -> u16;
}

macro_rules! reg_methods {
    ($reg:ident, $rd:ident, $wr:ident, $set:ident, $clear:ident) => {
        #[inline(always)]
        fn $rd(&self) -> u8 {
            self.$reg.read().bits()
        }

        #[inline(always)]
        fn $wr(&self, bits: u8) {
            self.$reg.write(|w| unsafe { w.bits(bits) });
        }

        #[inline(always)]
        fn $set(&self, bits: u8) {
            unsafe { self.$reg.set_bits(|w| w.bits(bits)) }
        }

        #[inline(always)]
        fn $clear(&self, bits: u8) {
            unsafe { self.$reg.clear_bits(|w| w.bits(bits)) }
        }
    };
}

macro_rules! gpio_impl {
    ($px:ident: $Px:ident =>
     $pxin:ident, $pxout:ident, $pxdir:ident, $pxren:ident, $pxselc:ident, $pxsel0:ident, $pxsel1:ident
     $(, [$pxies:ident, $pxie:ident, $pxifg:ident, $pxiv:ident])?
    ) => {
        mod $px {
            use super::*;

            impl Steal for pac::$Px {
                #[inline(always)]
                unsafe fn steal() -> Self {
                    pac::Peripherals::conjure().$Px
                }
            }

            impl GpioPeriph for pac::$Px {
                #[inline(always)]
                fn pxin_rd(&self) -> u8 {
                    self.$pxin.read().bits()
                }

                #[inline(always)]
                fn pxselc_wr(&self, bits: u8) {
                    self.$pxselc.write(|w| unsafe { w.bits(bits) })
                }

                #[inline(always)]
                fn pxout_toggle(&self, bits: u8) {
                    unsafe { self.$pxout.toggle_bits(|w| w.bits(bits)) };
                }

                reg_methods!($pxout, pxout_rd, pxout_wr, pxout_set, pxout_clear);
                reg_methods!($pxdir, pxdir_rd, pxdir_wr, pxdir_set, pxdir_clear);
                reg_methods!($pxren, pxren_rd, pxren_wr, pxren_set, pxren_clear);
                reg_methods!($pxsel0, pxsel0_rd, pxsel0_wr, pxsel0_set, pxsel0_clear);
                reg_methods!($pxsel1, pxsel1_rd, pxsel1_wr, pxsel1_set, pxsel1_clear);
            }

            $(
                impl IntrPeriph for pac::$Px {
                    reg_methods!($pxies, pxies_rd, pxies_wr, pxies_set, pxies_clear);
                    reg_methods!($pxie, pxie_rd, pxie_wr, pxie_set, pxie_clear);
                    reg_methods!($pxifg, pxifg_rd, pxifg_wr, pxifg_set, pxifg_clear);

                    #[inline(always)]
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
