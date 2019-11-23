use crate::fram::{Fram, WaitStates};
use core::marker::PhantomData;
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
    fn selms(&self) -> SELMS_A {
        match self {
            MclkSel::Refoclk => SELMS_A::REFOCLK,
            MclkSel::Vloclk => SELMS_A::VLOCLK,
            MclkSel::Dcoclk(_) => SELMS_A::DCOCLKDIV,
        }
    }

    #[inline]
    fn freq(&self) -> u32 {
        match self {
            MclkSel::Vloclk => VLOCLK as u32,
            MclkSel::Refoclk => REFOCLK as u32,
            MclkSel::Dcoclk(fsel) => fsel.freq(),
        }
    }
}

#[derive(Clone, Copy)]
enum AclkSel {
    Vloclk,
    Refoclk,
}

impl AclkSel {
    #[inline]
    fn sela(self) -> SELA_A {
        match self {
            AclkSel::Vloclk => SELA_A::VLOCLK,
            AclkSel::Refoclk => SELA_A::REFOCLK,
        }
    }

    #[inline]
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
    #[inline]
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

    #[inline]
    fn multiplier(self) -> u16 {
        match self {
            DcoclkFreqSel::_1MHz => 32,
            DcoclkFreqSel::_2MHz => 64,
            DcoclkFreqSel::_4MHz => 128,
            DcoclkFreqSel::_8MHz => 256,
            DcoclkFreqSel::_12MHz => 384,
            DcoclkFreqSel::_16MHz => 512,
            DcoclkFreqSel::_20MHz => 640,
            DcoclkFreqSel::_24MHz => 738,
        }
    }

    /// Numerical frequency
    #[inline]
    pub fn freq(self) -> u32 {
        (self.multiplier() as u32) * (REFOCLK as u32)
    }
}

#[doc(hidden)]
pub struct Undefined;
#[doc(hidden)]
pub struct MclkDefined;
#[doc(hidden)]
pub struct MclkDcoDefined;
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

#[doc(hidden)]
pub trait MclkState {}
impl MclkState for MclkDefined {}
impl MclkState for MclkDcoDefined {}

/// Builder object containing system clock configuration. Configuring MCLK must happen before SMCLK
/// is configured. SMCLK can be optionally disabled, in which case a `Smclk` object will not be
/// produced. Configuring ACLK select is optional, with its default being REFOCLK.
pub struct ClockConfig<MCLK, SMCLK> {
    periph: pac::CS,
    mclk_sel: MclkSel,
    mclk_div: MclkDiv,
    aclk_sel: AclkSel,
    smclk: SMCLK,
    _mclk: PhantomData<MCLK>,
}

macro_rules! make_clkconf {
    ($conf:expr, $smclk:expr) => {
        ClockConfig {
            periph: $conf.periph,
            mclk_sel: $conf.mclk_sel,
            mclk_div: $conf.mclk_div,
            aclk_sel: $conf.aclk_sel,
            smclk: $smclk,
            _mclk: PhantomData,
        }
    };
}

/// Extension trait allowing the PAC CS struct to be converted into the HAL clock configuration
/// builder object.
pub trait CsExt {
    /// Converts CS into clock configuration builder object
    fn constrain(self) -> ClockConfig<Undefined, Undefined>;
}

impl CsExt for pac::CS {
    #[inline(always)]
    fn constrain(self) -> ClockConfig<Undefined, Undefined> {
        // These are the microcontroller default settings
        ClockConfig {
            periph: self,
            smclk: Undefined,
            _mclk: PhantomData,
            mclk_div: MclkDiv::_1,
            mclk_sel: MclkSel::Refoclk,
            aclk_sel: AclkSel::Refoclk,
        }
    }
}

impl<MCLK, SMCLK> ClockConfig<MCLK, SMCLK> {
    /// Select REFOCLK for ACLK
    #[inline(always)]
    pub const fn aclk_refoclk(mut self) -> Self {
        self.aclk_sel = AclkSel::Refoclk;
        self
    }

    /// Select VLOCLK for ACLK
    #[inline(always)]
    pub const fn aclk_vloclk(mut self) -> Self {
        self.aclk_sel = AclkSel::Vloclk;
        self
    }
}

impl ClockConfig<Undefined, Undefined> {
    /// Select REFOCLK for MCLK and set the MCLK divider. Frequency is `10000 / mclk_div` Hz.
    #[inline(always)]
    pub const fn mclk_refoclk(self, mclk_div: MclkDiv) -> ClockConfig<MclkDefined, Undefined> {
        ClockConfig {
            mclk_div,
            mclk_sel: MclkSel::Refoclk,
            ..make_clkconf!(self, Undefined)
        }
    }

