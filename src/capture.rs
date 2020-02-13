//! Capture ports
//!
//! Signal capture pins are created from timers. TB0, TB1, and TB2 create 3 pins each and TB3
//! create 7 pins. Each capture pin has its own configurable input signal, edge
//! trigger, and capture storage. Each timer has a configurable timer counting from 0 to `2^16-1`,
//! whose value will be stored in a capture register whenever its capture event is triggered.
//! Capture pins can choose between two inputs sources for triggering events. These inputs can come
//! from input GPIOs or internal chip signals. When using a GPIO input, the user must configure the
//! GPIO pin correctly according to the datasheet for the capture event to work properly.

use crate::hw_traits::timerb::{
    CCRn, Ccis, Cm, TimerB, TimerSteal, CCR0, CCR1, CCR2, CCR3, CCR4, CCR5, CCR6,
};
use crate::timer::{read_tbxiv, SevenCCRnTimer, ThreeCCRnTimer, TimerVector};
use crate::util::SealedDefault;
use core::marker::PhantomData;
use msp430fr2355 as pac;

pub use crate::timer::{CapCmpPeriph, TimerConfig, TimerDiv, TimerExDiv, TimerPeriph};

mod sealed {
    use super::*;

    pub trait SealedCaptureExt {}

    impl SealedCaptureExt for pac::TB0 {}
    impl SealedCaptureExt for pac::TB1 {}
    impl SealedCaptureExt for pac::TB2 {}
    impl SealedCaptureExt for pac::TB3 {}
}

type Tb0 = pac::tb0::RegisterBlock;
type Tb1 = pac::tb1::RegisterBlock;
type Tb2 = pac::tb2::RegisterBlock;
type Tb3 = pac::tb3::RegisterBlock;

/// Capture input selection
pub enum CapSelect {
    /// Capture input A
    CapInputA,
    /// Capture input B
    CapInputB,
}

impl Into<Ccis> for CapSelect {
    #[inline]
    fn into(self) -> Ccis {
        match self {
            CapSelect::CapInputA => Ccis::InputA,
            CapSelect::CapInputB => Ccis::InputB,
        }
    }
}

/// Capture edge trigger
pub enum CapTrigger {
    /// Capture on rising edge
    RisingEdge,
    /// Capture on falling edge
    FallingEdge,
    /// Capture on both edges
    BothEdges,
}

impl Into<Cm> for CapTrigger {
    #[inline]
    fn into(self) -> Cm {
        match self {
            CapTrigger::RisingEdge => Cm::RisingEdge,
            CapTrigger::FallingEdge => Cm::FallingEdge,
            CapTrigger::BothEdges => Cm::BothEdges,
        }
    }
}

struct PinConfig {
    select: CapSelect,
    trigger: CapTrigger,
}

impl PinConfig {
    fn new() -> Self {
        Self {
            select: CapSelect::CapInputA,
            trigger: CapTrigger::RisingEdge,
        }
    }
}

/// Configuration object for all capture pins of a single port
pub struct CaptureConfig {
    cap0: PinConfig,
    cap1: PinConfig,
    cap2: PinConfig,
    cap3: PinConfig,
    cap4: PinConfig,
    cap5: PinConfig,
    cap6: PinConfig,
}

macro_rules! config_fn {
    ($config_capture:ident, $cap:ident) => {
        /// Configure the input select and edge trigger for one of the capture pins
        #[inline(always)]
        pub fn $config_capture(self, select: CapSelect, trigger: CapTrigger) -> Self {
            Self {
                $cap: PinConfig { select, trigger },
                ..self
            }
        }
    }
}

impl CaptureConfig {
    config_fn!(config_capture0, cap0);
    config_fn!(config_capture1, cap1);
    config_fn!(config_capture2, cap2);
    config_fn!(config_capture3, cap3);
    config_fn!(config_capture4, cap4);
    config_fn!(config_capture5, cap5);
    config_fn!(config_capture6, cap6);

