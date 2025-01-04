//! Serial UART
//!
//! The peripherals E_USCI_0 and E_USCI_1 can be used as serial UARTs.
//! After configuring the E_USCI peripherals, serial Rx and/or Tx pins can be obtained by
//! converting the appropriate GPIO pins to the alternate function corresponding to UART.
//!
//! The Tx and Rx pins are used to send and receive bytes via serial connection.

use crate::clock::{Aclk, Clock, Smclk};
use crate::gpio::{Alternate1, Pin, Pin1, Pin2, Pin3, Pin5, Pin6, Pin7, P1, P4};
use crate::hw_traits::eusci::{EUsciUart, UartUcxStatw, UcaCtlw0, Ucssel};
use core::marker::PhantomData;
use core::num::NonZeroU32;
use embedded_hal::serial::{Read, Write};
use msp430fr2355 as pac;

/// Bit order of transmit and receive
#[derive(Clone, Copy)]
pub enum BitOrder {
    /// LSB first (typically the default)
    LsbFirst,
    /// MSB first
    MsbFirst,
}

impl BitOrder {
    #[inline(always)]
    fn to_bool(self) -> bool {
        match self {
            BitOrder::LsbFirst => false,
            BitOrder::MsbFirst => true,
        }
    }
}

/// Number of bits per transaction
#[derive(Clone, Copy)]
pub enum BitCount {
    /// 8 bits
    EightBits,
    /// 7 bits
    SevenBits,
}

impl BitCount {
    #[inline(always)]
    fn to_bool(self) -> bool {
        match self {
            BitCount::EightBits => false,
            BitCount::SevenBits => true,
        }
    }
}

/// Number of stop bits at end of each byte
#[derive(Clone, Copy)]
pub enum StopBits {
    /// 1 stop bit
    OneStopBit,
    /// 2 stop bits
    TwoStopBits,
}

impl StopBits {
    #[inline(always)]
    fn to_bool(self) -> bool {
        match self {
            StopBits::OneStopBit => false,
            StopBits::TwoStopBits => true,
        }
    }
}

/// Parity bit for error checking
#[derive(Clone, Copy)]
pub enum Parity {
    /// No parity
    NoParity,
    /// Odd parity
    OddParity,
    /// Even parity
    EvenParity,
}

impl Parity {
    #[inline(always)]
    fn ucpen(self) -> bool {
        match self {
            Parity::NoParity => false,
            _ => true,
        }
    }

    #[inline(always)]
    fn ucpar(self) -> bool {
        match self {
            Parity::OddParity => false,
            Parity::EvenParity => true,
            _ => false,
        }
    }
}

/// Loopback settings
#[derive(Clone, Copy)]
pub enum Loopback {
    /// No loopback
    NoLoop,
    /// Tx feeds into Rx
    Loopback,
}

impl Loopback {
    #[inline(always)]
    fn to_bool(self) -> bool {
        match self {
            Loopback::NoLoop => false,
            Loopback::Loopback => true,
        }
    }
}

/// Marks a USCI type that can be used as a serial UART
pub trait SerialUsci: EUsciUart {
    /// Pin used for serial UCLK
    type ClockPin;
    /// Pin used for Tx
    type TxPin;
    /// Pin used for Rx
    type RxPin;
}

impl SerialUsci for pac::E_USCI_A0 {
    type ClockPin = UsciA0ClockPin;
    type TxPin = UsciA0TxPin;
    type RxPin = UsciA0RxPin;
}

macro_rules! impl_serial_pin {
    ($struct_name: ident, $port: ty, $pin: ty) => {
        impl<DIR> From<Pin<$port, $pin, Alternate1<DIR>>> for $struct_name {
            #[inline(always)]
            fn from(_val: Pin<$port, $pin, Alternate1<DIR>>) -> Self {
                $struct_name
            }
        }
    };
}

/// UCLK pin for E_USCI_A0
pub struct UsciA0ClockPin;
impl_serial_pin!(UsciA0ClockPin, P1, Pin5);

/// Tx pin for E_USCI_A0
pub struct UsciA0TxPin;
impl_serial_pin!(UsciA0TxPin, P1, Pin7);

/// Rx pin for E_USCI_A0
pub struct UsciA0RxPin;
impl_serial_pin!(UsciA0RxPin, P1, Pin6);

