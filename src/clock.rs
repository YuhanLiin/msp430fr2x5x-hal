//! Clock system for configuration of MCLK, SMCLK, and ACLK.
//!
//! Once configuration is complete, `Aclk` and `Smclk` clock objects are returned for configuring
//! other peripherals.
//! Configuration of MCLK and SMCLK *must* occur, though SMCLK can be disabled. In that case, only
//! `Aclk` is returned.
//!
//! DCO with FLL is supported on MCLK for select frequencies, as supporting other frequency values
//! would require complex calibrations not handled by the HAL.

use crate::fram::{Fram, WaitStates};
use msp430fr2355 as pac;
use pac::cs::csctl1::DCORSEL_A;
use pac::cs::csctl4::{SELA_A, SELMS_A};
pub use pac::cs::csctl5::{DIVM_A as MclkDiv, DIVS_A as SmclkDiv};

/// REFOCLK frequency
pub const REFOCLK: u16 = 32768;
/// VLOCLK frequency
pub const VLOCLK: u16 = 10000;

enum MclkSel {
    Refoclk,
    Vloclk,
    Dcoclk(DcoclkFreqSel),
}

impl MclkSel {
    #[inline]
    fn freq(&self) -> u32 {
        match self {
            MclkSel::Vloclk => VLOCLK as u32,
            MclkSel::Refoclk => REFOCLK as u32,
            MclkSel::Dcoclk(sel) => sel.freq(),
        }
    }

    #[inline(always)]
    fn selms(&self) -> SELMS_A {
        match self {
            MclkSel::Vloclk => SELMS_A::VLOCLK,
            MclkSel::Refoclk => SELMS_A::REFOCLK,
            MclkSel::Dcoclk(_) => SELMS_A::DCOCLKDIV,
        }
    }
}

#[derive(Clone, Copy)]
enum AclkSel {
    Vloclk,
    Refoclk,
}

impl AclkSel {
    #[inline(always)]
    fn sela(self) -> SELA_A {
        match self {
            AclkSel::Vloclk => SELA_A::VLOCLK,
            AclkSel::Refoclk => SELA_A::REFOCLK,
        }
    }

    #[inline(always)]
    fn freq(self) -> u16 {
        match self {
            AclkSel::Vloclk => VLOCLK,
            AclkSel::Refoclk => REFOCLK,
        }
    }
}

/// Selectable DCOCLK frequencies when using factory trim settings.
/// Actual frequencies may be slightly higher.
#[derive(Clone, Copy)]
pub enum DcoclkFreqSel {
    /// 1 MHz
    _1MHz,
    /// 2 MHz
    _2MHz,
    /// 4 MHz
    _4MHz,
    /// 8 MHz
    _8MHz,
    /// 12 MHz
    _12MHz,
    /// 16 MHz
    _16MHz,
    /// 20 MHz
    _20MHz,
    /// 24 MHz
    _24MHz,
}

impl DcoclkFreqSel {
    #[inline(always)]
    fn dcorsel(self) -> DCORSEL_A {
        match self {
            DcoclkFreqSel::_1MHz => DCORSEL_A::DCORSEL_0,
            DcoclkFreqSel::_2MHz => DCORSEL_A::DCORSEL_1,
            DcoclkFreqSel::_4MHz => DCORSEL_A::DCORSEL_2,
            DcoclkFreqSel::_8MHz => DCORSEL_A::DCORSEL_3,
            DcoclkFreqSel::_12MHz => DCORSEL_A::DCORSEL_4,
            DcoclkFreqSel::_16MHz => DCORSEL_A::DCORSEL_5,
            DcoclkFreqSel::_20MHz => DCORSEL_A::DCORSEL_6,
            DcoclkFreqSel::_24MHz => DCORSEL_A::DCORSEL_7,
        }
    }

    #[inline(always)]
    fn multiplier(self) -> u16 {
        match self {
            DcoclkFreqSel::_1MHz => 32,
            DcoclkFreqSel::_2MHz => 61,
            DcoclkFreqSel::_4MHz => 122,
            DcoclkFreqSel::_8MHz => 245,
            DcoclkFreqSel::_12MHz => 366,
            DcoclkFreqSel::_16MHz => 490,
            DcoclkFreqSel::_20MHz => 610,
            DcoclkFreqSel::_24MHz => 732,
        }
    }

    /// Numerical frequency
    #[inline]
    pub fn freq(self) -> u32 {
        (self.multiplier() as u32) * (REFOCLK as u32)
    }
}