    /// Create new capture config. All pins default to input A and rising edge trigger.
    pub fn new() -> Self {
        Self {
            cap0: PinConfig::new(),
            cap1: PinConfig::new(),
            cap2: PinConfig::new(),
            cap3: PinConfig::new(),
            cap4: PinConfig::new(),
            cap5: PinConfig::new(),
            cap6: PinConfig::new(),
        }
    }
}

// Implemented for RegisterBlocks, which the user will never name when using the HAL, so we can
// keep this trait hidden.
#[doc(hidden)]
pub trait CaptureConfigChannels {
    fn config_channels(&self, config: CaptureConfig);
}

impl CaptureConfigChannels for Tb0 {
    #[inline]
    fn config_channels(&self, config: CaptureConfig) {
        CCRn::<CCR0>::config_cap_mode(self, config.cap0.trigger.into(), config.cap0.select.into());
        CCRn::<CCR1>::config_cap_mode(self, config.cap1.trigger.into(), config.cap1.select.into());
        CCRn::<CCR2>::config_cap_mode(self, config.cap2.trigger.into(), config.cap2.select.into());
    }
}

impl CaptureConfigChannels for Tb1 {
    #[inline]
    fn config_channels(&self, config: CaptureConfig) {
        CCRn::<CCR0>::config_cap_mode(self, config.cap0.trigger.into(), config.cap0.select.into());
        CCRn::<CCR1>::config_cap_mode(self, config.cap1.trigger.into(), config.cap1.select.into());
        CCRn::<CCR2>::config_cap_mode(self, config.cap2.trigger.into(), config.cap2.select.into());
    }
}

impl CaptureConfigChannels for Tb2 {
    #[inline]
    fn config_channels(&self, config: CaptureConfig) {
        CCRn::<CCR0>::config_cap_mode(self, config.cap0.trigger.into(), config.cap0.select.into());
        CCRn::<CCR1>::config_cap_mode(self, config.cap1.trigger.into(), config.cap1.select.into());
        CCRn::<CCR2>::config_cap_mode(self, config.cap2.trigger.into(), config.cap2.select.into());
    }
}

impl CaptureConfigChannels for Tb3 {
    #[inline]
    fn config_channels(&self, config: CaptureConfig) {
        CCRn::<CCR0>::config_cap_mode(self, config.cap0.trigger.into(), config.cap0.select.into());
        CCRn::<CCR1>::config_cap_mode(self, config.cap1.trigger.into(), config.cap1.select.into());
        CCRn::<CCR2>::config_cap_mode(self, config.cap2.trigger.into(), config.cap2.select.into());
        CCRn::<CCR3>::config_cap_mode(self, config.cap3.trigger.into(), config.cap3.select.into());
        CCRn::<CCR4>::config_cap_mode(self, config.cap4.trigger.into(), config.cap4.select.into());
        CCRn::<CCR5>::config_cap_mode(self, config.cap5.trigger.into(), config.cap5.select.into());
        CCRn::<CCR6>::config_cap_mode(self, config.cap6.trigger.into(), config.cap6.select.into());
    }
}

/// Extension trait for creating capture pins from timer peripherals
pub trait CaptureExt: Sized + sealed::SealedCaptureExt {
    /// Timer peripheral's `RegisterBlock`
    type Capture: TimerPeriph + CaptureConfigChannels;
    /// Set of capture pins
    type Pins: SealedDefault;

    /// Create new capture port out of timer
    #[inline]
    fn to_capture(
        self,
        timer_config: TimerConfig<Self::Capture>,
        cap_config: CaptureConfig,
    ) -> Self::Pins {
        let timer = unsafe { Self::Capture::steal() };
        timer_config.write_regs(timer);
        timer.config_channels(cap_config);
        timer.continuous();
        Self::Pins::default()
    }
}

impl CaptureExt for pac::TB0 {
    type Capture = Tb0;
    type Pins = ThreeCCRnPins<Self::Capture>;
}
impl CaptureExt for pac::TB1 {
    type Capture = Tb1;
    type Pins = ThreeCCRnPins<Self::Capture>;
}
impl CaptureExt for pac::TB2 {
    type Capture = Tb2;
    type Pins = ThreeCCRnPins<Self::Capture>;
}
impl CaptureExt for pac::TB3 {
    type Capture = Tb3;
    type Pins = SevenCCRnPins<Self::Capture>;
}

