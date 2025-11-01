//! Capture ports
//!
//! Configures the board's TimerB peripherals into capture pins. Each capture pin has a 16-bit
//! capture register where its timer value is written whenever its capture event is triggered.
//!
//! Due to hardware constraints, the configurations for all capture pins derived from a timer must
//! be decided before any of them can be used. This differs from `Pwm`, where pins are initialized
//! on an individual basis.

use crate::gpio::{
    Alternate1, Alternate2, Floating, Input, Pin, Pin0, Pin1, Pin2, Pin3, Pin4, Pin5, Pin6, Pin7,
    P1, P2, P5, P6,
};
use crate::hw_traits::timerb::{CCRn, Ccis, Cm};
use crate::timer::{read_tbxiv, CapCmpTimer3, CapCmpTimer7, TimerVector};
use core::marker::PhantomData;
use crate::pac;

pub use crate::timer::{
    CapCmp, TimerConfig, TimerDiv, TimerExDiv, TimerPeriph, CCR0, CCR1, CCR2, CCR3, CCR4, CCR5,
    CCR6,
};

/// Capture edge trigger
pub enum CapTrigger {
    /// Capture on rising edge
    RisingEdge,
    /// Capture on falling edge
    FallingEdge,
    /// Capture on both edges
    BothEdges,
}

impl From<CapTrigger> for Cm {
    #[inline]
    fn from(val: CapTrigger) -> Self {
        match val {
            CapTrigger::RisingEdge => Cm::RisingEdge,
            CapTrigger::FallingEdge => Cm::FallingEdge,
            CapTrigger::BothEdges => Cm::BothEdges,
        }
    }
}

struct PinConfig {
    select: Ccis,
    trigger: CapTrigger,
}

impl Default for PinConfig {
    fn default() -> Self {
        Self {
            select: Ccis::Gnd,
            trigger: CapTrigger::RisingEdge,
        }
    }
}

/// Extension trait for creating capture pins from timer peripherals
pub trait CapturePeriph: TimerPeriph {
    /// GPIO pin that supplies input A for capture pin 1
    type Gpio1;
    /// GPIO pin that supplies input A for capture pin 2
    type Gpio2;
    /// GPIO pin that supplies input A for capture pin 3
    type Gpio3;
    /// GPIO pin that supplies input A for capture pin 4
    type Gpio4;
    /// GPIO pin that supplies input A for capture pin 5
    type Gpio5;
    /// GPIO pin that supplies input A for capture pin 6
    type Gpio6;
}
impl CapturePeriph for pac::TB0 {
    type Gpio1 = Pin<P1, Pin6, Alternate2<Input<Floating>>>;
    type Gpio2 = Pin<P1, Pin7, Alternate2<Input<Floating>>>;
    type Gpio3 = ();
    type Gpio4 = ();
    type Gpio5 = ();
    type Gpio6 = ();
}

impl CapturePeriph for pac::TB1 {
    type Gpio1 = Pin<P2, Pin0, Alternate1<Input<Floating>>>;
    type Gpio2 = Pin<P2, Pin1, Alternate1<Input<Floating>>>;
    type Gpio3 = ();
    type Gpio4 = ();
    type Gpio5 = ();
    type Gpio6 = ();
}

impl CapturePeriph for pac::TB2 {
    type Gpio1 = Pin<P5, Pin0, Alternate1<Input<Floating>>>;
    type Gpio2 = Pin<P5, Pin1, Alternate1<Input<Floating>>>;
    type Gpio3 = ();
    type Gpio4 = ();
    type Gpio5 = ();
    type Gpio6 = ();
}

impl CapturePeriph for pac::TB3 {
    type Gpio1 = Pin<P6, Pin0, Alternate1<Input<Floating>>>;
    type Gpio2 = Pin<P6, Pin1, Alternate1<Input<Floating>>>;
    type Gpio3 = Pin<P6, Pin2, Alternate1<Input<Floating>>>;
    type Gpio4 = Pin<P6, Pin3, Alternate1<Input<Floating>>>;
    type Gpio5 = Pin<P6, Pin4, Alternate1<Input<Floating>>>;
    type Gpio6 = Pin<P6, Pin5, Alternate1<Input<Floating>>>;
}

