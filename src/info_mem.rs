//! Information Memory.
//! 512 bytes of non-volatile memory.
//!
//! Access the information memory by calling one of the `InfoMemory::as_x()` methods,
//! which disables write protection and directly provides a reference to the information memory as an array.
//!

use crate::pac::SYS;

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
            .dfwp().dfwp_0()
        );
    }

    as_x!(as_u8s,   [u8;  512]);
    as_x!(as_u16s,  [u16; 256]);
    as_x!(as_u32s,  [u32; 128]);
    as_x!(as_u64s,  [u64;  64]);
    as_x!(as_u128s, [u128; 32]);

    as_x!(as_i8s,   [i8;  512]);
    as_x!(as_i16s,  [i16; 256]);
    as_x!(as_i32s,  [i32; 128]);
    as_x!(as_i64s,  [i64;  64]);
    as_x!(as_i128s, [i128; 32]);
}

impl System {
    /// Access the SYS register.
    /// Note: If the DFWP bit is re-enabled the information memory will not be writable.
    #[inline(always)]
    pub fn with(&mut self, f: impl FnOnce(&mut SYS)) {
        f(&mut self.0);
    }
}