/// Collection of uninitialized capture pins derived from timer peripheral with 3 capture-compare registers
pub struct ThreeCCRnPins<T: ThreeCCRnTimer> {
    /// Capture pin 0 (derived from capture-compare register 0)
    pub cap0: Capture<T, CCR0>,
    /// Capture pin 1 (derived from capture-compare register 1)
    pub cap1: Capture<T, CCR1>,
    /// Capture pin 2 (derived from capture-compare register 2)
    pub cap2: Capture<T, CCR2>,
    /// Interrupt vector register
    pub tbxiv: TBxIV<T>,
}

impl<T: ThreeCCRnTimer> SealedDefault for ThreeCCRnPins<T> {
    #[inline(always)]
    fn default() -> Self {
        Self {
            cap0: SealedDefault::default(),
            cap1: SealedDefault::default(),
            cap2: SealedDefault::default(),
            tbxiv: TBxIV(PhantomData),
        }
    }
}

/// Collection of uninitialized capture pins derived from timer peripheral with 7 capture-compare registers
pub struct SevenCCRnPins<T: SevenCCRnTimer> {
    /// Capture pin 0 (derived from capture-compare register 0)
    pub cap0: Capture<T, CCR0>,
    /// Capture pin 1 (derived from capture-compare register 1)
    pub cap1: Capture<T, CCR1>,
    /// Capture pin 2 (derived from capture-compare register 2)
    pub cap2: Capture<T, CCR2>,
    /// Capture pin 3 (derived from capture-compare register 3)
    pub cap3: Capture<T, CCR3>,
    /// Capture pin 4 (derived from capture-compare register 4)
    pub cap4: Capture<T, CCR4>,
    /// Capture pin 5 (derived from capture-compare register 5)
    pub cap5: Capture<T, CCR5>,
    /// Capture pin 6 (derived from capture-compare register 6)
    pub cap6: Capture<T, CCR6>,
    /// Interrupt vector register
    pub tbxiv: TBxIV<T>,
}

impl<T: SevenCCRnTimer> SealedDefault for SevenCCRnPins<T> {
    #[inline(always)]
    fn default() -> Self {
        Self {
            cap0: SealedDefault::default(),
            cap1: SealedDefault::default(),
            cap2: SealedDefault::default(),
            cap3: SealedDefault::default(),
            cap4: SealedDefault::default(),
            cap5: SealedDefault::default(),
            cap6: SealedDefault::default(),
            tbxiv: TBxIV(PhantomData),
        }
    }
}

/// Single capture pin with its own capture register
pub struct Capture<T: CCRn<C>, C>(PhantomData<T>, PhantomData<C>);

impl<T: CCRn<C>, C> SealedDefault for Capture<T, C> {
    fn default() -> Self {
        Self(PhantomData, PhantomData)
    }
}

// Candidate for embedded_hal inclusion
/// Single input capture pin
pub trait CapturePin {
    /// Type  of value returned by capture
    type Capture;
    /// Enumeration of `Capture` errors
    ///
    /// Possible errors:
    ///
    /// - *overcapture*, the previous capture value was overwritten because it
    ///   was not read in a timely manner// Enumeration of `Capture` errors
    ///
    /// Possible errors:
    ///
    /// - *overcapture*, the previous capture value was overwritten because it
    ///   was not read in a timely manner
    type Error;

    /// "Waits" for a transition in the capture `channel` and returns the value
    /// of counter at that instant
    fn capture(&mut self) -> nb::Result<Self::Capture, Self::Error>;
}

impl<T: CCRn<C>, C> CapturePin for Capture<T, C> {
    type Capture = u16;
    type Error = OverCapture;

