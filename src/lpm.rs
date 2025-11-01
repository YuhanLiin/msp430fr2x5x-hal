//! Low Power Mode (LPM) control
//!
//! The MSP430FR2x5x series supports several low power modes, namely LPM0, LPM3, LPM4, as well as LPM3.5 and LPM4.5.
//! # LPM0
//! LPM0 turns off the CPU, while the rest of the system continues unimpeded. Entering LPM0 has no special requirements.
//!
//! # LPM3
//! LPM3 turns off most high frequency clocks (FLL and DCO subsystems, MODCLK, etc.), most notably SMCLK.
//! Since most peripherals can be clocked by a low frequency clock source, this allows many peripherals to continue operating
//! at a reduced speed.
//!
//! LPM3 will only be entered if no peripherals have been configured to use SMCLK, otherwise LPM0 will be entered instead.
//!
//! GPIO pins will maintain the value they had when LPM3 was entered.
//!
//! # LPM4
//! LPM4 turns off all clock sources. The RTC or Watchdog peripherals can request very low power oscillators (VLOCLK or XTCLK)
//! if needed, though this will increase power consumption and is not considered 'true' LPM4. Some analog peripherals
//! (eCOMP, SAC) continue to function in LPM4. Wake-up events are limited to GPIO or RTC interrupts.
//!
//! LPM4 will only be entered if no peripherals have been configured to use SMCLK or ACLK. If any peripherals have requested SMCLK
//! then LPM0 will be entered, otherwise if any peripherals have requested ACLK then LPM3 will be entered.
//!
//! GPIO pins will maintain the value they had when LPM4 was entered.
//!
//! # LPM3.5
//! LPM3.5 is an extension of LPM4 that also disables the RAM and analog peripherals. Unlike in LPM4, the very low power
//! oscillators (VLOCLK or XTCLK) are expected to be used in LPM3.5.
//! Because the RAM is unpowered, all internal state is lost and when the MCU wakes up execution will restart from the beginning
//! of the program. The 32-byte region of Backup Memory is powered through LPM3.5, which can be used to maintain some state
//! between iterations.
//!
//! Unlike with LPM3 and 4, a peripheral requesting a high-speed clock source like SMCLK will not stop LPM3.5 from being entered.
//!
//! During LPM3.5 GPIO pins will maintain the value they had when LPM3.5 was entered but the register contents are lost, so after a wake-up
//! the GPIO pins will take on their reset values when LOCKLPM5 is cleared.
//!
//! # LPM4.5
//! LPM4.5 is an extension of LPM3.5 that also disables the very low power oscillators, backup memory, and RTC. Barely anything
//! is powered in this mode. The only methods to wake from LPM4.5 are a GPIO interrupt, the reset pin, or a power cycle.
//! Like LPM3.5, all internal state is lost and program execution restarts from the beginning when a wake-up occurs. Unlike LPM3.5,
//! the backup memory is unpowered, so can't be used to store state. The 512-byte section of non-volatile Information Memory can however
//! be used to store data while the MCU is in active mode, and can be read back after a wake-up event.
//!
//! Unlike with LPM3 and 4, a peripheral requesting a clock source like SMCLK or ACLK will not stop LPM4.5 from being entered.
//!
//! During LPM4.5 GPIO pins will maintain the value they had when LPM3.5 was entered but the register contents are lost, so after a wake-up
//! the GPIO pins will take on their reset values when LOCKLPM5 is cleared.

use core::arch::asm;
use crate::pac::{Peripherals, RTC};

use crate::{
    rtc::{Rtc, RtcVloclk},
    watchdog::{WatchdogSelect, Wdt},
};

pub use crate::pac::pmm::pmmctl0::SVSHE_A as SvsState;

// Status register:
// SCG1 SCG0 OSC_OFF CPU_OFF GIE N Z C
// 7    6    5       4       3   2 1 0
const SCG1:    u8 = 1 << 7;
const SCG0:    u8 = 1 << 6;
const OSC_OFF: u8 = 1 << 5;
const CPU_OFF: u8 = 1 << 4;
const GIE:     u8 = 1 << 3;

/// For each set bit in the bitmask, set the corresponding bit in the status register.
#[inline(always)]
fn set_sr_bits<const MASK: u8>() {
    unsafe { asm!("bis.b #{mask}, SR", mask = const MASK, options(nomem, nostack)) };
}

/// Enter Low Power Mode 0 (LPM0).
///
/// In LPM0 the CPU and MCLK are disabled.
///
/// Power draw in LPM0: Approx 40 uA / MHz.
#[inline(always)]
pub fn enter_lpm0() {
    const LPM0: u8 = CPU_OFF;
    set_sr_bits::<LPM0>();
}

/// Request Low Power Mode 3 (LPM3).
///
/// LPM3 can only be reached if no peripherals have been configured to use SMCLK.
/// If any peripherals are configured to use SMCLK then LPM0 will be entered instead.
///
/// In LPM3 the CPU, FLL, and all clocks (except ACLK) are disabled.
///
/// Power draw in LPM3: Approx 1.4 uA.
#[inline(always)]
pub fn request_lpm3() {
    const LPM3: u8 = SCG1 | SCG0 | CPU_OFF;
    set_sr_bits::<LPM3>();
}

/// Request Low Power Mode 4 (LPM4).
///
/// LPM4 can only be reached if no peripherals have been configured to use SMCLK or ACLK.
///
/// If any peripherals use SMCLK then LPM0 will be entered.
/// If no peripherals use SMCLK but at least one uses ACLK then LPM3 will be entered.
///
/// In LPM4 the CPU, FLL, and all clocks (except optionally the very low power oscillators VLOCLK or XTCLK) are disabled.
///
/// Power draw in LPM4: Approx 820 nA.
#[inline(always)]
pub fn request_lpm4() {
    const LPM4: u8 = SCG1 | SCG0 | OSC_OFF | CPU_OFF;
    set_sr_bits::<LPM4>();
}

