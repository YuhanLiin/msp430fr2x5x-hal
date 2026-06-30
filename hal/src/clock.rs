//! Clock system for configuration of MCLK, SMCLK, ACLK, and XT1.
//!
//! Once configuration is complete, `Aclk`, `Smclk`, and optionally `Xt1clk` clock objects 
//! are returned. These objects are used to set the clock sources on other peripherals.
//!
//! Configuration of MCLK and SMCLK *must* occur, though SMCLK can be disabled. XT1 
//! configuration is optional but, when enabled, provides a high-precision source for 
//! system clocks or the FLL reference.
//!
//! DCO with FLL is supported on MCLK for select frequencies. The FLL can be 
//! referenced by either the internal REFO or the external XT1 crystal. Supporting 
//! arbitrary frequencies on the DCO requires complex calibration routines not 
//! supported by the HAL.

use core::arch::asm;
use core::marker::PhantomData;

use crate::delay::SysDelay;
use crate::fram::{Fram, WaitStates};
use crate::_pac::{
    self,
    cs::{
        csctl1::Dcorsel,
        csctl4::{Sela, Selms},
        csctl3::{Selref, Fllrefdiv},
        csctl6::Xt1drive,
    },
};
pub use crate::_pac::cs::csctl5::{Divm as MclkDiv, Divs as SmclkDiv};

#[cfg(feature = "xt1_high_frequency")]
use crate::_pac::cs::csctl6::{Xt1hffreq, Xts};

/// REFOCLK frequency
pub const REFOCLK_FREQ_HZ: u16 = 32768;
/// VLOCLK frequency
pub const VLOCLK_FREQ_HZ: u16 = 10000;
pub use crate::device_specific::MODCLK_FREQ_HZ;

enum MclkSel {
    Refoclk,
    Vloclk,
    Dcoclk(DcoclkFreqSel),
    Xt1clk(u32),
}

impl MclkSel {
    #[inline]
    fn freq(&self) -> u32 {
        match self {
            MclkSel::Vloclk => VLOCLK_FREQ_HZ as u32,
            MclkSel::Refoclk => REFOCLK_FREQ_HZ as u32,
            MclkSel::Dcoclk(sel) => sel.freq(),
            MclkSel::Xt1clk(freq) => *freq,
        }
    }

    #[inline(always)]
    fn selms(&self) -> Selms {
        match self {
            MclkSel::Vloclk => Selms::Vloclk,
            MclkSel::Refoclk => Selms::Refoclk,
            MclkSel::Dcoclk(_) => Selms::Dcoclkdiv,
            MclkSel::Xt1clk(_) => Selms::Xt1clk,
        }
    }
}

#[derive(Clone, Copy)]
enum AclkSel {
    #[cfg(feature = "vloclk_source")]
    Vloclk,
    Refoclk,
    Xt1clk(u32),
}

impl AclkSel {
    #[inline(always)]
    fn sela(self) -> Sela {
        match self {
            #[cfg(feature = "vloclk_source")]
            AclkSel::Vloclk => Sela::Vloclk,
            AclkSel::Refoclk => Sela::Refoclk,
            AclkSel::Xt1clk(_) => Sela::Xt1clk,
        }
    }

