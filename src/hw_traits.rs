pub trait Steal {
    unsafe fn steal() -> Self;
}

pub mod ecomp;
pub mod eusci;
pub mod gpio;
pub mod sac;
pub mod timerb;