impl SerialUsci for pac::E_USCI_A1 {
    type ClockPin = UsciA1ClockPin;
    type TxPin = UsciA1TxPin;
    type RxPin = UsciA1RxPin;
}

/// UCLK pin for E_USCI_A1
pub struct UsciA1ClockPin;
impl_serial_pin!(UsciA1ClockPin, P4, Pin1);

/// Tx pin for E_USCI_A1
pub struct UsciA1TxPin;
impl_serial_pin!(UsciA1TxPin, P4, Pin3);

/// Rx pin for E_USCI_A1
pub struct UsciA1RxPin;
impl_serial_pin!(UsciA1RxPin, P4, Pin2);

/// Typestate for a serial interface with an unspecified clock source
pub struct NoClockSet {
    baudrate: NonZeroU32,
}

/// Typestate for a serial interface with a specified clock source
pub struct ClockSet {
    baud_config: BaudConfig,
    clksel: Ucssel,
}

/// Builder object for configuring a serial UART
///
/// Once the clock source has been selected, the builder can be converted into pins that can
/// transmit or received bytes via a serial connection.
pub struct SerialConfig<USCI: SerialUsci, S> {
    usci: USCI,
    order: BitOrder,
    cnt: BitCount,
    stopbits: StopBits,
    parity: Parity,
    loopback: Loopback,
    state: S,
}

macro_rules! serial_config {
    ($conf:expr, $state:expr) => {
        SerialConfig {
            usci: $conf.usci,
            order: $conf.order,
            cnt: $conf.cnt,
            stopbits: $conf.stopbits,
            parity: $conf.parity,
            loopback: $conf.loopback,
            state: $state,
        }
    };
}

impl<USCI: SerialUsci> SerialConfig<USCI, NoClockSet> {
    /// Create a new serial configuration using a EUSCI peripheral
    #[inline]
    pub fn new(
        usci: USCI,
        order: BitOrder,
        cnt: BitCount,
        stopbits: StopBits,
        parity: Parity,
        loopback: Loopback,
        baudrate: u32,
    ) -> Self {
        SerialConfig {
            order,
            cnt,
            stopbits,
            parity,
            loopback,
            usci,
            // Safety: .max(1) ensures baudrate is non-zero
            state: NoClockSet {
                baudrate: NonZeroU32::new(baudrate).unwrap_or( const {NonZeroU32::new(1).unwrap()} ),
            },
        }
    }

    /// Configure serial UART to use external UCLK, passing in the appropriately configured pin
    /// used as the clock signal as well as the frequency of the clock.
    #[inline(always)]
    pub fn use_uclk<P: Into<USCI::ClockPin>>(
        self,
        _clk_pin: P,
        freq: u32,
    ) -> SerialConfig<USCI, ClockSet> {
        serial_config!(
            self,
            ClockSet {
                baud_config: calculate_baud_config(freq, self.state.baudrate),
                clksel: Ucssel::Uclk,
            }
        )
    }

    /// Configure serial UART to use ACLK.
    #[inline(always)]
    pub fn use_aclk(self, aclk: &Aclk) -> SerialConfig<USCI, ClockSet> {
        serial_config!(
            self,
            ClockSet {
                baud_config: calculate_baud_config(aclk.freq() as u32, self.state.baudrate),
                clksel: Ucssel::Aclk,
            }
        )
    }

    /// Configure serial UART to use SMCLK.
    #[inline(always)]
    pub fn use_smclk(self, smclk: &Smclk) -> SerialConfig<USCI, ClockSet> {
        serial_config!(
            self,
            ClockSet {
                baud_config: calculate_baud_config(smclk.freq(), self.state.baudrate),
                clksel: Ucssel::Smclk,
            }
        )
    }
}

struct BaudConfig {
    br: u16,
    brs: u8,
    brf: u8,
    ucos16: bool,
}

#[inline]
fn calculate_baud_config(clk_freq: u32, bps: NonZeroU32) -> BaudConfig {
    // Ensure n stays within the 16 bit boundary
    let n = (clk_freq / bps).clamp(1, 0xFFFF);

    let brs = lookup_brs(clk_freq, bps);

    if (n >= 16) && (bps.get() < u32::MAX / 16) {
        //  div = bps * 16
        let div = bps.saturating_mul(const { NonZeroU32::new(16).unwrap() });

        // n / 16, but more precise
        let br = (clk_freq / div) as u16;

        // same as n % 16, but more precise
        let brf = ((clk_freq % div) / bps) as u8;
        BaudConfig {
            ucos16: true,
            br,
            brf,
            brs,
        }
    } else {
        BaudConfig {
            ucos16: false,
            br: n as u16,
            brf: 0,
            brs,
        }
    }
}

