#![no_main]
#![no_std]

use embedded_hal::spi::FullDuplex;
use embedded_hal::blocking::spi::Transfer;
use embedded_hal::spi::MODE_0;
use embedded_hal::blocking::delay::DelayMs;
use msp430_rt::entry;
use msp430fr2x5x_hal::{
    clock::{ClockConfig, DcoclkFreqSel, MclkDiv, SmclkDiv}, fram::Fram, gpio::Batch, pmm::Pmm, spi::SpiBusConfig, watchdog::Wdt
};
use nb::block;
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

    let mut spi = SpiBusConfig::new(periph.E_USCI_A0, MODE_0, true)
        .use_smclk(&smclk, 16) // 8MHz / 16 = 500kHz
        .configure_with_hardware_cs(miso, mosi, sck, cs);

    loop {
        // Non-blocking send. Sending is infallible, besides `nb::WouldBlock` when the bus is busy.
        // In this particular case we know the SPI bus isn't busy, so unwrapping is safe here.
        spi.send(0b10101010).unwrap();

        // Wait for the above send to complete. Wrap the non-blocking `.read()` with `block!()` to make it blocking. 
        // Note `.read()` only checks the recieve buffer, it does not send any SPI packets,
        // so if you haven't previously used `.send()` there will be nothing to read.
        // You should handle errors here rather than unwrapping
        let _data = block!(spi.read()).unwrap();

        // Multi-byte blocking send + recieve example
        let mut send_recv_buf = [0b11001100; 10];
        // You should handle errors here rather than unwrapping
        spi.transfer(&mut send_recv_buf).unwrap();

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
