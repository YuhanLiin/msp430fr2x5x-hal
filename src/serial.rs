//! Serial UART
//!
//! The peripherals E_USCI_A0 and E_USCI_A1 can be used as serial UARTs.
//!
//! Begin configuration by calling [`SerialConfig::new()`]. After configuration, [`Rx`] and/or [`Tx`] structs are produced by
//! providing the corresponding GPIO pins.
//!
//! The [`Tx`] and [`Rx`] structs are used to send and receive bytes via serial. They implement both [`embedded-io`](embedded_io)'s
//! serial traits (which are buffer-based, blocking), and the single-byte-based non-blocking [`embedded-hal-nb`](embedded_hal_nb::serial) version.
//!
//! As the MSP430 has only a single byte buffer, `embedded-io`'s buffer-based traits can be a bit unwieldy -
//! [`emb_io::Write::write`](embedded_io::Write::write) and [`emb_io::Read::read`](embedded_io::Read::read) will
//! always only send or recieve a single byte, despite taking slices as inputs.
//!
//! For reading or writing single bytes it is recommended to use `embedded-hal-nb`'s
//! [`emb_hal_nb::Read::read`](embedded_hal_nb::serial::Read::read) and
//! [`emb_hal_nb::Write::write`](embedded_hal_nb::serial::Write::write) ([`nb::block`] can be used to make them blocking).
//!
//! For writing multiple bytes, embedded_io's [`Write::write_all`](embedded_io::Write::write_all) and
//! [`Read::read_exact`](embedded_io::Read::read_exact) methods are useful.
//!