    /// Select VLOCLK for MCLK and set the MCLK divider. Frequency is `32768 / mclk_div` Hz.
    #[inline(always)]
    pub const fn mclk_vcoclk(self, mclk_div: MclkDiv) -> ClockConfig<MclkDefined, Undefined> {
        ClockConfig {
            mclk_div,
            mclk_sel: MclkSel::Vloclk,
            ..make_clkconf!(self, Undefined)
        }
    }

    /// Select DCOCLK for MCLK with FLL for stabilization. Frequency is `target_freq / mclk_div` Hz.
    /// This setting selects the default factory trim for DCO trimming and performs no extra
    /// calibration, so only a select few frequency targets can be selected.
    #[inline(always)]
    pub const fn mclk_dcoclk(
        self,
        target_freq: DcoclkFreqSel,
        mclk_div: MclkDiv,
    ) -> ClockConfig<MclkDcoDefined, Undefined> {
        ClockConfig {
            mclk_div,
            mclk_sel: MclkSel::Dcoclk(target_freq),
            ..make_clkconf!(self, Undefined)
        }
    }
}

impl<MCLK: MclkState> ClockConfig<MCLK, Undefined> {
    /// Enable SMCLK and set SMCLK divider, which divides the MCLK frequency
    #[inline(always)]
    pub fn smclk_on(self, div: SmclkDiv) -> ClockConfig<MCLK, SmclkDefined> {
        make_clkconf!(self, SmclkDefined(div))
    }

    /// Disable SMCLK
    #[inline(always)]
    pub fn smclk_off(self) -> ClockConfig<MCLK, SmclkDisabled> {
        make_clkconf!(self, SmclkDisabled)
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

impl<SMCLK: SmclkState> ClockConfig<MclkDcoDefined, SMCLK> {
    #[inline]
    fn configure_dco_fll(&self) {
        // FLL configuration procedure from the user's guide
        // Should always be true
        if let MclkSel::Dcoclk(target_freq) = self.mclk_sel {
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
        } else {
            debug_assert!(false);
        }
    }

    #[inline]
    unsafe fn configure_fram(&self, fram: &mut Fram, mclk_freq: u32) {
        if mclk_freq > 16_000_000 {
            fram.set_wait_states(WaitStates::Wait2);
        } else if mclk_freq > 8_000_000 {
            fram.set_wait_states(WaitStates::Wait1);
        }
    }
}

impl<MCLK: MclkState, SMCLK: SmclkState> ClockConfig<MCLK, SMCLK> {
    #[inline]
    fn configure_cs(&self) {
        self.periph.csctl4.write(|w| {
            w.sela()
                .variant(self.aclk_sel.sela())
                .selms()
                .variant(self.mclk_sel.selms())
        });

        self.periph.csctl5.write(|w| {
            let w = w.vloautooff().set_bit().divm().variant(self.mclk_div);
            match self.smclk.div() {
                Some(div) => w.divs().variant(div),
                None => w.smclkoff().set_bit(),
            }
        });
    }
}

impl ClockConfig<MclkDefined, SmclkDefined> {
    /// Apply clock configuration and return MCLK, SMCLK, and ACLK clock objects
    #[inline]
    pub fn freeze(self) -> (Smclk, Aclk) {
        self.configure_cs();
        let mclk_freq = self.mclk_sel.freq() >> (self.mclk_div as u32);
        (
            Smclk(mclk_freq >> (self.smclk.0 as u32)),
            Aclk(self.aclk_sel.freq()),
        )
    }
}

impl ClockConfig<MclkDefined, SmclkDisabled> {
    /// Apply clock configuration and return MCLK and ACLK clock objects, as SMCLK is disabled
    #[inline]
    pub fn freeze(self) -> Aclk {
        self.configure_cs();
        Aclk(self.aclk_sel.freq())
    }
}

impl ClockConfig<MclkDcoDefined, SmclkDefined> {
    /// Apply clock configuration and return MCLK, SMCLK, and ACLK clock objects.
    /// Possibly set FRAM wait state if MCLK goes above 8MHz (1 cycle wait) or 16MHz (2 cycle
    /// wait). Do not change wait state afterwards.
    #[inline]
    pub fn freeze(self, fram: &mut Fram) -> (Smclk, Aclk) {
        let mclk_freq = self.mclk_sel.freq() >> (self.mclk_div as u32);
        self.configure_cs();
        unsafe { self.configure_fram(fram, mclk_freq) };
        self.configure_dco_fll();
        (
            Smclk(mclk_freq >> (self.smclk.0 as u32)),
            Aclk(self.aclk_sel.freq()),
        )
    }
}

impl ClockConfig<MclkDcoDefined, SmclkDisabled> {
    /// Apply clock configuration and return MCLK and ACLK clock objects, as SMCLK is disabled
    #[inline]
    pub fn freeze(self, fram: &mut Fram) -> Aclk {
        let mclk_freq = self.mclk_sel.freq() >> (self.mclk_div as u32);
        self.configure_cs();
        unsafe { self.configure_fram(fram, mclk_freq) };
        self.configure_dco_fll();
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