#[inline(always)]
fn lookup_brs(clk_freq: u32, bps: NonZeroU32) -> u8 {
    // bps is between [1, u32::MAX]
    // clk_freq is between [0, u32::MAX]

    // modulo = clk_freq % bps => modulo is between [0, bps-1]
    let modulo = clk_freq % bps;

    // fraction = modulo * 10_000 / bps, so within [0, ((bps-1) * 10_000) / bps].
    // To prove upper bound we note `(bps-1)/bps` is largest when bps == u32::MAX:
    // (4_294_967_294 * 10_000) / 4_294_967_295 = 42_949_672_940_000 / 4_294_967_295 = 9999.99... truncated to 9_999 because integer division
    // So fraction is within [0, 9999]
    let fraction_as_ten_thousandths =
        ((modulo as u64 * 10_000) / core::num::NonZeroU64::from(bps)) as u16;

    // See Table 22-4 from MSP430FR4xx and MSP430FR2xx family user's guide (Rev. I)
    match fraction_as_ten_thousandths {
        0..529     => 0x00,
        529..715   => 0x01,
        715..835   => 0x02,
        835..1001  => 0x04,
        1001..1252 => 0x08,
        1252..1430 => 0x10,
        1430..1670 => 0x20,
        1670..2147 => 0x11,
        2147..2224 => 0x21,
        2224..2503 => 0x22,
        2503..3000 => 0x44,
        3000..3335 => 0x25,
        3335..3575 => 0x49,
        3575..3753 => 0x4A,
        3753..4003 => 0x52,
        4003..4286 => 0x92,
        4286..4378 => 0x53,
        4378..5002 => 0x55,
        5002..5715 => 0xAA,
        5715..6003 => 0x6B,
        6003..6254 => 0xAD,
        6254..6432 => 0xB5,
        6432..6667 => 0xB6,
        6667..7001 => 0xD6,
        7001..7147 => 0xB7,
        7147..7503 => 0xBB,
        7503..7861 => 0xDD,
        7861..8004 => 0xED,
        8004..8333 => 0xEE,
        8333..8464 => 0xBF,
        8464..8572 => 0xDF,
        8572..8751 => 0xEF,
        8751..9004 => 0xF7,
        9004..9170 => 0xFB,
        9170..9288 => 0xFD,
        9288..     => 0xFE,
    }
}

impl<USCI: SerialUsci> SerialConfig<USCI, ClockSet> {
    #[inline]
    fn config_hw(self) {
        let ClockSet {
            baud_config,
            clksel,
        } = self.state;
        let usci = self.usci;

        usci.ctl0_reset();
        usci.brw_settings(baud_config.br);
        usci.mctlw_settings(baud_config.ucos16, baud_config.brs, baud_config.brf);
        usci.loopback(self.loopback.to_bool());
        usci.ctl0_settings(UcaCtlw0 {
            ucpen: self.parity.ucpen(),
            ucpar: self.parity.ucpar(),
            ucmsb: self.order.to_bool(),
            uc7bit: self.cnt.to_bool(),
            ucspb: self.stopbits.to_bool(),
            ucssel: clksel,
            // We want erroneous bytes to trigger RXIFG so all errors can be caught
            ucrxeie: true,
        });
    }

    /// Perform hardware configuration and split into Tx and Rx pins from appropriate GPIOs
    #[inline]
    pub fn split<T: Into<USCI::TxPin>, R: Into<USCI::RxPin>>(
        self,
        _tx: T,
        _rx: R,
    ) -> (Tx<USCI>, Rx<USCI>) {
        self.config_hw();
        (Tx(PhantomData), Rx(PhantomData))
    }

    /// Perform hardware configuration and create Tx pin from appropriate GPIO
    #[inline]
    pub fn tx_only<T: Into<USCI::TxPin>>(self, _tx: T) -> Tx<USCI> {
        self.config_hw();
        Tx(PhantomData)
    }

