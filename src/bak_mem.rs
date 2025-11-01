//! Backup Memory.
//! 32 bytes of memory that survive system resets.
//!
//! This memory is still volatile however, so it won't survive power loss. The backup memory is powered in all modes except LPM4.5.
//!
//! The peripheral access crate exposes the backup memory as 16 individual 16-bit registers.
//! This module provides helper functions for reinterpreting the backup memory as various array types.
//!
//! After choosing the most convenient data type for your application call the relevant method,
//! such as [`BackupMemory::as_u8s()`], to recieve a mutable reference to the backup memory.

use crate::pac::BKMEM;

/// Helper struct with static methods for interpreting the backup memory into more usable forms
pub struct BackupMemory;

macro_rules! as_x {
    ($fn_name: ident, $arr: ty) => {
        #[doc = "Interpret the backup memory as a `&mut"] #[doc = stringify!($arr)] #[doc = "`"]
        #[inline(always)]
        pub fn $fn_name(_reg: BKMEM) -> &'static mut $arr {
            unsafe { &mut *(BKMEM::PTR as *mut $arr) }
        }
    };
}

impl BackupMemory {
    as_x!(as_u8s,   [u8; 32]);
    as_x!(as_u16s,  [u16;16]);
    as_x!(as_u32s,  [u32; 8]);
    as_x!(as_u64s,  [u64; 4]);
    as_x!(as_u128s, [u128;2]);

    as_x!(as_i8s,   [i8; 32]);
    as_x!(as_i16s,  [i16;16]);
    as_x!(as_i32s,  [i32; 8]);
    as_x!(as_i64s,  [i64; 4]);
    as_x!(as_i128s, [i128;2]);
}
