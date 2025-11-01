use super::Steal;
use crate::{
    ecomp::{ComparatorDac, BufferSel, DacVRef, FilterStrength, Hysteresis, NegativeInput, OutputPolarity, PositiveInput, PowerMode}, 
    gpio::{Alternate2, Floating, Input, Output, Pin, Pin0, Pin1, Pin4, Pin5}, sac::Amplifier,
    pac::{E_COMP0, E_COMP1, P1, P2, SAC0, SAC1, SAC2, SAC3},
};

#[allow(non_camel_case_types)]
pub trait ECompPeriph: Steal {
    /// COMPx.0 - One of the two GPIO inputs that can be fed into either comparator input.
    ///
    /// For COMP0 this is P1.0, COMP1 has P2.5.
    type COMPx_0;
    /// COMPx.1 - One of the two GPIO inputs that can be fed into either comparator input.
    ///
    /// For COMP0 this is P1.1, COMP1 has P2.4.
    type COMPx_1;
    /// The GPIO pin that can connect to the comparator output.
    ///
    /// For COMP0 this is P2.0, for COMP1 this is P2.1.
    type COMPx_Out;
    /// The Smart Analog Combo peripheral that is accessible from the positive comparator input:
    ///
    /// SAC0 for COMP0, SAC2 for COMP1.
    type SACp;
    /// The Smart Analog Combo peripheral that is accessible from the negative comparator input:
    ///
    /// SAC1 for COMP0, SAC3 for COMP1.
    type SACn;
    fn cpxdacctl(enable: bool, vref: DacVRef, buf_mode: DacBufferMode, buf: BufferSel);
    fn set_buf1_val(buf: u8);
    fn set_buf2_val(buf: u8);
    fn set_dac_buffer_mode(mode: DacBufferMode);
    fn select_buffer(sel: BufferSel);
    fn cpxctl0(pos_in: PositiveInput<Self>, neg_in: NegativeInput<Self>) where Self: core::marker::Sized;
    fn configure_comparator(pol: OutputPolarity, pwr: PowerMode, hstr: Hysteresis, fltr: FilterStrength);
    fn value() -> bool;
    fn en_cpie();
    fn dis_cpie();
    fn en_cpiie();
    fn dis_cpiie();
}

// Marker trait for an eCOMP DAC. Since the DAC has a typestate (hardware/software double buffer)
// we can't just say `type CompDac = ComparatorDac<COMP>`
pub trait CompDacPeriph<COMP: ECompPeriph> {}
impl<COMP: ECompPeriph, MODE> CompDacPeriph<COMP> for ComparatorDac<'_, COMP, MODE> {}

/// Possible eCOMP DAC dual buffer modes
pub enum DacBufferMode {
    /// In hardware mode the DAC count is determined by either CPDACBUF1 or CPDACBUF2,
    /// based on the output value of the comparator
    Hardware,
    /// In software mode the DAC count can be switched between CPDACBUF1 and CPDACBUF2 by software
    Software,
}
impl From<DacBufferMode> for bool {
    fn from(value: DacBufferMode) -> Self {
        match value {
            DacBufferMode::Hardware => false,
            DacBufferMode::Software => true,
        }
    }
}

