#![no_main]
#![no_std]

// This example demonstrates hardware control of the CS pin, and shows how to use the SpiDevice interface as a result.

use embedded_hal::spi::Operation;
use embedded_hal::spi::SpiDevice;
use embedded_hal::spi::MODE_0;
use embedded_hal::delay::DelayNs;
use msp430_rt::entry;
use msp430fr2x5x_hal::{
    clock::{ClockConfig, DcoclkFreqSel, MclkDiv, SmclkDiv}, fram::Fram, gpio::Batch, pmm::Pmm, spi::SpiConfig, watchdog::Wdt
};
use panic_msp430 as _;

#[entry]
fn main() -> ! {
    let periph = msp430fr2355::Peripherals::take().unwrap();

    let mut fram = Fram::new(periph.FRCTL);
    let _wdt = Wdt::constrain(periph.WDT_A);

    let pmm = Pmm::new(periph.PMM);
    let p1 = Batch::new(periph.P1)
        .split(&pmm);
    let miso = p1.pin7.to_alternate1();
    let mosi = p1.pin6.to_alternate1();
    let sck  = p1.pin5.to_alternate1();
    let cs   = p1.pin4.to_alternate1();

    let (smclk, _aclk, mut delay) = ClockConfig::new(periph.CS)
        .mclk_dcoclk(DcoclkFreqSel::_8MHz, MclkDiv::_1)
        .smclk_on(SmclkDiv::_1)
        .aclk_vloclk()
        .freeze(&mut fram);

    // If we have only one slave we can have the hardware control the chip select pin for us.
    // If you need multiple slaves then use  `.configure_with_software_cs()` and manage the CS pins yourself.
    let mut spi_dev = SpiConfig::new(periph.E_USCI_A0, MODE_0, true)
        .use_smclk(&smclk, 16) // 8MHz / 16 = 500kHz
        .configure_with_hardware_cs(miso, mosi, sck, cs, delay);

    loop {
        let mut byte_0 = [0u8];
        let mut bytes_2_3 = [0xF0, 0x0F];

        // As the hardware is managing the CS pin for us, we can use the SpiDevice interface:
        spi_dev.transaction(&mut [
            Operation::Read(&mut byte_0),               // Write dummy packet to MOSI, read MISO
            Operation::Write(&[0b10101010]),            // Write to MOSI, discard MISO
            Operation::TransferInPlace(&mut bytes_2_3), // Write 0xF0 and 0x0F to MOSI, read MISO
        ]).unwrap();    // You should handle errors

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