    /// Perform hardware configuration and create Rx pin from appropriate GPIO
    #[inline]
    pub fn rx_only<R: Into<USCI::RxPin>>(self, _rx: R) -> Rx<USCI> {
        self.config_hw();
        Rx(PhantomData)
    }
}

/// Serial transmitter pin
pub struct Tx<USCI: SerialUsci>(PhantomData<USCI>);

impl<USCI: SerialUsci> Tx<USCI> {
    /// Enable Tx interrupts, which fire when ready to send.
    #[inline(always)]
    pub fn enable_tx_interrupts(&mut self) {
        let usci = unsafe { USCI::steal() };
        usci.txie_set();
    }

    /// Disable Tx interrupts
    #[inline(always)]
    pub fn disable_tx_interrupts(&mut self) {
        let usci = unsafe { USCI::steal() };
        usci.txie_clear();
    }
}

impl<USCI: SerialUsci> Write<u8> for Tx<USCI> {
    type Error = void::Void;

    /// Due to errata USCI42, UCTXCPTIFG will fire every time a byte is done transmitting,
    /// even if there's still more buffered. Thus, the implementation uses UCTXIFG instead. When
    /// `flush()` completes, the Tx buffer will be empty but the FIFO may still be sending.
    #[inline]
    fn flush(&mut self) -> nb::Result<(), Self::Error> {
        let usci = unsafe { USCI::steal() };
        if usci.txifg_rd() {
            Ok(())
        } else {
            Err(nb::Error::WouldBlock)
        }
    }

    #[inline]
    /// Check if Tx interrupt flag is set. If so, write a byte into the Tx buffer. Otherwise block
    /// on the Tx flag.
    fn write(&mut self, data: u8) -> nb::Result<(), Self::Error> {
        let usci = unsafe { USCI::steal() };
        if usci.txifg_rd() {
            usci.tx_wr(data);
            Ok(())
        } else {
            Err(nb::Error::WouldBlock)
        }
    }
}

impl<USCI: SerialUsci> embedded_hal::blocking::serial::write::Default<u8> for Tx<USCI> {}

/// Serial receiver pin
pub struct Rx<USCI: SerialUsci>(PhantomData<USCI>);

impl<USCI: SerialUsci> Rx<USCI> {
    /// Enable Rx interrupts, which fire when ready to read
    #[inline(always)]
    pub fn enable_rx_interrupts(&mut self) {
        let usci = unsafe { USCI::steal() };
        usci.rxie_set();
    }

    /// Disable Rx interrupts
    #[inline(always)]
    pub fn disable_rx_interrupts(&mut self) {
        let usci = unsafe { USCI::steal() };
        usci.rxie_clear();
    }

    /// Reads raw value from Rx buffer with no checks for validity
    #[inline(always)]
    pub fn read_no_check(&mut self) -> u8 {
        let usci = unsafe { USCI::steal() };
        usci.rx_rd()
    }

    #[inline(always)]
    /// Writes a byte into the Tx buffer with no checks for validity
    pub fn write_no_check(&mut self, data: u8) {
        let usci = unsafe { USCI::steal() };
        usci.tx_wr(data);
    }
}

/// Serial receive errors
pub enum RecvError {
    /// Framing error
    Framing,
    /// Parity error
    Parity,
    /// Buffer overrun error. Contains the most recently read byte, which is still valid.
    Overrun(u8),
}

impl<USCI: SerialUsci> Read<u8> for Rx<USCI> {
    type Error = RecvError;

    #[inline]
    /// Check if Rx interrupt flag is set. If so, try reading the received byte and clear the flag.
    /// Otherwise block on the Rx interrupt flag. May return errors caused by data corruption or
    /// buffer overruns.
    fn read(&mut self) -> nb::Result<u8, Self::Error> {
        let usci = unsafe { USCI::steal() };

        if usci.rxifg_rd() {
            let statw = usci.statw_rd();
            let data = usci.rx_rd();

            if statw.ucfe() {
                Err(nb::Error::Other(RecvError::Framing))
            } else if statw.ucpe() {
                Err(nb::Error::Other(RecvError::Parity))
            } else if statw.ucoe() {
                Err(nb::Error::Other(RecvError::Overrun(data)))
            } else {
                Ok(data)
            }
        } else {
            Err(nb::Error::WouldBlock)
        }
    }
}