#[doc(hidden)]
pub struct NoClockDefined;
#[doc(hidden)]
pub struct MclkDefined(MclkSel);
#[doc(hidden)]
pub struct SmclkDefined(SmclkDiv);
#[doc(hidden)]
pub struct SmclkDisabled;

#[doc(hidden)]
pub trait SmclkState {
    fn div(&self) -> Option<SmclkDiv>;
}

impl SmclkState for SmclkDefined {
    #[inline(always)]
    fn div(&self) -> Option<SmclkDiv> {
        Some(self.0)
    }
}

impl SmclkState for SmclkDisabled {
    #[inline(always)]
    fn div(&self) -> Option<SmclkDiv> {
        None
    }
}

/// Builder object containing system clock configuration
pub struct ClockConfig<MCLK, SMCLK> {
    periph: pac::CS,
    mclk: MCLK,
    mclk_div: MclkDiv,
    aclk_sel: AclkSel,
    smclk: SMCLK,
}

macro_rules! make_clkconf {
    ($conf:expr, $mclk:expr, $smclk:expr) => {
        ClockConfig {
            periph: $conf.periph,
            mclk: $mclk,
            mclk_div: $conf.mclk_div,
            aclk_sel: $conf.aclk_sel,
            smclk: $smclk,
        }
    };
}

/// Extension trait allowing the PAC CS struct to be converted into the HAL clock configuration
/// builder object.
pub trait CsExt {
    /// Converts CS into clock configuration builder object
    fn constrain(self) -> ClockConfig<NoClockDefined, NoClockDefined>;
}

impl CsExt for pac::CS {
    #[inline]
    fn constrain(self) -> ClockConfig<NoClockDefined, NoClockDefined> {
        // These are the microcontroller default settings
        ClockConfig {
            periph: self,
            smclk: NoClockDefined,
            mclk: NoClockDefined,
            mclk_div: MclkDiv::_1,
            aclk_sel: AclkSel::Refoclk,
        }
    }
}

impl<MCLK, SMCLK> ClockConfig<MCLK, SMCLK> {
    /// Select REFOCLK for ACLK
    #[inline]
    pub fn aclk_refoclk(mut self) -> Self {
        self.aclk_sel = AclkSel::Refoclk;
        self
    }

    /// Select VLOCLK for ACLK
    #[inline]
    pub fn aclk_vloclk(mut self) -> Self {
        self.aclk_sel = AclkSel::Vloclk;
        self
    }

    /// Select REFOCLK for MCLK and set the MCLK divider. Frequency is `10000 / mclk_div` Hz.
    #[inline]
    pub fn mclk_refoclk(self, mclk_div: MclkDiv) -> ClockConfig<MclkDefined, SMCLK> {
        ClockConfig {
            mclk_div,
            ..make_clkconf!(self, MclkDefined(MclkSel::Refoclk), self.smclk)
        }
    }

    /// Select VLOCLK for MCLK and set the MCLK divider. Frequency is `32768 / mclk_div` Hz.
    #[inline]
    pub fn mclk_vcoclk(self, mclk_div: MclkDiv) -> ClockConfig<MclkDefined, SMCLK> {
        ClockConfig {
            mclk_div,
            ..make_clkconf!(self, MclkDefined(MclkSel::Vloclk), self.smclk)
        }
    }

    /// Select DCOCLK for MCLK with FLL for stabilization. Frequency is `target_freq / mclk_div` Hz.
    /// This setting selects the default factory trim for DCO trimming and performs no extra
    /// calibration, so only a select few frequency targets can be selected.
    #[inline]
    pub fn mclk_dcoclk(
        self,
        target_freq: DcoclkFreqSel,
        mclk_div: MclkDiv,
    ) -> ClockConfig<MclkDefined, SMCLK> {
        ClockConfig {
            mclk_div,
            ..make_clkconf!(self, MclkDefined(MclkSel::Dcoclk(target_freq)), self.smclk)
        }
    }

    /// Enable SMCLK and set SMCLK divider, which divides the MCLK frequency
    #[inline]
    pub fn smclk_on(self, div: SmclkDiv) -> ClockConfig<MCLK, SmclkDefined> {
        make_clkconf!(self, self.mclk, SmclkDefined(div))
    }

    /// Disable SMCLK
    #[inline]
    pub fn smclk_off(self) -> ClockConfig<MCLK, SmclkDisabled> {
        make_clkconf!(self, self.mclk, SmclkDisabled)
    }
}