macro_rules! config_fn {
    (methods $config_sel_b:ident, $config_trigger:ident, $pin:ident) => {
        #[allow(non_snake_case)]
        #[inline(always)]
        /// Configure the capture input select of the capture pin as capture input B
        pub fn $config_sel_b(mut self) -> Self {
            self.$pin.select = Ccis::InputB;
            self
        }

        #[inline(always)]
        /// Configure the capture trigger event of the capture pin
        pub fn $config_trigger(mut self, trigger: CapTrigger) -> Self {
            self.$pin.trigger = trigger;
            self
        }
    };

    ($config_sel_a:ident, $config_sel_b:ident, $config_trigger:ident, $pin:ident, $gpio:ident) => {
        #[allow(non_snake_case)]
        #[inline(always)]
        /// Configure the capture input select of the capture pin as capture input A, which
        /// requires a correctly configured GPIO pin.
        pub fn $config_sel_a(mut self, _gpio: T::$gpio) -> Self {
            self.$pin.select = Ccis::InputA;
            self
        }
        config_fn!(methods $config_sel_b, $config_trigger, $pin);
    };

    ($config_sel_a:ident, $config_sel_b:ident, $config_trigger:ident, $pin:ident) => {
        #[allow(non_snake_case)]
        #[inline(always)]
        /// Configure the capture input select of the capture pin as capture input A
        pub fn $config_sel_a(mut self) -> Self {
            self.$pin.select = Ccis::InputA;
            self
        }
        config_fn!(methods $config_sel_b, $config_trigger, $pin);
    };
}

/// Builder object for configuring capture ports derived from timer peripherals with 3
/// capture-compare registers
///
/// Each pin has a input source, which determines the signal that controls the capture, and a
/// capture trigger event, which determines the input transitions that actually trigger the
/// capture. By default, all pins use GND as their input source and trigger a capture on a rising
/// edge.
pub struct CaptureConfig3<T: CapturePeriph>
where
    T: CapCmpTimer3,
{
    timer: T,
    config: TimerConfig<T>,
    cap0: PinConfig,
    cap1: PinConfig,
    cap2: PinConfig,
}

impl<T: CapturePeriph + CapCmpTimer3> CaptureParts3<T> {
    /// Create capture configuration
    pub fn config(timer: T, config: TimerConfig<T>) -> CaptureConfig3<T> {
        CaptureConfig3 {
            timer,
            config,
            cap0: PinConfig::default(),
            cap1: PinConfig::default(),
            cap2: PinConfig::default(),
        }
    }
}

impl<T: CapturePeriph + CapCmpTimer3> CaptureConfig3<T> {
    config_fn!(
        config_cap0_input_A,
        config_cap0_input_B,
        config_cap0_trigger,
        cap0
    );
    config_fn!(
        config_cap1_input_A,
        config_cap1_input_B,
        config_cap1_trigger,
        cap1,
        Gpio1
    );
    config_fn!(
        config_cap2_input_A,
        config_cap2_input_B,
        config_cap2_trigger,
        cap2,
        Gpio2
    );

    /// Writes all previously configured timer and capture settings into peripheral registers
    pub fn commit(self) -> CaptureParts3<T> {
        let timer = self.timer;
        self.config.write_regs(&timer);
        CCRn::<CCR0>::config_cap_mode(&timer, self.cap0.trigger.into(), self.cap0.select);
        CCRn::<CCR1>::config_cap_mode(&timer, self.cap1.trigger.into(), self.cap1.select);
        CCRn::<CCR2>::config_cap_mode(&timer, self.cap2.trigger.into(), self.cap2.select);
        timer.continuous();

        CaptureParts3 {
            cap0: Capture::new(),
            cap1: Capture::new(),
            cap2: Capture::new(),
            tbxiv: TBxIV(PhantomData),
        }
    }
}

/// Builder object for configuring capture ports derived from timer peripherals with 7
/// capture-compare registers
///
/// Each pin has a input source, which determines the signal that controls the capture, and a
/// capture trigger event, which determines the input transitions that actually trigger the
/// capture. By default, all pins use GND as their input source and trigger a capture on a rising
/// edge.
pub struct CaptureConfig7<T: CapturePeriph>
where
    T: CapCmpTimer7,
{
    timer: T,
    config: TimerConfig<T>,
    cap0: PinConfig,
    cap1: PinConfig,
    cap2: PinConfig,
    cap3: PinConfig,
    cap4: PinConfig,
    cap5: PinConfig,
    cap6: PinConfig,
}

impl<T: CapturePeriph + CapCmpTimer7> CaptureParts7<T> {
    /// Create capture configuration
    pub fn config(timer: T, config: TimerConfig<T>) -> CaptureConfig7<T> {
        CaptureConfig7 {
            timer,
            config,
            cap0: PinConfig::default(),
            cap1: PinConfig::default(),
            cap2: PinConfig::default(),
            cap3: PinConfig::default(),
            cap4: PinConfig::default(),
            cap5: PinConfig::default(),
            cap6: PinConfig::default(),
        }
    }
}