macro_rules! impl_ecomp {
    ($COMP: ident,
        $inPP: ident, $inPp: ident,
        $inNP: ident, $inNp: ident,
        $outP: ident, $outp: ident,
        $SACp: ty, $SACn: ty,
        $cpctl0: ident, $cpctl1: ident,
        $cpdacctl: ident, $cpdacdata: ident,
        $cpint: ident, $cpiv: ident ) => {
        impl Steal for $COMP {
            #[inline(always)]
            unsafe fn steal() -> Self {
                crate::pac::Peripherals::conjure().$COMP
            }
        }
        impl ECompPeriph for $COMP{
            type COMPx_0   = Pin<$inPP, $inPp, Alternate2<Input<Floating>>>;
            type COMPx_1   = Pin<$inNP, $inNp, Alternate2<Input<Floating>>>;
            type COMPx_Out = Pin<$outP, $outp, Alternate2<Output>>;
            type SACp      = $SACp;
            type SACn      = $SACn;

            #[inline(always)]
            fn cpxdacctl(enable: bool, vref: DacVRef, buf_mode: DacBufferMode, buf: BufferSel) {
                unsafe {
                    let comp = $COMP::steal();
                    comp.$cpdacctl.modify(|_, w| w
                        .cpdacen().bit(enable)
                        .cpdacrefs().bit(vref.into())
                        .cpdacbufs().bit(buf_mode.into())
                        .cpdacsw().bit(buf.into())
                    );
                }
            }
            #[inline(always)]
            fn set_buf1_val(buf: u8) {
                unsafe {
                    let comp = $COMP::steal();
                    comp.$cpdacdata.modify(|_, w| w.cpdacbuf1().bits(buf));
                }
            }
            #[inline(always)]
            fn set_buf2_val(buf: u8) {
                unsafe {
                    let comp = $COMP::steal();
                    comp.$cpdacdata.modify(|_, w| w.cpdacbuf2().bits(buf));
                }
            }
            #[inline(always)]
            fn select_buffer(sel: BufferSel) {
                unsafe {
                    let comp = $COMP::steal();
                    comp.$cpdacctl.modify(|_, w| w.cpdacsw().bit(sel.into()));
                }
            }
            #[inline(always)]
            fn set_dac_buffer_mode(mode: DacBufferMode) {
                unsafe {
                    let comp = $COMP::steal();
                    comp.$cpdacctl.modify(|_, w| w.cpdacbufs().bit(mode.into()));
                }
            }
            #[inline(always)]
            fn cpxctl0(pos_in: PositiveInput<$COMP>, neg_in: NegativeInput<$COMP>) {
                unsafe {
                    let comp = $COMP::steal();
                    comp.$cpctl0.modify(|_, w| w
                        .cppen().set_bit()
                        .cppsel().bits(pos_in.into())
                        .cpnen().set_bit()
                        .cpnsel().bits(neg_in.into())
                    );
                }
            }
            #[inline(always)]
            fn configure_comparator(pol: OutputPolarity, pwr: PowerMode, hstr: Hysteresis, fltr: FilterStrength) {
                unsafe {
                    let comp = $COMP::steal();
                    comp.$cpctl1.modify(|_, w| w
                        .cphsel().bits(hstr as u8)
                        .cpen().set_bit()
                        .cpmsel().bit(pwr.into())
                        .cpflt().bit(fltr != FilterStrength::Off)
                        // Note: fltr could be 3 bits, will be truncated to 00 but only if filter is off anyway
                        .cpfltdly().bits(fltr as u8) 
                        .cpinv().bit(pol.into())
                    );
                }
            }
            #[inline(always)]
            fn value() -> bool {
                let comp = unsafe { $COMP::steal() };
                comp.$cpctl1.read().cpout().bit()
            }
            #[inline(always)]
            fn en_cpie() {
                unsafe {
                    let comp = { $COMP::steal() };
                    comp.$cpctl1.set_bits(|w| w.cpie().set_bit())
                }
            }
            #[inline(always)]
            fn dis_cpie() {
                unsafe {
                    let comp = { $COMP::steal() };
                    comp.$cpctl1.clear_bits(|w| w.cpie().clear_bit())
                }
            }
            #[inline(always)]
            fn en_cpiie() {
                unsafe {
                    let comp = { $COMP::steal() };
                    comp.$cpctl1.set_bits(|w| w.cpiie().set_bit())
                }
            }
            #[inline(always)]
            fn dis_cpiie() {
                unsafe {
                    let comp = { $COMP::steal() };
                    comp.$cpctl1.clear_bits(|w| w.cpiie().clear_bit())
                }
            }
        }
    };
}

impl_ecomp!(
    E_COMP0,  // eCOMP module
    P1, Pin0, // Positive input pin
    P1, Pin1, // Negative input pin
    P2, Pin0, // Output pin
    Amplifier<SAC0>, Amplifier<SAC2>, // Paired SAC units (pos in, neg in)
    cpctl0, cpctl1,
    cpdacctl, cpdacdata,
    cpint, cpiv
);

impl_ecomp!(
    E_COMP1,
    P2, Pin5,
    P2, Pin4,
    P2, Pin1,
    Amplifier<SAC1>, Amplifier<SAC3>,
    cp1ctl0, cp1ctl1,
    cp1dacctl, cp1dacdata,
    cp1int, cp1iv
);
