/// Trait representing a Smart Analog Combo (SAC) peripheral.
pub trait SacPeriph {
    /// Non-inverting opamp input pin
    type PosInputPin;
    /// Inverting opamp input pin
    type NegInputPin;
    /// Opamp output pin
    type OutputPin;
    fn configure_sacoa(psel: u8, nsel: NSel, pm: bool);
    fn configure_sacpga(gain: u8, mode: MSel);
    fn configure_dac(load_condition: u8, vref: bool);
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
    ($SAC: ident, 
        $pos_port: ident, $pos_pin: ident, // Positive input
        $neg_port: ident, $neg_pin: ident, // Negative input
        $out_port: ident, $out_pin: ident, // Output 
        $sacXoa: ident, $sacXpga: ident, $sacXdac: ident, $sacXdat: ident) => {
        impl Steal for $SAC {
            #[inline(always)]
            unsafe fn steal() -> Self {
                crate::pac::Peripherals::conjure().$SAC
            }
        }
        impl SacPeriph for $SAC {
            type PosInputPin = Pin<$pos_port, $pos_pin, Alternate3<Input<Floating>>>;
            type NegInputPin = Pin<$neg_port, $neg_pin, Alternate3<Input<Floating>>>;
            type OutputPin   = Pin<$out_port, $out_pin, Alternate3<Input<Floating>>>;
            #[inline(always)]
            fn configure_sacoa(psel: u8, nsel: NSel, pm: bool) {
                unsafe {
                    let sac = $SAC::steal();
                    sac.$sacXoa.write(|w| w
                        .nsel().bits(nsel as u8)
                        .psel().bits(psel)
                        .oapm().bit(pm)
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
            fn configure_dac(lsel: u8, vref: bool) {
                unsafe{
                    let sac = $SAC::steal();
                    sac.$sacXdac.write(|w| w
                        .dacsref().bit(vref)
                        .daclsel().bits(lsel)
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
pub(crate) use impl_sac_periph;
