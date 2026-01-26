pub use crate::hw_traits::timer_base::*;

// TimerA unique features not yet implemented, just forward the base impl
#[allow(unused_imports)] // TODO: Remove when TimerA is actually used by an implementation
pub(crate) use timer_base_impl as timer_a_impl;
