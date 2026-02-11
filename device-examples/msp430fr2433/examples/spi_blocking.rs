#![no_main]
#![no_std]

// This example uses the SpiBus embedded-hal interface, with a software controlled CS pin.

use embedded_hal::{delay::DelayNs, digital::{OutputPin, StatefulOutputPin}, spi::MODE_0};
use msp430_rt::entry;
use msp430fr2x5x_hal::{
    clock::{ClockConfig, DcoclkFreqSel, MclkDiv, SmclkDiv}, fram::Fram, gpio::Batch, pmm::Pmm, spi::SpiConfig, watchdog::Wdt
};
use panic_msp430 as _;

#[entry]
fn main() -> ! {
    let periph = msp430fr2433::Peripherals::take().unwrap();

    let mut fram = Fram::new(periph.fram);
    let _wdt = Wdt::constrain(periph.watchdog_timer);

    let pmm = Pmm::new(periph.pmm);
    let p1 = Batch::new(periph.p1).split(&pmm);
    let sck    = p1.pin6.to_alternate1();
    let miso   = p1.pin5.to_alternate1();
    let mosi   = p1.pin4.to_alternate1();
    let mut cs = p1.pin3.to_output();
    cs.set_high();
    let mut red_led = p1.pin0.to_output();
    red_led.set_low();

    let (smclk, _aclk, mut delay) = ClockConfig::new(periph.cs)
        .mclk_dcoclk(DcoclkFreqSel::_8MHz, MclkDiv::_1)
        .smclk_on(SmclkDiv::_1)
        .aclk_refoclk()
        .freeze(&mut fram);

    let mut spi = SpiConfig::new(periph.usci_a0_spi_mode, MODE_0, true)
        .to_master_using_smclk(&smclk, 16) // 8MHz / 16 = 500kHz
        .single_master_bus(miso, mosi, sck);

    // The embedded-hal ecosystem includes multiple SPI traits:
    // embedded-hal contains the basic SpiBus trait, which models just the SPI bus (good for devices with no chip select pin).
    // embedded-hal also contains SpiDevice, which facilitates automatic chip select pin management and bus sharing. embedded-hal-bus provides common implementations of SpiDevice.
    // embedded-hal-nb contains a non-blocking interface through spi::FullDuplex.

    // For simplicity we'll use SpiBus here.
    use embedded_hal::spi::SpiBus;

    loop {
        // Perform the following transaction:
        // Send: 0x12, 0x00,    0x00,    0x34,    0x56,
        // Recv: N/A,  recv[0], recv[1], recv[2], N/A
        let mut recv = [0; 3];
        cs.set_low();
            // These methods do return errors, but because we haven't used the non-blocking
            // API (from embedded-hal-nb) or interrupts the Rx buffer should never overrun because
            // the blocking interface automatically reads after every write.
            spi.write(&[0x12]).unwrap();
            spi.read(&mut recv[0..2]).unwrap();
            spi.transfer(&mut recv[2..], &[0x34, 0x56]).unwrap();
            spi.flush().unwrap();
        cs.set_high();

        red_led.toggle();
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
