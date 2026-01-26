pub use crate::hw_traits::timer_base::*;

// TimerB unique features not yet implemented, just forward the base impl
pub(crate) use timer_base_impl as timer_b_impl;