use crate::clock::{Aclk, Clock, Smclk};
use crate::gpio::{Alternate1, Pin, Pin1, Pin2, Pin3, Pin5, Pin6, Pin7, P1, P4};
use crate::hw_traits::eusci::{EUsciUart, UartUcxStatw, UcaCtlw0, Ucssel};
use core::convert::Infallible;
use core::marker::PhantomData;
use core::num::NonZeroU32;
use crate::pac;

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
        const ONE: NonZeroU32 = NonZeroU32::new(1).unwrap();
        SerialConfig {
            order,
            cnt,
            stopbits,
            parity,
            loopback,
            usci,
            state: NoClockSet {
                baudrate: NonZeroU32::new(baudrate).unwrap_or(ONE),
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
        const SIXTEEN: NonZeroU32 = NonZeroU32::new(16).unwrap();
        let div = bps.saturating_mul(SIXTEEN);

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
    // bps is between [1, 5_000_000] (datasheet max)
    // clk_freq is between [0, 24_000_000] (datasheet max)

    // modulo = clk_freq % bps => modulo is between [0, 4_999_999]
    let modulo = clk_freq % bps;

    // fraction = modulo * 10_000 / (bps), so within [0, ((bps-1) * 10_000) / bps].
    // To prove upper bound we note `(bps-1)/bps` is largest when bps == 5_000_000:
    // (4_999_999 * 10_000) / 5_000_000 = 49_999_990_000 (watch out for overflow!) / 5_000_000 = 9999.99... truncated to 9_999 because integer division
    // So fraction is within [0, 9999]
    let fraction_as_ten_thousandths = if modulo < u32::MAX/10_000 {
        // Most accurate
        ((modulo * 10_000) / bps) as u16
    } else {
        // Avoid overflow if modulo is large. Assume modulo < 5_000_000 from datasheet max
        (((modulo * 500) / bps) * 20) as u16
    };

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

    // Internal flush function
    #[inline]
    fn flush(&mut self) -> nb::Result<(), Infallible> {
        let usci = unsafe { USCI::steal() };
        if usci.txifg_rd() {
            Ok(())
        } else {
            Err(nb::Error::WouldBlock)
        }
    }

    #[inline(always)]
    /// Writes a byte into the Tx buffer with no checks for validity
    /// # Safety
    /// May clobber unsent data still in the buffer
    pub unsafe fn write_no_check(&mut self, data: u8) {
        let usci = unsafe { USCI::steal() };
        usci.tx_wr(data);
    }

    // Internal send function
    #[inline]
    fn send(&mut self, data: u8) -> nb::Result<(), Infallible> {
        let usci = unsafe { USCI::steal() };
        if usci.txifg_rd() {
            usci.tx_wr(data);
            Ok(())
        } else {
            Err(nb::Error::WouldBlock)
        }
    }
}

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
    /// # Safety
    /// May read duplicate data
    #[inline(always)]
    pub unsafe fn read_no_check(&mut self) -> u8 {
        let usci = unsafe { USCI::steal() };
        usci.rx_rd()
    }

    // Internal recieve function
    fn recv(&mut self) -> nb::Result<u8, RecvError> {
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

/// Serial receive errors
#[derive(Clone, Copy, Debug)]
pub enum RecvError {
    /// Framing error
    Framing,
    /// Parity error
    Parity,
    /// Buffer overrun error. Contains the most recently read byte, which is still valid.
    Overrun(u8),
}

mod emb_io {
    use super::*;
    use embedded_io::{Error, ErrorType, Read, ReadReady, Write, WriteReady};
    use nb::block;

    impl<USCI: SerialUsci> ErrorType for Rx<USCI> { type Error = RecvError; }
    impl Error for RecvError {
        fn kind(&self) -> embedded_io::ErrorKind {
            match self {
                RecvError::Framing      => embedded_io::ErrorKind::Other,
                RecvError::Parity       => embedded_io::ErrorKind::Other,
                RecvError::Overrun(_)   => embedded_io::ErrorKind::Other,
            }
        }
    }
    impl<USCI: SerialUsci> Read for Rx<USCI> {
        #[inline]
        /// Read one byte into the specified buffer, then returns the number of bytes sent (1).
        /// If a byte isn't currently available to read, this function blocks until one is available.
        ///
        /// If `buf` is length zero, `write` returns `Ok(0)` without blocking.
        fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
            if buf.is_empty() { return Ok(0) }
            buf[0] = block!(self.recv())?;
            Ok(1)
        }
    }
    impl<USCI: SerialUsci> ReadReady for Rx<USCI> {
        fn read_ready(&mut self) -> Result<bool, Self::Error> {
            let usci = unsafe { USCI::steal() };
            Ok(usci.rxifg_rd())
        }
    }

    impl<USCI: SerialUsci> ErrorType for Tx<USCI> { type Error = Infallible; }
    impl<USCI: SerialUsci> Write for Tx<USCI> {
        /// Due to errata USCI42, UCTXCPTIFG will fire every time a byte is done transmitting,
        /// even if there's still more buffered. Thus, the implementation uses UCTXIFG instead. When
        /// `flush()` completes, the Tx buffer will be empty but the FIFO may still be sending.
        ///
        /// As the error type is `Infallible`, this can be safely unwrapped.
        #[inline]
        fn flush(&mut self) -> Result<(), Self::Error> {
            block!(self.flush())
        }

        #[inline]
        /// This function sends only **THE FIRST** byte in the buffer, blocking until the writer is ready to accept, then returns `Ok(1)`.
        /// If you want to send the entire buffer use `write_all()` instead.
        ///
        /// If `buf` is length zero, `write` returns `Ok(0)` without blocking.
        ///
        /// As the error type is `Infallible`, this can be safely unwrapped.
        fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
            if buf.is_empty() { return Ok(0) }
            block!(self.send(buf[0]))?;
            Ok(1)
        }
        // The default version of this impl panics if .write() returns Ok(0) when given a non-empty buffer. Our impl never does this, so remove it.
        /// Write an entire buffer into this writer.
        ///
        /// This function calls `write()` in a loop until exactly `buf.len()` bytes have
        /// been written, blocking if needed.
        ///
        /// If you are using [`WriteReady`] to avoid blocking, you should not use this function.
        /// `WriteReady::write_ready()` returning true only guarantees the first call to `write()` will
        /// not block, so this function may still block in subsequent calls.
        fn write_all(&mut self, mut buf: &[u8]) -> Result<(), Self::Error> {
            while !buf.is_empty() {
                let Ok(n) = self.write(buf);
                buf = &buf[n..];
            }
            Ok(())
        }
    }
    impl<USCI: SerialUsci> WriteReady for Tx<USCI> {
        /// Whether the writer is ready for immediate writing. If this returns `true`, the next call to [`Write::write`] will not block.
        ///
        /// As the error type is `Infallible`, this can be safely unwrapped.
        fn write_ready(&mut self) -> Result<bool, Self::Error> {
            let usci = unsafe { USCI::steal() };
            Ok(usci.txifg_rd())
        }
    }
}