/// Enter Low Power Mode 3.5 (LPM3.5).
///
/// In LPM3.5 everything except the backup memory, VLOCLK, and the RTC are disabled. The only enabled interrupts are from the RTC, I/O pins, the RST pin, or a power cycle.
///
/// I/O pins have their state latched while in LPM3.5, but the IO register values are reset on wake-up.
///
/// **Waking up from LPM3.5 requires a full system reset**.
///
/// Power draw in LPM3.5: Approx 620 nA.
#[inline(always)]
pub fn enter_lpm3_5<MODE: WatchdogSelect>(wdt: Wdt<MODE>, _rtc: Rtc<RtcVloclk>, svs: SvsState) -> ! {
    lpm3_5(wdt, svs);
}

/// Enter LPM3.5 without providing a correctly configured RTC (because it has already been configured in a prior iteration).
/// # Safety
/// If the RTC was not correctly configured previously then the system will not enter LPM3.5.
#[inline(always)]
pub unsafe fn enter_lpm3_5_unchecked<MODE: WatchdogSelect>(wdt: Wdt<MODE>, svs: SvsState) -> ! {
    lpm3_5(wdt, svs)
}

fn lpm3_5<MODE: WatchdogSelect>(wdt: Wdt<MODE>, svs: SvsState) -> ! {
    // Take peripherals. Execution won't return from this fn.
    let regs = unsafe { crate::pac::Peripherals::conjure() };

    // If LF XT crystal is not in use, reset everything, otherwise reset everything but XIN, XOUT
    const MASK: u8 = (1 << 6) | (1 << 7);
    let lfxt_in_use =
        (regs.P2.p2sel1.read().bits() & MASK == MASK) && (regs.P2.p2sel0.read().bits() & MASK == 0);
    if lfxt_in_use {
        // Reset everything except for XIN and XOUT
        unsafe {
            regs.P2.p2sel1.clear_bits(|w| w.bits(MASK));
            regs.P2.p2sel0.clear_bits(|w| w.bits(MASK));
        }
    } else {
        // Reset everything
        regs.P2.p2sel0.reset();
        regs.P2.p2sel0.reset();
    }

    enter_lpmx_5(wdt, svs, regs)
}

/// Enter Low Power Mode 4.5 (LPM4.5).
///
/// In LPM4.5 *everything* is disabled. The only available interrupt sources are from I/O pins, the RST pin, or a power cycle.
///
/// I/O pins have their state latched while in LPM4.5, but the register values are reset on wake-up.
///
/// **Waking up from LPM4.5 requires a full system reset**.
///
/// Power draw in LPM4.5: Approx 42 nA.
#[inline]
pub fn enter_lpm4_5<MODE: WatchdogSelect>(wdt: Wdt<MODE>, rtc_reg: RTC, svs: SvsState) -> ! {
    // Disable RTC
    unsafe { rtc_reg.rtcctl.clear_bits(|w| w.rtcss().disabled()) };

    // Take peripherals. Execution won't return from this fn.
    let regs = unsafe { crate::pac::Peripherals::conjure() };

    // Reset P2SEL, including XIN and XOUT
    regs.P2.p2sel0.reset();
    regs.P2.p2sel1.reset();

    enter_lpmx_5(wdt, svs, regs)
}

/// Configuration common to LPM3.5 and 4.5
fn enter_lpmx_5<MODE: WatchdogSelect>(mut wdt: Wdt<MODE>, svs: SvsState, regs: Peripherals) -> ! {
    // Pause WDT
    wdt.pause();

    // Reset PxSEL
    regs.P1.p1sel0.reset();
    regs.P1.p1sel1.reset();
    /* P2 reset by 4.5 and 3.5 fns */
    regs.P3.p3sel0.reset();
    regs.P3.p3sel1.reset();
    regs.P4.p4sel0.reset();
    regs.P4.p4sel1.reset();
    regs.P5.p5sel0.reset();
    regs.P5.p5sel1.reset();
    regs.P6.p6sel0.reset();
    regs.P6.p6sel1.reset();

    let interrupts_were_enabled = msp430::register::sr::read().gie();
    msp430::interrupt::disable();

    // Write PMM password to get PMM control regs
    // Set PMMREGOFF
    const PASSWORD: u8 = 0xA5;
    regs.PMM.pmmctl0.write(|w| w
        .pmmpw().variant(PASSWORD)
        .svshe().variant(svs)
        .pmmregoff().pmmregoff_1()
    );

    // Write incorrect password to PMM to lock
    // Only write to the upper byte of PMMCTL0
    let pmmctl0_h = (regs.PMM.pmmctl0.as_ptr() as *mut u8).wrapping_add(1);
    unsafe{ pmmctl0_h.write_volatile(0); }

    if interrupts_were_enabled {
        const LPMX_5: u8 = SCG1 | SCG0 | OSC_OFF | CPU_OFF | GIE;
        set_sr_bits::<LPMX_5>();
    } else {
        const LPMX_5: u8 = SCG1 | SCG0 | OSC_OFF | CPU_OFF;
        set_sr_bits::<LPMX_5>();
    }

    // LPMx.5 achieved.

    // This part won't actually run, but just to appease compiler about '!'
    #[allow(clippy::empty_loop)]
    loop {}
}