impl<T: CapturePeriph + CapCmpTimer7> CaptureConfig7<T> {
    config_fn!(
        config_cap0_input_A,
        config_cap0_input_B,
        config_cap0_trigger,
        cap0
    );
    config_fn!(
        config_cap1_input_A,
        config_cap1_input_B,
        config_cap1_trigger,
        cap1,
        Gpio1
    );
    config_fn!(
        config_cap2_input_A,
        config_cap2_input_B,
        config_cap2_trigger,
        cap2,
        Gpio2
    );
    config_fn!(
        config_cap3_input_A,
        config_cap3_input_B,
        config_cap3_trigger,
        cap3,
        Gpio3
    );
    config_fn!(
        config_cap4_input_A,
        config_cap4_input_B,
        config_cap4_trigger,
        cap4,
        Gpio4
    );
    config_fn!(
        config_cap5_input_A,
        config_cap5_input_B,
        config_cap5_trigger,
        cap5,
        Gpio5
    );
    config_fn!(
        config_cap6_input_A,
        config_cap6_input_B,
        config_cap6_trigger,
        cap6,
        Gpio6
    );

    /// Writes all previously configured timer and capture settings into peripheral registers
    pub fn commit(self) -> CaptureParts7<T> {
        let timer = self.timer;
        self.config.write_regs(&timer);
        CCRn::<CCR0>::config_cap_mode(&timer, self.cap0.trigger.into(), self.cap0.select);
        CCRn::<CCR1>::config_cap_mode(&timer, self.cap1.trigger.into(), self.cap1.select);
        CCRn::<CCR2>::config_cap_mode(&timer, self.cap2.trigger.into(), self.cap2.select);
        CCRn::<CCR3>::config_cap_mode(&timer, self.cap3.trigger.into(), self.cap3.select);
        CCRn::<CCR4>::config_cap_mode(&timer, self.cap4.trigger.into(), self.cap4.select);
        CCRn::<CCR5>::config_cap_mode(&timer, self.cap5.trigger.into(), self.cap5.select);
        CCRn::<CCR6>::config_cap_mode(&timer, self.cap6.trigger.into(), self.cap6.select);
        timer.continuous();

        CaptureParts7 {
            cap0: Capture::new(),
            cap1: Capture::new(),
            cap2: Capture::new(),
            cap3: Capture::new(),
            cap4: Capture::new(),
            cap5: Capture::new(),
            cap6: Capture::new(),
            tbxiv: TBxIV(PhantomData),
        }
    }
}

/// Collection of capture pins derived from timer peripheral with 3 capture-compare registers
pub struct CaptureParts3<T: CapCmpTimer3> {
    /// Capture pin 0 (derived from capture-compare register 0)
    pub cap0: Capture<T, CCR0>,
    /// Capture pin 1 (derived from capture-compare register 1)
    pub cap1: Capture<T, CCR1>,
    /// Capture pin 2 (derived from capture-compare register 2)
    pub cap2: Capture<T, CCR2>,
    /// Interrupt vector register
    pub tbxiv: TBxIV<T>,
}

/// Collection of capture pins derived from timer peripheral with 7 capture-compare registers
pub struct CaptureParts7<T: CapCmpTimer7> {
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

/// Single capture pin with its own capture register
pub struct Capture<T: CapCmp<C>, C>(PhantomData<T>, PhantomData<C>);

impl<T: CapCmp<C>, C> Capture<T, C> {
    fn new() -> Self {
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
    ///   was not read in a timely manner
    type Error;

    /// "Waits" for a transition in the capture `channel` and returns the value
    /// of counter at that instant
    fn capture(&mut self) -> nb::Result<Self::Capture, Self::Error>;
}

impl<T: CapCmp<C>, C> CapturePin for Capture<T, C> {
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

impl<T: CapCmp<C>, C> Capture<T, C> {
    #[inline]
    /// Enable capture interrupts
    pub fn enable_interrupts(&mut self) {
        let timer = unsafe { T::steal() };
        timer.ccie_set();
    }

    #[inline]
    /// Disable capture interrupts
    pub fn disable_interrupts(&mut self) {
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

impl<T: CapCmp<C>, C> InterruptCapture<T, C> {
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
    /// Read the capture interrupt vector and resets corresponding interrupt flag. If
    /// the vector corresponds to an available capture, a one-time capture read token will be
    /// returned as well.
    pub fn interrupt_vector(&mut self) -> CaptureVector<T> {
        let timer = unsafe { T::steal() };
        match read_tbxiv(&timer) {
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