mod ehal_nb1 {
    use super::*;
    use embedded_hal_nb::serial::{Error, ErrorKind, ErrorType, Read, Write};

    impl Error for RecvError {
        fn kind(&self) -> ErrorKind {
            match self {
                RecvError::Framing      => ErrorKind::FrameFormat,
                RecvError::Parity       => ErrorKind::Parity,
                RecvError::Overrun(_)   => ErrorKind::Overrun,
            }
        }
    }
    impl<USCI: SerialUsci> ErrorType for Rx<USCI> { type Error = RecvError; }
    impl<USCI: SerialUsci> Read<u8> for Rx<USCI> {
        #[inline]
        /// Check if Rx interrupt flag is set. If so, try reading the received byte and clear the flag.
        /// Otherwise return `WouldBlock`. May return errors caused by data corruption or
        /// buffer overruns.
        fn read(&mut self) -> nb::Result<u8, Self::Error> {
            self.recv()
        }
    }

    impl<USCI: SerialUsci> ErrorType for Tx<USCI> { type Error = Infallible; }
    impl<USCI: SerialUsci> Write<u8> for Tx<USCI> {
        /// Due to errata USCI42, UCTXCPTIFG will fire every time a byte is done transmitting,
        /// even if there's still more buffered. Thus, the implementation uses UCTXIFG instead. When
        /// `flush()` completes, the Tx buffer will be empty but the FIFO may still be sending.
        #[inline]
        fn flush(&mut self) -> nb::Result<(), Self::Error> {
            self.flush()
        }

        #[inline]
        /// Check if Tx interrupt flag is set. If so, write a byte into the Tx buffer. Otherwise return `WouldBlock`
        fn write(&mut self, data: u8) -> nb::Result<(), Self::Error> {
            self.send(data)
        }
    }
}

#[cfg(feature = "embedded-hal-02")]
mod ehal02 {
    use super::*;
    use embedded_hal_02::serial::{Read, Write};

    impl<USCI: SerialUsci> Read<u8> for Rx<USCI> {
        type Error = RecvError;

        #[inline]
        /// Check if Rx interrupt flag is set. If so, try reading the received byte and clear the flag.
        /// Otherwise return `WouldBlock`. May return errors caused by data corruption or
        /// buffer overruns.
        fn read(&mut self) -> nb::Result<u8, Self::Error> {
            self.recv()
        }
    }

    impl<USCI: SerialUsci> Write<u8> for Tx<USCI> {
        type Error = void::Void;

        /// Due to errata USCI42, UCTXCPTIFG will fire every time a byte is done transmitting,
        /// even if there's still more buffered. Thus, the implementation uses UCTXIFG instead. When
        /// `flush()` completes, the Tx buffer will be empty but the FIFO may still be sending.
        #[inline]
        fn flush(&mut self) -> nb::Result<(), Self::Error> {
            self.flush().map_err(|_| nb::Error::WouldBlock)
        }

        #[inline]
        /// Check if Tx interrupt flag is set. If so, write a byte into the Tx buffer. Otherwise return `WouldBlock`
        fn write(&mut self, data: u8) -> nb::Result<(), Self::Error> {
            self.send(data).map_err(|_| nb::Error::WouldBlock)
        }
    }

    impl<USCI: SerialUsci> embedded_hal_02::blocking::serial::write::Default<u8> for Tx<USCI> {}
}
