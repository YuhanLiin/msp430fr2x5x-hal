pub trait Steal {
    unsafe fn steal() -> Self;
}

pub mod eusci;
pub mod gpio;
pub mod timerb;
