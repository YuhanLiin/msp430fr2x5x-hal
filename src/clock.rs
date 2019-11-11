use msp430fr2355 as pac;
use pac::cs::csctl1::DCORSEL_A;
use pac::cs::csctl4::{SELA_A, SELMS_A};
pub use pac::cs::csctl5::{DIVM_A as MclkDiv, DIVS_A as SmclkDiv};

/// REFOCLK frequency
pub const REFOCLK: u16 = 32768;
/// VLOCLK frequency
pub const VLOCLK: u16 = 10000;

const MCLK_DIV_EXP: u8 = 7;
const MAX_DCO_MUL_EXP: u8 = 10;

/// Clock sources for MCLK
pub enum MclkSel {
    /// REFOCLK
    Refoclk,
    /// VLOCLK
    Vloclk,
    //Dcoclk { multiplier: u16, range: DCORSEL_A },
}

impl MclkSel {
    fn selms(&self) -> SELMS_A {
        match self {
            MclkSel::Refoclk => SELMS_A::REFOCLK,
            MclkSel::Vloclk => SELMS_A::VLOCLK,
        }
    }

    fn freq(&self) -> u32 {
        match self {
            MclkSel::Vloclk => VLOCLK as u32,
            MclkSel::Refoclk => REFOCLK as u32,
        }
    }
}

/// Clock sources for ACLK
#[derive(Clone, Copy)]
pub enum AclkSel {
    /// VLOCLK
    Vloclk,
    /// REFOCLK
    Refoclk,
}

impl AclkSel {
    fn to_sela(self) -> SELA_A {
        match self {
            AclkSel::Vloclk => SELA_A::VLOCLK,
            AclkSel::Refoclk => SELA_A::REFOCLK,
        }
    }

    fn freq(self) -> u16 {
        match self {
            AclkSel::Vloclk => VLOCLK,
            AclkSel::Refoclk => REFOCLK,
        }
    }
}

#[doc(hidden)]
pub struct Undefined;
#[doc(hidden)]
pub struct MclkDefined;
#[doc(hidden)]
pub struct SmclkDefined(SmclkDiv);
#[doc(hidden)]
pub struct SmclkDisabled;

#[doc(hidden)]
pub trait SmclkState {
    fn div(&self) -> Option<SmclkDiv>;
}

impl SmclkState for SmclkDefined {
    fn div(&self) -> Option<SmclkDiv> {
        Some(self.0)
    }
}

impl SmclkState for SmclkDisabled {
    fn div(&self) -> Option<SmclkDiv> {
        None
    }
}

/// Builder object containing system clock configuration. Configuring MCLK must happen before SMCLK
/// is configured. SMCLK can be optionally disabled, in which case a `Smclk` object will not be
/// produced. Configuring ACLK select is optional, with its default being REFOCLK.
pub struct ClockConfig<MODE> {
    periph: pac::CS,
    mclk_sel: MclkSel,
    mclk_div: MclkDiv,
    aclk_sel: AclkSel,
    mode: MODE,
}

macro_rules! make_clkconf {
    ($conf:expr, $mode:expr) => {
        ClockConfig {
            periph: $conf.periph,
            mclk_sel: $conf.mclk_sel,
            mclk_div: $conf.mclk_div,
            aclk_sel: $conf.aclk_sel,
            mode: $mode,
        }
    };
}

/// Extension trait allowing the PAC CS struct to be converted into the HAL clock configuration
/// builder object.
pub trait CsExt {
    /// Converts CS into clock configuration builder object
    fn constrain(self) -> ClockConfig<Undefined>;
}

impl CsExt for pac::CS {
    fn constrain(self) -> ClockConfig<Undefined> {
        // These are the microcontroller default settings
        ClockConfig {
            periph: self,
            mode: Undefined,
            mclk_div: MclkDiv::_1,
            mclk_sel: MclkSel::Refoclk,
            aclk_sel: AclkSel::Refoclk,
        }
    }
}

impl<MODE> ClockConfig<MODE> {
    /// Select REFOCLK for ACLK
    pub const fn aclk_refoclk(mut self) -> Self {
        self.aclk_sel = AclkSel::Refoclk;
        self
    }

    /// Select VLOCLK for ACLK
    pub const fn aclk_vloclk(mut self) -> Self {
        self.aclk_sel = AclkSel::Vloclk;
        self
    }
}

impl ClockConfig<Undefined> {
    /// Select REFOCLK for MCLK and set the MCLK divider
    pub const fn mclk_refoclk(self, mclk_div: MclkDiv) -> ClockConfig<MclkDefined> {
        ClockConfig {
            mclk_div,
            mclk_sel: MclkSel::Refoclk,
            ..make_clkconf!(self, MclkDefined)
        }
    }

    /// Select VLOCLK for MCLK and set the MCLK divider
    pub const fn mclk_vcoclk(self, mclk_div: MclkDiv) -> ClockConfig<MclkDefined> {
        ClockConfig {
            mclk_div,
            mclk_sel: MclkSel::Vloclk,
            ..make_clkconf!(self, MclkDefined)
        }
    }
}

impl ClockConfig<MclkDefined> {
    /// Enable SMCLK and set SMCLK divider, which divides the MCLK frequency
    pub const fn smclk_on(self, div: SmclkDiv) -> ClockConfig<SmclkDefined> {
        make_clkconf!(self, SmclkDefined(div))
    }

    /// Disable SMCLK
    pub const fn smclk_off(self) -> ClockConfig<SmclkDisabled> {
        make_clkconf!(self, SmclkDisabled)
    }
}

impl<MODE: SmclkState> ClockConfig<MODE> {
    fn configure_periph(&self) {
        self.periph.csctl4.write(|w| {
            w.sela()
                .variant(self.aclk_sel.to_sela())
                .selms()
                .variant(self.mclk_sel.selms())
        });

        self.periph.csctl5.write(|w| {
            let w = w.vloautooff().set_bit().divm().variant(self.mclk_div);
            match self.mode.div() {
                Some(div) => w.divs().variant(div),
                None => w.smclkoff().set_bit(),
            }
        });
    }
}

impl ClockConfig<SmclkDefined> {
    /// Apply clock configuration and return MCLK, SMCLK, and ACLK clock objects
    pub fn freeze(self) -> (Mclk, Smclk, Aclk) {
        self.configure_periph();
        // The clock divider enums are ordered such that their numerical values are the log2 values
        // of the frequency divisor
        let mclk_freq = self.mclk_sel.freq() >> (self.mclk_div as u32);
        (
            Mclk(mclk_freq),
            Smclk(mclk_freq >> (self.mode.0 as u32)),
            Aclk(self.aclk_sel.freq()),
        )
    }
}

impl ClockConfig<SmclkDisabled> {
    /// Apply clock configuration and return MCLK and ACLK clock objects, as SMCLK is disabled
    pub fn freeze(self) -> (Mclk, Aclk) {
        self.configure_periph();
        let mclk_freq = self.mclk_sel.freq() >> (self.mclk_div as u32);
        (Mclk(mclk_freq), Aclk(self.aclk_sel.freq()))
    }
}

/// MCLK clock object
pub struct Mclk(u32);
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

impl Clock for Mclk {
    type Freq = u32;

    fn freq(&self) -> u32 {
        self.0
    }
}

impl Clock for Smclk {
    type Freq = u32;

    fn freq(&self) -> u32 {
        self.0
    }
}

impl Clock for Aclk {
    type Freq = u16;

    fn freq(&self) -> u16 {
        self.0
    }
}
