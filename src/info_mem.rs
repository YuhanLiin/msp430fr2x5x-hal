//! Information Memory.
//! 512 bytes of non-volatile memory.
//!
//! Access the information memory by calling one of the `InfoMemory::as_x()` methods,
//! which disables write protection and directly provides a reference to the information memory as an array.
//!

use crate::_pac::SYS;
use crate::device_specific::INFO_MEM_SIZE;

/// A struct that manages writing and reading from information memory.
pub struct InfoMemory(());

/// A wrapper for the SYS register block. Allows access to the SYS registers, but prevents calling InfoMemory::as_x() multiple times.
pub struct System(SYS);

/// Start address of the information memory
const INFO_MEM_START_ADDR: *mut u8 = 0x1800 as *mut u8;
const SYSCFG0_PASSWORD: u8 = 0xA5;

macro_rules! as_x {
    ($fn_name: ident, $arr: ty) => {
        #[doc = "Disable the write protection bit, interpret the information memory as a `&mut"] #[doc = stringify!($arr)] 
        #[doc = "` and return a mutable reference to it."]
        #[inline(always)]
        pub fn $fn_name(mut sys: SYS) -> (&'static mut $arr, System) {
            const { assert!( core::mem::size_of::<$arr>() == INFO_MEM_SIZE ) }
            Self::disable_write_protect(&mut sys);
            let arr = unsafe { &mut *(INFO_MEM_START_ADDR as *mut $arr) };
            (arr, System(sys))
        }
    };
}

impl InfoMemory {
    #[inline(always)]
    fn disable_write_protect(sys: &mut SYS) {
        sys.syscfg0.write(|w| w
            .frwppw().variant(SYSCFG0_PASSWORD)
            .dfwp().clear_bit()
        );
    }

    as_x!(as_u8s,   [u8;   INFO_MEM_SIZE/size_of::<u8>()]);
    as_x!(as_u16s,  [u16;  INFO_MEM_SIZE/size_of::<u16>()]);
    as_x!(as_u32s,  [u32;  INFO_MEM_SIZE/size_of::<u32>()]);
    as_x!(as_u64s,  [u64;  INFO_MEM_SIZE/size_of::<u64>()]);
    as_x!(as_u128s, [u128; INFO_MEM_SIZE/size_of::<u128>()]);

    as_x!(as_i8s,   [i8;   INFO_MEM_SIZE/size_of::<i8>()]);
    as_x!(as_i16s,  [i16;  INFO_MEM_SIZE/size_of::<i16>()]);
    as_x!(as_i32s,  [i32;  INFO_MEM_SIZE/size_of::<i32>()]);
    as_x!(as_i64s,  [i64;  INFO_MEM_SIZE/size_of::<i64>()]);
    as_x!(as_i128s, [i128; INFO_MEM_SIZE/size_of::<i128>()]);
}

impl System {
    /// Access the SYS register.
    /// Note: If the DFWP bit is re-enabled the information memory will not be writable.
    #[inline(always)]
    pub fn with(&mut self, f: impl FnOnce(&mut SYS)) {
        f(&mut self.0);
    }
}