    #[inline]
    fn capture(&mut self) -> nb::Result<Self::Capture, Self::Error> {
        let timer = unsafe { T::steal() };
        let (cov, ccifg) = timer.cov_ccifg_rd();
        if ccifg {
            let ccrn = timer.get_ccrn();
            timer.cov_ccifg_clr();
            if cov {
                Err(nb::Error::Other(OverCapture(ccrn)))
            } else {
                Ok(ccrn)
            }
        } else {
            Err(nb::Error::WouldBlock)
        }
    }
}

impl<T: CCRn<C>, C> Capture<T, C> {
    #[inline]
    /// Enable capture interrupts
    pub fn enable_interrupts(&mut self) {
        let timer = unsafe { T::steal() };
        timer.ccie_set();
    }

    #[inline]
    /// Disable capture interrupts
    pub fn disable_interrupt(&mut self) {
        let timer = unsafe { T::steal() };
        timer.ccie_clr();
    }
}

/// Error returned when the previous capture was overwritten before being read
pub struct OverCapture(pub u16);

/// Capture TBIV interrupt vector
pub enum CaptureVector<T> {
    /// No pending interrupt
    NoInterrupt,
    /// Interrupt caused by capture register 1.
    Capture1(InterruptCapture<T, CCR1>),
    /// Interrupt caused by capture register 2.
    Capture2(InterruptCapture<T, CCR2>),
    /// Interrupt caused by capture register 3.
    Capture3(InterruptCapture<T, CCR3>),
    /// Interrupt caused by capture register 4.
    Capture4(InterruptCapture<T, CCR4>),
    /// Interrupt caused by capture register 5.
    Capture5(InterruptCapture<T, CCR5>),
    /// Interrupt caused by capture register 6.
    Capture6(InterruptCapture<T, CCR6>),
    /// Interrupt caused by main timer overflow
    MainTimer,
}

/// Token returned when reading the interrupt vector that allows a one-time read of the capture
/// register corresponding to the interrupt.
pub struct InterruptCapture<T, C>(PhantomData<T>, PhantomData<C>);

impl<T: CCRn<C>, C> InterruptCapture<T, C> {
    /// Performs a one-time capture read without considering the interrupt flag. Always call this
    /// instead of `capture()` after reading the capture interrupt vector, since reading the vector
    /// already clears the interrupt flag that `capture()` checks for.
    #[inline]
    pub fn interrupt_capture(self, _cap: &mut Capture<T, C>) -> Result<u16, OverCapture> {
        let timer = unsafe { T::steal() };
        let (cov, _) = timer.cov_ccifg_rd();
        let ccrn = timer.get_ccrn();
        if cov {
            timer.cov_ccifg_clr();
            Err(OverCapture(ccrn))
        } else {
            Ok(ccrn)
        }
    }
}

/// Interrupt vector register for determining which capture-register caused an ISR
pub struct TBxIV<T: TimerPeriph>(PhantomData<T>);

impl<T: TimerPeriph> TBxIV<T> {
    #[inline]
    /// Read the capture interrupt vector. Automatically resets corresponding interrupt flag. If
    /// the vector corresponds to an available capture, a one-time capture read token will be
    /// returned as well.
    pub fn interrupt_vector(&mut self) -> CaptureVector<T> {
        let timer = unsafe { T::steal() };
        match read_tbxiv(timer) {
            TimerVector::NoInterrupt => CaptureVector::NoInterrupt,
            TimerVector::SubTimer1 => {
                CaptureVector::Capture1(InterruptCapture(PhantomData, PhantomData))
            }
            TimerVector::SubTimer2 => {
                CaptureVector::Capture2(InterruptCapture(PhantomData, PhantomData))
            }
            TimerVector::SubTimer3 => {
                CaptureVector::Capture3(InterruptCapture(PhantomData, PhantomData))
            }
            TimerVector::SubTimer4 => {
                CaptureVector::Capture4(InterruptCapture(PhantomData, PhantomData))
            }
            TimerVector::SubTimer5 => {
                CaptureVector::Capture5(InterruptCapture(PhantomData, PhantomData))
            }
            TimerVector::SubTimer6 => {
                CaptureVector::Capture6(InterruptCapture(PhantomData, PhantomData))
            }
            TimerVector::MainTimer => CaptureVector::MainTimer,
        }
    }
}
