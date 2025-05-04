#![no_main]
#![no_std]

use embedded_hal::{i2c::I2c, delay::DelayNs};
use msp430_rt::entry;
use msp430fr2x5x_hal::{
    clock::{ClockConfig, DcoclkFreqSel, MclkDiv, SmclkDiv}, fram::Fram, gpio::Batch, i2c::{GlitchFilter, I2cConfig}, pmm::Pmm, watchdog::Wdt
};
use panic_msp430 as _;

#[entry]
fn main() -> ! {
    let periph = msp430fr2355::Peripherals::take().unwrap();

    let mut fram = Fram::new(periph.FRCTL);
    let _wdt = Wdt::constrain(periph.WDT_A);

    let pmm = Pmm::new(periph.PMM);
    let p4 = Batch::new(periph.P4)
        .split(&pmm);
    let scl = p4.pin7.to_alternate1();
    let sda = p4.pin6.to_alternate1();

    let (smclk, _aclk, mut delay) = ClockConfig::new(periph.CS)
        .mclk_dcoclk(DcoclkFreqSel::_8MHz, MclkDiv::_1)
        .smclk_on(SmclkDiv::_1)
        .aclk_vloclk()
        .freeze(&mut fram);

    let mut i2c = I2cConfig::new(periph.E_USCI_B1, GlitchFilter::Max50ns)
        .use_smclk(&smclk, 80) // 8MHz / 10 = 100kHz
        .configure(scl, sda);

    loop {
        // Blocking read. Read 10 bytes (length of buffer) from address 0x12.
        // Pass a u8 address for 7-bit addressing mode, pass a u16 for 10-bit addressing mode.
        let mut buf = [0; 10];
        // You should handle errors here rather than unwrapping
        i2c.read(0x12_u8, &mut buf).unwrap();

        // Blocking write. Write one byte to address 0x12.
        // You should handle errors here rather than unwrapping
        i2c.write(0x12_u8, &[0b10101010]).unwrap();

        // Blocking send + recieve. Write 10 bytes to 0x12, then read 20 bytes
        let send_buf = [0b11001100; 10];
        let mut recv_buf = [0; 20];
        // You should handle errors here rather than unwrapping
        i2c.write_read(0x12_u8, &send_buf, &mut recv_buf).unwrap();

        delay.delay_ms(1000);
    }
}

// The compiler will emit calls to the abort() compiler intrinsic if debug assertions are
// enabled (default for dev profile). MSP430 does not actually have meaningful abort() support
// so for now, we create our own in each application where debug assertions are present.
#[no_mangle]
extern "C" fn abort() -> ! {
    panic!();
}
