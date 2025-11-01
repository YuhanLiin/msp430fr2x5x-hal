use crate::{
    gpio::{Pin, Alternate3, Input, Floating, Pin1, Pin2, Pin3, Pin5, Pin6, Pin7}, 
    hw_traits::Steal, sac::{LoadTrigger, VRef, PowerMode},
    pac::{P1, P3, SAC0, SAC1, SAC2, SAC3},
};

/// Trait representing a Smart Analog Combo (SAC) peripheral.
pub trait SacPeriph {
    /// Non-inverting opamp input pin
    type PosInputPin;
    /// Inverting opamp input pin
    type NegInputPin;
    /// Opamp output pin
    type OutputPin;
    fn configure_sacoa(psel: u8, nsel: NSel, pm: PowerMode);
    fn configure_sacpga(gain: u8, mode: MSel);
    fn configure_dac(load_condition: LoadTrigger, vref: VRef);
    fn set_dac_count(val: u16);
}

// Our PositiveInput enum coincides exactly with PSel, so no need for a separate enum

#[derive(Debug, Copy, Clone)]
pub enum NSel {
    ExtPinMinus = 0b00,
    Feedback    = 0b01,
    PairedOpamp = 0b10,
}

#[derive(Debug, Copy, Clone)]
pub enum MSel {
    Inverting    = 0b00,
    Follower     = 0b01,
    NonInverting = 0b10,
    Cascade      = 0b11,
}

macro_rules! impl_sac_periph {
    ($SAC: ident, $port: ident, $inPp: ident, $inNp: ident, $outp: ident, // Register block, port, pos_in, neg_in, out
        $sacXoa: ident, $sacXpga: ident, $sacXdac: ident, $sacXdat: ident) => {
        impl Steal for $SAC {
            #[inline(always)]
            unsafe fn steal() -> Self {
                crate::pac::Peripherals::conjure().$SAC
            }
        }
        impl SacPeriph for $SAC {
            type PosInputPin = Pin<$port, $inPp, Alternate3<Input<Floating>>>;
            type NegInputPin = Pin<$port, $inNp, Alternate3<Input<Floating>>>;
            type OutputPin   = Pin<$port, $outp, Alternate3<Input<Floating>>>;
            #[inline(always)]
            fn configure_sacoa(psel: u8, nsel: NSel, pm: PowerMode) {
                unsafe {
                    let sac = $SAC::steal();
                    sac.$sacXoa.write(|w| w
                        .nsel().bits(nsel as u8)
                        .psel().bits(psel)
                        .oapm().bit(pm.into())
                        .nmuxen().set_bit()
                        .pmuxen().set_bit()
                        .sacen().set_bit()
                        .oaen().set_bit()
                    );
                }
            }
            #[inline(always)]
            fn configure_sacpga(gain: u8, msel: MSel) {
                unsafe{
                    let sac = $SAC::steal();
                    sac.$sacXpga.write(|w| w
                        .gain().bits(gain)
                        .msel().bits(msel as u8)
                    );
                }
            }
            #[inline(always)]
            fn configure_dac(lsel: LoadTrigger, vref: VRef) {
                unsafe{
                    let sac = $SAC::steal();
                    sac.$sacXdac.write(|w| w
                        .dacsref().bit(vref.into())
                        .daclsel().bits(lsel.into())
                        .dacdmae().clear_bit()
                        .dacie().clear_bit()
                        .dacen().set_bit()
                    );
                }
            }
            #[inline(always)]
            fn set_dac_count(val: u16) {
                unsafe{
                    let sac = $SAC::steal();
                    sac.$sacXdat.write(|w| w
                        .dacdata().bits(val)
                    );
                }
            }
        }
    };
}

impl_sac_periph!(
    SAC0, P1, Pin3, Pin2, Pin1, // Register block, port, pos_in, neg_in, out
    sac0oa, sac0pga, sac0dac, sac0dat
);
impl_sac_periph!(
    SAC1, P1, Pin7, Pin6, Pin5,
    sac1oa, sac1pga, sac1dac, sac1dat
);
impl_sac_periph!(
    SAC2, P3, Pin3, Pin2, Pin1,
    sac2oa, sac2pga, sac2dac, sac2dat
);
impl_sac_periph!(
    SAC3, P3, Pin7, Pin6, Pin5,
    sac3oa, sac3pga, sac3dac, sac3dat
);