    #[inline(always)]
    fn freq(self) -> u32 {
        match self {
            #[cfg(feature = "vloclk_source")]
            AclkSel::Vloclk => VLOCLK_FREQ_HZ as u32,
            AclkSel::Refoclk => REFOCLK_FREQ_HZ as u32,
            AclkSel::Xt1clk(freq) => freq,
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
    #[cfg(feature = "enhanced_cs")]
    /// 20 MHz
    _20MHz,
    #[cfg(feature = "enhanced_cs")]
    /// 24 MHz
    _24MHz,
}

impl DcoclkFreqSel {
    #[inline(always)]
    fn dcorsel(self) -> Dcorsel {
        match self {
            DcoclkFreqSel::_1MHz => Dcorsel::Dcorsel0,
            DcoclkFreqSel::_2MHz => Dcorsel::Dcorsel1,
            DcoclkFreqSel::_4MHz => Dcorsel::Dcorsel2,
            DcoclkFreqSel::_8MHz => Dcorsel::Dcorsel3,
            DcoclkFreqSel::_12MHz => Dcorsel::Dcorsel4,
            DcoclkFreqSel::_16MHz => Dcorsel::Dcorsel5,
            #[cfg(feature = "enhanced_cs")]
            DcoclkFreqSel::_20MHz => Dcorsel::Dcorsel6,
            #[cfg(feature = "enhanced_cs")]
            DcoclkFreqSel::_24MHz => Dcorsel::Dcorsel7,
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
            #[cfg(feature = "enhanced_cs")]
            DcoclkFreqSel::_20MHz => 610,
            #[cfg(feature = "enhanced_cs")]
            DcoclkFreqSel::_24MHz => 732,
        }
    }

    /// Numerical frequency
    #[inline]
    pub fn freq(self) -> u32 {
        (self.multiplier() as u32) * (REFOCLK_FREQ_HZ as u32)
    }
}

/// Typestate for `ClockConfig` that represents unconfigured clocks
pub struct NoClockDefined;
/// Typestate for `ClockConfig` that represents a configured MCLK
pub struct MclkDefined(MclkSel);
/// Typestate for `ClockConfig` that represents a configured SMCLK
pub struct SmclkDefined(SmclkDiv);
/// Typestate for `ClockConfig` that represents disabled SMCLK
pub struct SmclkDisabled;
/// Typestate for `ClockConfig` that represents a configured XT1CLK
pub struct Xt1Defined<MODE>(Xt1Config<MODE>);
/// Typestate for `ClockConfig` that represents disabled/unconfigured XT1CLK
pub struct Xt1Disabled;


/// Valid XT1 XIN (input) pin
pub trait Xt1XinPin {}

/// Valid XT1 XOUT (output) pin
pub trait Xt1XoutPin {}


/// Typestate marker for XT1 **crystal mode** (external crystal, oscillator enabled).
pub struct CrystalMode;

/// Typestate marker for XT1 **bypass mode** (external clock input, oscillator disabled).
pub struct BypassMode;

/// Configuration object for the XT1 oscillator.
///
/// This struct defines how the XT1 clock source should be initialized when
/// applying the clock configuration via [`ClockConfig::xt1clk_on`].
///
/// XT1 can operate in two modes:
///
/// - **Crystal mode**: Uses an external crystal connected to both XIN and XOUT.
/// - **Bypass mode**: Uses an external clock signal fed into XIN only.
///
/// The configuration includes:
/// - The input frequency (used for timing calculations and routing decisions)
/// - Drive strength for the oscillator (relevant in crystal mode)
/// - Whether bypass mode is enabled
/// - Automatic Gain Control (AGC) behavior
pub struct Xt1Config<MODE> {
    frequency: u32,
    drive: Xt1drive,
    agc: bool,
    start_counter: bool,
    bypass: bool,
    #[cfg(feature = "enhanced_cs")]
    fault_switch: bool,
    auto_off: bool,
    _mode: PhantomData<MODE>,
}

#[cfg(feature = "enhanced_cs")]
impl<MODE> Xt1Config<MODE> {
    /// Disable the automatic clock fallback on XT1 fault.
    ///
    /// When disabled, the system will not automatically switch to a fallback 
    /// oscillator (like REFO) if the XT1 crystal fails or stops.
    pub fn disable_fault_switch(mut self) -> Self {
        self.fault_switch = false;
        self
    }
}

impl Xt1Config<CrystalMode> {
    /// Configure XT1 as a crystal oscillator.
    ///
    /// This mode expects a crystal connected to XIN/XOUT and enables
    /// internal oscillator circuitry.
    ///
    /// - `frequency`: Target crystal frequency in Hz.
    /// - `_xin`, `_xout`: Pins connected to the crystal.
    ///
    /// The start counter is enabled by default in crystal mode because
    /// crystals require a stabilization period before producing a valid clock.
    /// The counter ensures the oscillator is given sufficient startup time
    /// before being considered stable.
    pub fn crystal<XIN, XOUT>(frequency: u32, _xin: XIN, _xout: XOUT) -> Self
    where
        XIN: Xt1XinPin,
        XOUT: Xt1XoutPin,
    {
        // Enforce crystal frequency limits (32kHz typical, up to 24MHz on some MCUs)
        Self {
            frequency,
            drive: Xt1drive::Xt1drive3,
            agc: false,
            start_counter: true,
            bypass: false,
            #[cfg(feature = "enhanced_cs")]
            fault_switch: true,
            auto_off: true,
            _mode: PhantomData,
        }
    }

    /// Enable Automatic Gain Control (AGC).
    ///
    /// AGC is only applicable in crystal mode and helps stabilize oscillation
    /// amplitude across varying conditions.
    pub fn enable_agc(mut self) -> Self {
        self.agc = true;
        self
    }

    /// Set oscillator drive strength.
    ///
    /// Higher drive strength may be required for higher-frequency crystals
    /// or specific load conditions.
    pub fn with_drive(mut self, drive: Xt1drive) -> Self {
        self.drive = drive;
        self
    }

    /// Disable the startup counter.
    ///
    /// This skips the oscillator stabilization wait period. Only disable this
    /// if startup timing is externally managed or guaranteed, as doing so may
    /// result in using an unstable clock.
    pub fn disable_start_counter(mut self) -> Self {
        self.start_counter = false;
        self
    }

    /// Disable automatic crystal power-down.
    ///
    /// Setting this to false ensures XT1 remains active even if it is not 
    /// currently requested by a system clock (ACLK, MCLK, SMCLK) or the FLL.
    pub fn disable_auto_off(mut self) -> Self {
        self.auto_off = false;
        self
    }
}

impl Xt1Config<BypassMode> {
    /// Configure XT1 in bypass mode using an external clock source.
    ///
    /// In this mode, a digital clock signal is fed directly into XIN and the
    /// internal crystal oscillator circuitry is bypassed.
    ///
    /// - `frequency`: Input clock frequency in Hz.
    /// - `_xin`: Pin receiving the external clock.
    ///
    /// The start counter is disabled by default because an external clock
    /// source is assumed to already be stable and does not require oscillator
    /// startup time.
    pub fn bypass<XIN>(frequency: u32, _xin: XIN) -> Self
    where
        XIN: Xt1XinPin,
    {
        // Frequency limits depend on MCU family:
        // - Some devices: max ~1MHz
        // - Others (e.g. FR2355): up to 24MHz
        Self {
            frequency,
            drive: Xt1drive::Xt1drive0, // Ignored in bypass mode
            agc: false,                 // Not applicable in bypass mode
            start_counter: false,       // External clock assumed stable
            bypass: true,
            #[cfg(feature = "enhanced_cs")]
            fault_switch: true,
            auto_off: true,
            _mode: PhantomData,
        }
    }

    /// Enable the startup counter.
    ///
    /// This is typically unnecessary in bypass mode, but may be useful if the
    /// external clock source has a delayed or uncertain startup behavior and
    /// additional stabilization time is required.
    pub fn enable_start_counter(mut self) -> Self {
        self.start_counter = true;
        self
    }
}

#[doc(hidden)]
pub trait Xt1State {
    fn freq(&self) -> u32;
}

impl<MODE> Xt1State for Xt1Defined<MODE> {
    #[inline(always)]
    fn freq(&self) -> u32 {
        self.0.frequency
    }
}

impl Xt1State for Xt1Disabled {
    #[inline(always)]
    // Unreachanble freq() only called when self.fll_ref is Xt1clk and that is only possble when Xt1 is defined
    fn freq(&self) -> u32 {
        unreachable!() 
    }
}


// Using SmclkState as a trait bound outside the HAL will never be useful, since we only configure
// the clock once, so just keep it hidden
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

/// Builder object that configures system clocks
///
/// Can only commit configurations to hardware if both MCLK and SMCLK settings have been
/// configured. ACLK configurations are optional, with its default source being REFOCLK.
pub struct ClockConfig<MCLK, SMCLK, XT1CLK> {
    periph: _pac::Cs,
    mclk: MCLK,
    mclk_div: MclkDiv,
    aclk_sel: AclkSel,
    smclk: SMCLK,
    xt1clk: XT1CLK,
    fll_ref: Selref
}

macro_rules! make_clkconf {
    ($conf:expr, $mclk:expr, $smclk:expr, $xt1clk:expr, $fll_ref: expr) => {
        ClockConfig {
            periph: $conf.periph,
            mclk: $mclk,
            mclk_div: $conf.mclk_div,
            aclk_sel: $conf.aclk_sel,
            smclk: $smclk,
            xt1clk: $xt1clk,
            fll_ref: $fll_ref,
        }
    };
}

impl ClockConfig<NoClockDefined, NoClockDefined, Xt1Disabled> {
    /// Converts CS into a fresh, unconfigured clock builder object
    pub fn new(cs: _pac::Cs) -> Self {
        ClockConfig {
            periph: cs,
            smclk: NoClockDefined,
            mclk: NoClockDefined,
            xt1clk: Xt1Disabled,
            mclk_div: MclkDiv::_1,
            aclk_sel: AclkSel::Refoclk,
            fll_ref: Selref::Refoclk,
        }
    }
}

impl<MCLK, SMCLK, XT1CLK> ClockConfig<MCLK, SMCLK, XT1CLK> {
    /// Select REFOCLK for ACLK
    #[inline]
    pub fn aclk_refoclk(mut self) -> Self {
        self.aclk_sel = AclkSel::Refoclk;
        self
    }

    #[cfg(feature = "vloclk_source")]
    /// Select VLOCLK for ACLK
    #[inline]
    pub fn aclk_vloclk(mut self) -> Self {
        self.aclk_sel = AclkSel::Vloclk;
        self
    }

    /// Select REFOCLK for MCLK and set the MCLK divider. Frequency is `32_768 / mclk_div` Hz.
    #[inline]
    pub fn mclk_refoclk(self, mclk_div: MclkDiv) -> ClockConfig<MclkDefined, SMCLK, XT1CLK> {
        ClockConfig {
            mclk_div,
            ..make_clkconf!(self, MclkDefined(MclkSel::Refoclk), self.smclk, self.xt1clk, self.fll_ref)
        }
    }

    /// Select VLOCLK for MCLK and set the MCLK divider. Frequency is `10_000 / mclk_div` Hz.
    #[inline]
    pub fn mclk_vloclk(self, mclk_div: MclkDiv) -> ClockConfig<MclkDefined, SMCLK, XT1CLK> {
        ClockConfig {
            mclk_div,
            ..make_clkconf!(self, MclkDefined(MclkSel::Vloclk), self.smclk, self.xt1clk, self.fll_ref)
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
    ) -> ClockConfig<MclkDefined, SMCLK, XT1CLK> {
        ClockConfig {
            mclk_div,
            ..make_clkconf!(self, MclkDefined(MclkSel::Dcoclk(target_freq)), self.smclk, self.xt1clk, self.fll_ref)
        }
    }

    /// Enable SMCLK and set SMCLK divider, which divides the MCLK frequency
    #[inline]
    pub fn smclk_on(self, div: SmclkDiv) -> ClockConfig<MCLK, SmclkDefined, XT1CLK> {
        make_clkconf!(self, self.mclk, SmclkDefined(div), self.xt1clk, self.fll_ref)
    }

    /// Disable SMCLK
    #[inline]
    pub fn smclk_off(self) -> ClockConfig<MCLK, SmclkDisabled, XT1CLK> {
        make_clkconf!(self, self.mclk, SmclkDisabled, self.xt1clk, self.fll_ref)
    }

    /// Enable XT1 with specific hardware requirements
    #[inline]
    pub fn xt1clk_on<MODE>(
        self, 
        config: Xt1Config<MODE>
    ) -> ClockConfig<MCLK, SMCLK, Xt1Defined<MODE>> {
        make_clkconf!(self, self.mclk, self.smclk, Xt1Defined(config), self.fll_ref)
    }

    /// Explicitly disable XT1 to save power
    #[inline]
    pub fn xt1clk_off(self) -> ClockConfig<MCLK, SMCLK, Xt1Disabled> {
        make_clkconf!(self, self.mclk, self.smclk, Xt1Disabled, Selref::Refoclk)
    }
}

impl<MCLK, SMCLK, MODE> ClockConfig<MCLK, SMCLK, Xt1Defined<MODE>> {
    /// Select XT1CLK for ACLK
    #[inline]
    pub fn aclk_xt1clk(mut self) -> Self {
        self.aclk_sel = AclkSel::Xt1clk(self.xt1clk.freq());
        self
    }

    /// Select XT1CLK for MCLK
    #[inline]
    pub fn mclk_xt1clk(
        self,
        mclk_div: MclkDiv,
    ) -> ClockConfig<MclkDefined, SMCLK, Xt1Defined<MODE>> {
        let freq = self.xt1clk.freq();
        ClockConfig {
            mclk_div,
            ..make_clkconf!(self, MclkDefined(MclkSel::Xt1clk(freq)), self.smclk, self.xt1clk, self.fll_ref)
        }
    }

    /// Select XT1CLK for fll
    #[inline]
    pub fn fll_ref_xt1(mut self) -> Self  {
        self.fll_ref = Selref::Xt1clk;
        self
    }
}

#[inline(always)]
fn fll_off() {
    // 64 = 1 << 6, which is the 6th bit of SR
    unsafe { asm!("bis.b #64, SR", options(nomem, nostack)) };
}

#[inline(always)]
fn fll_on() {
    // 64 = 1 << 6, which is the 6th bit of SR
    unsafe { asm!("bic.b #64, SR", options(nomem, nostack)) };
}

impl<SMCLK: SmclkState, XT1CLK: Xt1State> ClockConfig<MclkDefined, SMCLK, XT1CLK> {
    #[inline]
    fn configure_dco_fll(&self) {
        // Run FLL configuration procedure from the user's guide if we are using DCO
        if let MclkSel::Dcoclk(target_freq) = self.mclk.0 {
            // These thresholds are calculated as the "handover" points between 
            // hardware dividers. Each cutoff (1.5MHz, 3MHz, etc.) ensures the 
            // resulting FLL reference stays within the stable ~23kHz to ~47kHz range.
            let (ref_freq, ref_div) = match self.fll_ref {
                #[cfg(not(any(feature = "msp430fr2433")))]
                Selref::Xt1clk => {
                    let freq = self.xt1clk.freq();
                    if freq <= 40_000 { 
                        (freq, Fllrefdiv::_1) 
                    } else if freq <= 1_500_000 { 
                        // Divider handover: at 1.5MHz, /32 is 46.8kHz, /64 is 23.4kHz.
                        (freq / 32, Fllrefdiv::_32) 
                    } else if freq <= 3_000_000 { 
                        // Divider handover: at 3.0MHz, /64 is 46.8kHz, /128 is 23.4kHz.
                        (freq / 64, Fllrefdiv::_64) 
                    } else if freq <= 6_000_000 { 
                        // Divider handover: at 6.0MHz, /128 is 46.8kHz, /256 is 23.4kHz.
                        (freq / 128, Fllrefdiv::_128) 
                    } else if freq <= 12_000_000 { 
                        // Divider handover: at 12.0MHz, /256 is 46.8kHz, /512 is 23.4kHz.
                        (freq / 256, Fllrefdiv::_256) 
                    } else if freq <= 18_000_000 { 
                        // Max standard divider is /512. At 18MHz, ref is 35.1kHz.
                        (freq / 512, Fllrefdiv::_512) 
                    } else {
                        // TODO: make Fllrefdiv same across pacs
                        #[cfg(feature = "enhanced_cs")]
                        {
                            if freq <= 22_000_000 {
                                // Handover to /640 for 24MHz capable systems.
                                (freq / 640, Fllrefdiv::Fllrefdiv6)
                            } else {
                                // Handover to /768 to keep 24MHz crystal at 31.25kHz ref.
                                (freq / 768, Fllrefdiv::Fllrefdiv7)
                            }
                        }
                        #[cfg(not(feature = "enhanced_cs"))]
                        {
                            (freq / 512, Fllrefdiv::_512)
                        }
                    }
                }
                #[cfg(not(any(feature = "msp430fr2433")))]
                _ => (REFOCLK_FREQ_HZ as u32, Fllrefdiv::_1),
                #[cfg(any(feature = "msp430fr2433"))]
                Selref::Xt1clk => {
                    let freq = self.xt1clk.freq();
                    if freq <= 40_000 {
                        // Standard 32.768kHz crystal
                        (freq, Fllrefdiv::Fllrefdiv0)
                    } else if freq <= 80_000 {
                        // 80kHz / 2 = 40kHz
                        (freq / 2, Fllrefdiv::Fllrefdiv1)
                    } else if freq <= 160_000 {
                        // 160kHz / 4 = 40kHz
                        (freq / 4, Fllrefdiv::Fllrefdiv2)
                    } else if freq <= 320_000 {
                        // 320kHz / 8 = 40kHz
                        (freq / 8, Fllrefdiv::Fllrefdiv3)
                    } else if freq <= 480_000 {
                        // 480kHz / 12 = 40kHz
                        (freq / 12, Fllrefdiv::Fllrefdiv4)
                    } else {
                        // Max divider is /16. 
                        // If user provides a 1MHz signal, ref = 62.5kHz (Risky/Out of spec!)
                        (freq / 16, Fllrefdiv::Fllrefdiv5)
                    }
                }
                #[cfg(any(feature = "msp430fr2433"))]
                _ => (REFOCLK_FREQ_HZ as u32, Fllrefdiv::Fllrefdiv0),
            };

            fll_off();

            self.periph.csctl3()
                .write(|w| w.selref().variant(self.fll_ref).fllrefdiv().variant(ref_div));
            self.periph.csctl0().write(|w| unsafe { w.bits(0) });
            self.periph
                .csctl1()
                .write(|w| w.dcorsel().variant(target_freq.dcorsel()));

            // Use the calculated ref_freq to get a precise multiplier
            let multiplier = (target_freq.freq() / ref_freq) as u16;

            self.periph.csctl2().write(|w| {
                unsafe { w.flln().bits(multiplier - 1) }
                    .flld()
                    ._1()
            });

            msp430::asm::nop();
            msp430::asm::nop();
            msp430::asm::nop();

            fll_on();

            while !self.periph.csctl7().read().fllunlock().is_fllunlock_0() {}
        }
    }

    #[inline]
    fn configure_cs(&self) {
        // Configure clock selector and divisors
        self.periph.csctl4().write(|w| {
            w.sela()
                .variant(self.aclk_sel.sela())
                .selms()
                .variant(self.mclk.0.selms())
        });

        self.periph.csctl5().write(|w| {
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

impl<SMCLK: SmclkState, MCLK, MODE> ClockConfig<MCLK, SMCLK, Xt1Defined<MODE>> {
    #[inline]
    fn configure_xt1(&self) {
        let cfg = &self.xt1clk.0;
        let sfr = unsafe { &*_pac::Sfr::ptr() };
        
        self.periph.csctl6().modify(|_, w| {
            #[cfg(feature = "xt1_high_frequency")]
            {
                let (xts, hf_range) = if cfg.frequency <= 40_000 {
                    (Xts::Xts0, Xt1hffreq::Xt1hffreq0)
                } else if cfg.frequency <= 4_000_000 {
                    (Xts::Xts1, Xt1hffreq::Xt1hffreq0)
                } else if cfg.frequency <= 6_000_000 {
                    (Xts::Xts1, Xt1hffreq::Xt1hffreq1)
                } else if cfg.frequency <= 16_000_000 {
                    (Xts::Xts1, Xt1hffreq::Xt1hffreq2)
                } else {
                    (Xts::Xts1, Xt1hffreq::Xt1hffreq3)
                };
                
                w.xt1bypass().bit(cfg.bypass)
                 .xt1faultoff().bit(!cfg.fault_switch)
                 .xts().variant(xts)
                 .xt1agcoff().bit(!cfg.agc)
                 .xt1autooff().bit(cfg.auto_off)
                 .xt1hffreq().variant(hf_range)
            }

            #[cfg(not(feature = "xt1_high_frequency"))]
            {        
                w.xt1bypass().bit(cfg.bypass)
                 .xts().clear_bit()
                 .xt1agcoff().bit(!cfg.agc)
                 .xt1autooff().bit(cfg.auto_off)
            }
        });

        self.periph.csctl7().modify(|_, w| w.enstfcnt1().bit(cfg.start_counter));

        loop {
            unsafe {
                self.periph.csctl7().clear_bits(|w| 
                    w.xt1offg().clear_bit()
                      .dcoffg().clear_bit()
                );
                sfr.sfrifg1().clear_bits(|w| w.ofifg().clear_bit());
            }

            // Poll global fault flag
            if sfr.sfrifg1().read().ofifg().bit_is_clear() {
                break;
            }
        }

        self.periph.csctl6().modify(|_, w| { w.xt1drive().variant(cfg.drive) });
    }
}

impl<MODE> ClockConfig<MclkDefined, SmclkDefined, Xt1Defined<MODE>> {
    /// Apply clock configuration to hardware and return SMCLK and ACLK clock objects.
    /// Also returns delay provider
    #[inline]
    pub fn freeze(self, fram: &mut Fram) -> (Smclk, Aclk, Xt1clk, SysDelay) {
        let mclk_freq = self.mclk.0.freq() >> (self.mclk_div as u32);
        unsafe { Self::configure_fram(fram, mclk_freq) };
        self.configure_xt1();
        self.configure_dco_fll();
        self.configure_cs();
        (
            Smclk(mclk_freq >> (self.smclk.0 as u32)),
            Aclk(self.aclk_sel.freq()),
            Xt1clk(self.xt1clk.freq()),
            SysDelay::new(mclk_freq),
        )
    }
}

impl ClockConfig<MclkDefined, SmclkDefined, Xt1Disabled> {
    /// Apply clock configuration to hardware and return SMCLK and ACLK clock objects.
    /// Also returns delay provider
    #[inline]
    pub fn freeze(self, fram: &mut Fram) -> (Smclk, Aclk, SysDelay) {
        let mclk_freq = self.mclk.0.freq() >> (self.mclk_div as u32);
        unsafe { Self::configure_fram(fram, mclk_freq) };
        self.configure_dco_fll();
        self.configure_cs();
        (
            Smclk(mclk_freq >> (self.smclk.0 as u32)),
            Aclk(self.aclk_sel.freq()),
            SysDelay::new(mclk_freq),
        )
    }
}

impl<MODE> ClockConfig<MclkDefined, SmclkDisabled, Xt1Defined<MODE>> {
    /// Apply clock configuration to hardware and return ACLK clock object, as SMCLK is disabled.
    /// Also returns delay provider.
    #[inline]
    pub fn freeze(self, fram: &mut Fram) -> (Aclk, Xt1clk, SysDelay) {
        let mclk_freq = self.mclk.0.freq() >> (self.mclk_div as u32);
        unsafe { Self::configure_fram(fram, mclk_freq) };
        self.configure_xt1();
        self.configure_dco_fll();
        self.configure_cs();
        (
            Aclk(self.aclk_sel.freq()), 
            Xt1clk(self.xt1clk.freq()),
            SysDelay::new(mclk_freq)
        )
    }
}

impl ClockConfig<MclkDefined, SmclkDisabled, Xt1Disabled> {
    /// Apply clock configuration to hardware and return ACLK clock object, as SMCLK is disabled.
    /// Also returns delay provider.
    #[inline]
    pub fn freeze(self, fram: &mut Fram) -> (Aclk, SysDelay) {
        let mclk_freq = self.mclk.0.freq() >> (self.mclk_div as u32);
        unsafe { Self::configure_fram(fram, mclk_freq) };
        self.configure_dco_fll();
        self.configure_cs();
        (Aclk(self.aclk_sel.freq()), SysDelay::new(mclk_freq))
    }
}

/// SMCLK clock object
pub struct Smclk(u32);
/// ACLK clock object
pub struct Aclk(u32);
/// XT1CLK clock object
pub struct Xt1clk(u32);

/// Trait for configured clock objects
pub trait Clock {
    /// Returning a 32-bit frequency may seem suspect, since we're on a 16-bit system, but it is
    /// required as SMCLK can go up to 24 MHz. Clock frequencies are usually for initialization
    /// tasks such as computing baud rates, which should be optimized away, avoiding the extra cost
    /// of 32-bit computations.
    fn freq(&self) -> u32;
}

impl Clock for Smclk {
    #[inline]
    fn freq(&self) -> u32 {
        self.0
    }
}

impl Clock for Aclk {
    #[inline]
    fn freq(&self) -> u32 {
        self.0
    }
}

impl Clock for Xt1clk {
    #[inline]
    fn freq(&self) -> u32 {
        self.0
    }
}