#[inline(always)]
fn fll_off() {
    const FLAG: u8 = 1 << 6;
    unsafe { asm!("bis.b $0, SR" :: "i"(FLAG) : "memory" : "volatile") };
}

#[inline(always)]
fn fll_on() {
    const FLAG: u8 = 1 << 6;
    unsafe { asm!("bic.b $0, SR" :: "i"(FLAG) : "memory" : "volatile") };
}

impl<SMCLK: SmclkState> ClockConfig<MclkDefined, SMCLK> {
    fn configure_dco_fll(&self) {
        // Run FLL configuration procedure from the user's guide if we are using DCO
        if let MclkSel::Dcoclk(target_freq) = self.mclk.0 {
            fll_off();
            self.periph.csctl3.write(|w| w.selref().refoclk());
            self.periph.csctl0.write(|w| unsafe { w.bits(0) });
            self.periph
                .csctl1
                .write(|w| w.dcorsel().variant(target_freq.dcorsel()));
            self.periph.csctl2.write(|w| {
                unsafe { w.flln().bits(target_freq.multiplier() - 1) }
                    .flld()
                    ._1()
            });

            msp430::asm::nop();
            msp430::asm::nop();
            msp430::asm::nop();
            fll_on();

            while !self.periph.csctl7.read().fllunlock().is_fllunlock_0() {}
        }
    }

    #[inline]
    fn configure_cs(&self) {
        // Configure clock selector and divisors
        self.periph.csctl4.write(|w| {
            w.sela()
                .variant(self.aclk_sel.sela())
                .selms()
                .variant(self.mclk.0.selms())
        });

        self.periph.csctl5.write(|w| {
            let w = w.vloautooff().set_bit().divm().variant(self.mclk_div);
            match self.smclk.div() {
                Some(div) => w.divs().variant(div),
                None => w.smclkoff().set_bit(),
            }
        });
    }

    #[inline]
    unsafe fn configure_fram(fram: &mut Fram, mclk_freq: u32) {
        if mclk_freq > 16_000_000 {
            fram.set_wait_states(WaitStates::Wait2);
        } else if mclk_freq > 8_000_000 {
            fram.set_wait_states(WaitStates::Wait1);
        } else {
            fram.set_wait_states(WaitStates::Wait0);
        }
    }
}

impl ClockConfig<MclkDefined, SmclkDefined> {
    /// Apply clock configuration and return MCLK, SMCLK, and ACLK clock objects
    #[inline]
    pub fn freeze(self, fram: &mut Fram) -> (Smclk, Aclk) {
        let mclk_freq = self.mclk.0.freq() >> (self.mclk_div as u32);
        self.configure_dco_fll();
        unsafe { Self::configure_fram(fram, mclk_freq) };
        self.configure_cs();
        (
            Smclk(mclk_freq >> (self.smclk.0 as u32)),
            Aclk(self.aclk_sel.freq()),
        )
    }
}

impl ClockConfig<MclkDefined, SmclkDisabled> {
    /// Apply clock configuration and return MCLK and ACLK clock objects, as SMCLK is disabled
    #[inline]
    pub fn freeze(self, fram: &mut Fram) -> Aclk {
        let mclk_freq = self.mclk.0.freq() >> (self.mclk_div as u32);
        self.configure_dco_fll();
        unsafe { Self::configure_fram(fram, mclk_freq) };
        self.configure_cs();
        Aclk(self.aclk_sel.freq())
    }
}

/// SMCLK clock object
pub struct Smclk(u32);
/// ACLK clock object
pub struct Aclk(u16);

/// Trait for configured clock objects
pub trait Clock {
    /// Type of the returned frequency value
    type Freq;

    /// Frequency of the clock
    fn freq(&self) -> Self::Freq;
}

impl Clock for Smclk {
    type Freq = u32;

    /// Returning a 32-bit frequency may seem suspect, since we're on a 16-bit system, but it is
    /// required as SMCLK can go up to 24 MHz. Clock frequencies are usually for initialization
    /// tasks such as computing baud rates, which should be optimized away, avoiding the extra cost
    /// of 32-bit computations.
    #[inline]
    fn freq(&self) -> u32 {
        self.0
    }
}

impl Clock for Aclk {
    type Freq = u16;

    #[inline]
    fn freq(&self) -> u16 {
        self.0
    }
}
