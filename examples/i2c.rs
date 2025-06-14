#![no_main]
#![no_std]

use embedded_hal::{delay::DelayNs, digital::{OutputPin, StatefulOutputPin}, i2c::{I2c, Operation}};
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
    let mut red_led = Batch::new(periph.P1).split(&pmm).pin0.to_output();
    let mut green_led = Batch::new(periph.P6).split(&pmm).pin6.to_output();
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
        .use_smclk(&smclk, 80) // 8MHz / 80 = 100kHz
        .configure(scl, sda);
    
    loop {
        // Below are examples of the various I2C methods provided for writing to / reading from the bus.
        // 7- and 10-bit addressing modes are controlled by passing the address as either a u8 or a u16.
        
        let mut is_ok = true;

        // Check if anything with this address is present on the bus by 
        // sending a zero-byte write and listening for an ACK.
        if !i2c.is_slave_present(0x29_u8) {
            is_ok = false;
        }
        
        // Blocking write. Write two bytes (length of buffer) to address 0x12.
        // If a NACK is recieved the transmission is aborted.
        let wr_res = i2c.write(0x12_u8, &[(1<<7) + (0b01 << 5), 0b11]);
        if wr_res.is_err() {
            is_ok = false;
        }
        
        // Blocking read. Read one byte from address 0x12.
        // Each byte recieved is automatically ACKed, except for the last one which is NACKed.
        let mut recv = [0];
        let rd_res = i2c.read(0x12_u8, &mut recv);
        if rd_res.is_err() {
            is_ok = false;
        }
        
        // Do a write then a read within one transaction.
        // Commonly used to read a specific register from the slave.
        // There is no 'stop' between the write and read, only a repeated start.
        let wr_rd_res = i2c.write_read(0x12_u8, &[(1<<7) + (0b01 << 5), 0b11], &mut recv);
        if wr_rd_res.is_err() {
            is_ok = false;
        }

        // Do any arbitrary transaction. One initial start, a repeated
        // start between operations of dissimilar types, and a stop at the end.
        // This particular example is equivalent to the write_read call above.
        let tr_res = i2c.transaction(0x12_u8, &mut [
            Operation::Write(&[(1<<7) + (0b01 << 5), 0b11]), 
            Operation::Read(&mut recv)
        ]);
        if tr_res.is_err() {
            is_ok = false;
        }

        green_led.set_state(is_ok.into()).ok();
        red_led.toggle().ok();
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
