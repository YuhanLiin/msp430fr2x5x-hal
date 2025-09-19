#![no_main]
#![no_std]
#![feature(abi_msp430_interrupt)]

// Demonstrates a blocking multi-master implementation, and a blocking and interrupt-based master-slave.
// The master sends a byte to the master-slave, then switches to read mode. The master-slave echoes the sent value back to the master.
// The master-slave then probes the bus looking for a device with address 0x09.

// eUSCI B1 is configured as a master-slave. eUSCI B0 is configured as a master.
// Connect:
// P1.2 <--> P4.6
// P1.3 <--> P4.7

// We use UnsafeCell here over RefCell to minimise binary size. Binary size suffers when panics are possible, as they pull in lots of
// strings and formatting. Debug builds from old compiler versions suffer in particular.
use core::cell::UnsafeCell;

use critical_section::Mutex;
use embedded_hal::{delay::DelayNs, digital::OutputPin, i2c::I2c};
use msp430::interrupt::enable as enable_interrupts;
use msp430_rt::entry;
use msp430fr2355::{interrupt, E_USCI_B1};
use msp430fr2x5x_hal::{
    clock::{ClockConfig, DcoclkFreqSel, MclkDiv, SmclkDiv}, fram::Fram, gpio::Batch, 
    i2c::{GlitchFilter, I2cConfig, I2cInterruptFlags as Flags, I2cMasterSlave, I2cVector}, pmm::Pmm, watchdog::Wdt
};
use panic_msp430 as _;

static I2C_MULTI_MASTER: Mutex<UnsafeCell<Option< I2cMasterSlave<E_USCI_B1> >>> = Mutex::new(UnsafeCell::new(None));
// Sets the LED on P1.0 if communication is successful, sets the LED on P6.6 if there is no device with address 0x09 on the bus.
#[entry]
fn main() -> ! {
    let periph = msp430fr2355::Peripherals::take().unwrap();

    let mut fram = Fram::new(periph.FRCTL);
    let _wdt = Wdt::constrain(periph.WDT_A);

    let pmm = Pmm::new(periph.PMM);
    let p1 = Batch::new(periph.P1).split(&pmm);
    let mut red_led = p1.pin0.to_output();
    let mut green_led = Batch::new(periph.P6).split(&pmm).pin6.to_output();
    let p4 = Batch::new(periph.P4).split(&pmm);
    let ms_scl = p4.pin7.pullup().to_alternate1(); // You may need stronger external pullup resistors
    let ms_sda = p4.pin6.pullup().to_alternate1();

    let m_scl = p1.pin3.to_alternate1();
    let m_sda = p1.pin2.to_alternate1();

    let (smclk, _aclk, mut delay) = ClockConfig::new(periph.CS)
        .mclk_dcoclk(DcoclkFreqSel::_8MHz, MclkDiv::_1)
        .smclk_on(SmclkDiv::_1)
        .aclk_vloclk()
        .freeze(&mut fram);

    // Configure an I2C device as both master and slave. The device will automatically failover from master to slave when addressed.
    // Attempting any master actions will fail until the slave event has been handled.
    const MASTER_SLAVE_ADDR: u8 = 26;
    let mut i2c_master_slave = I2cConfig::new(periph.E_USCI_B1, GlitchFilter::Max50ns)
        .as_master_slave(MASTER_SLAVE_ADDR)
        .use_smclk(&smclk, 80) // 8MHz / 80 = 100kHz
        .configure(ms_scl, ms_sda);

    // Make another I2C device to test the master-slave. Since there are now two masters present
    // on the bus this has to be a multi-master, rather than a single-master.
    let mut i2c_master = I2cConfig::new(periph.E_USCI_B0, GlitchFilter::Max50ns)
        .as_multi_master()
        .use_smclk(&smclk, 80) // 8MHz / 80 = 100kHz
        .configure(m_scl, m_sda);

    critical_section::with(|cs| {
        i2c_master_slave.set_interrupts(Flags::StartReceived);
        unsafe { *I2C_MULTI_MASTER.borrow(cs).get() = Some(i2c_master_slave) }
    });
    unsafe { enable_interrupts() };

    loop {
        // The master sends a byte then receives a byte from the master-slave.
        // The master-slave echoes the master's byte back.
        let mut echo_rx = [0; 1];
        const ECHO_TX: [u8; 1] = [128; 1];

        // Start a transaction using the master. This will force the master-slave into slave mode.
        // We don't care about errors for this example, but should be handled in a real implementation.
        let _ = i2c_master.write_read(MASTER_SLAVE_ADDR, &ECHO_TX, &mut echo_rx);

        // The master-slave is set back into master mode in the StopReceived interrupt, so now we can just use it like a master.
        // Here we check if a device with address 0x9 is on the bus (aka a zero-byte write)
        critical_section::with(|cs| {
            let Some(i2c_master_slave) = unsafe { &mut *I2C_MULTI_MASTER.borrow(cs).get() }.as_mut() else { return; };
            if let Ok(false) = i2c_master_slave.is_slave_present(9u8) {
                green_led.set_high().ok(); // Turn on the green LED if address 0x9 is not on bus
            }
        });

        // If the I2C devices echoed correctly set the red LED
        red_led.set_state((echo_rx == ECHO_TX).into()).ok();
        delay.delay_ms(100);
    }
}

// Static mut variables defined inside an interrupt handler are safe. See: https://docs.rust-embedded.org/book/start/interrupts.html
#[allow(static_mut_refs)]
#[interrupt]
fn EUSCI_B1() {
    static mut TEMP_VAR: u8 = 0;
    critical_section::with(|cs| {
        let Some(i2c_master_slave) = unsafe { &mut *I2C_MULTI_MASTER.borrow(cs).get() }.as_mut() else { return; };
        match i2c_master_slave.interrupt_source() {
            I2cVector::StartReceived => {
                // We have been addressed as a slave. Enable Rx, Tx and Stop interrupts.
                i2c_master_slave.set_interrupts(Flags::TxBufEmpty | Flags::RxBufFull | Flags::StopReceived);
            }
            I2cVector::RxBufFull => {
                // Store the received value so we can echo it back later when the master switches to read mode
                *TEMP_VAR = unsafe { i2c_master_slave.read_rx_buf_as_slave_unchecked() };
            }
            I2cVector::TxBufEmpty => {
                // Echo back the stored value
                unsafe { i2c_master_slave.write_tx_buf_as_slave_unchecked(*TEMP_VAR) };
            }
            I2cVector::StopReceived => {
                // Slave addressing concluded. Disable Rx, Tx, and Stop interrupts. We don't want these to trigger when acting as a master.
                i2c_master_slave.clear_interrupts(Flags::TxBufEmpty | Flags::RxBufFull | Flags::StopReceived);
                i2c_master_slave.return_to_master();
            }
            _ => (), // unreachable
        }
    })
}

// The compiler will emit calls to the abort() compiler intrinsic if debug assertions are
// enabled (default for dev profile). MSP430 does not actually have meaningful abort() support
// so for now, we create our own in each application where debug assertions are present.
#[no_mangle]
extern "C" fn abort() -> ! {
    panic!();
}
