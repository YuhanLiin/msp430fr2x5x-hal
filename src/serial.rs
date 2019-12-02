//! Serial UART

use crate::clock::{Aclk, Clock, Smclk};
use crate::gpio::{Alternate1, Pin, Pin1, Pin2, Pin3, Pin5, Pin6, Pin7, Port1, Port4};
use crate::hw_traits::eusci::UcxCtl0;
use crate::hw_traits::eusci::{EUsci, EUsciUart, UcaxStatw, Ucssel};
use core::marker::PhantomData;
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

/// Loopback enabled
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
pub trait SerialUsci: Sized {
    /// Peripheral type
    type Periph: EUsciUart;
    /// Pin used for serial UCLK
    type ClockPin;
    /// Pin used for Tx
    type TxPin;
    /// Pin used for Rx
    type RxPin;

    /// Transform the peripheral into a Serial object
    #[inline]
    fn to_serial(
        self,
        order: BitOrder,
        cnt: BitCount,
        stopbits: StopBits,
        parity: Parity,
        loopback: Loopback,
        baudrate: u32,
    ) -> SerialConfigNoClock<Self> {
        SerialConfigNoClock {
            order,
            cnt,
            stopbits,
            parity,
            loopback,
            baudrate,
            _usci: PhantomData,
        }
    }
}

impl SerialUsci for pac::E_USCI_A0 {
    type Periph = pac::e_usci_a0::RegisterBlock;
    type ClockPin = UsciA0ClockPin;
    type TxPin = UsciA0TxPin;
    type RxPin = UsciA0RxPin;
}

/// UCLK pin for E_USCI_A0
pub struct UsciA0ClockPin;
impl<DIR> Into<UsciA0ClockPin> for Pin<Port1, Pin5, Alternate1<DIR>> {
    #[inline(always)]
    fn into(self) -> UsciA0ClockPin {
        UsciA0ClockPin
    }
}

/// Tx pin for E_USCI_A0
pub struct UsciA0TxPin;
impl<DIR> Into<UsciA0TxPin> for Pin<Port1, Pin7, Alternate1<DIR>> {
    #[inline(always)]
    fn into(self) -> UsciA0TxPin {
        UsciA0TxPin
    }
}

/// Rx pin for E_USCI_A0
pub struct UsciA0RxPin;
impl<DIR> Into<UsciA0RxPin> for Pin<Port1, Pin6, Alternate1<DIR>> {
    #[inline(always)]
    fn into(self) -> UsciA0RxPin {
        UsciA0RxPin
    }
}

impl SerialUsci for pac::E_USCI_A1 {
    type Periph = pac::e_usci_a1::RegisterBlock;
    type ClockPin = UsciA1ClockPin;
    type TxPin = UsciA1TxPin;
    type RxPin = UsciA1RxPin;
}

/// UCLK pin for E_USCI_A1
pub struct UsciA1ClockPin;
impl<DIR> Into<UsciA1ClockPin> for Pin<Port4, Pin1, Alternate1<DIR>> {
    #[inline(always)]
    fn into(self) -> UsciA1ClockPin {
        UsciA1ClockPin
    }
}

/// Tx pin for E_USCI_A1
pub struct UsciA1TxPin;
impl<DIR> Into<UsciA1TxPin> for Pin<Port4, Pin3, Alternate1<DIR>> {
    #[inline(always)]
    fn into(self) -> UsciA1TxPin {
        UsciA1TxPin
    }
}

/// Rx pin for E_USCI_A1
pub struct UsciA1RxPin;
impl<DIR> Into<UsciA1RxPin> for Pin<Port4, Pin2, Alternate1<DIR>> {
    #[inline(always)]
    fn into(self) -> UsciA1RxPin {
        UsciA1RxPin
    }
}

/// Configuration object for serial UART
pub struct SerialConfigNoClock<USCI: SerialUsci> {
    _usci: PhantomData<USCI>,
    order: BitOrder,
    cnt: BitCount,
    stopbits: StopBits,
    parity: Parity,
    loopback: Loopback,
    baudrate: u32,
}

/// Configuration object for serial UART
pub struct SerialConfig<USCI: SerialUsci> {
    config: SerialConfigNoClock<USCI>,
    clksel: Ucssel,
    freq: u32,
}

impl<USCI: SerialUsci> SerialConfigNoClock<USCI> {
    /// Configure serial UART to use external UCLK, passing in the appropriately configured pin
    /// used as the clock signal as well as the frequency of the clock.
    #[inline(always)]
    pub fn use_uclk<P: Into<USCI::ClockPin>>(self, _clk_pin: P, freq: u32) -> SerialConfig<USCI> {
        SerialConfig {
            config: self,
            clksel: Ucssel::Uclk,
            freq,
        }
    }

    /// Configure serial UART to use ACLK.
    #[inline(always)]
    pub fn use_aclk(self, aclk: &Aclk) -> SerialConfig<USCI> {
        SerialConfig {
            config: self,
            clksel: Ucssel::Aclk,
            freq: aclk.freq() as u32,
        }
    }

    /// Configure serial UART to use SMCLK.
    #[inline(always)]
    pub fn use_smclk(self, smclk: &Smclk) -> SerialConfig<USCI> {
        SerialConfig {
            config: self,
            clksel: Ucssel::Smclk,
            freq: smclk.freq(),
        }
    }
}

struct BaudConfig {
    br: u16,
    brs: u8,
    brf: u8,
    ucos16: bool,
}

// Optimization hack to make sure this expensive magic baudrate calculation is always inlined
// during release mode. Specifically, inlining causes this function to be const-propagated away
// when LTO is enabled, but blows up code size without LTO. This attribute achieves both of best
// worlds, assuming release mode has LTO enabled
#[cfg_attr(not(debug_assertions), inline(always))]
fn calculate_baud_config(clk_freq: u32, bps: u32) -> BaudConfig {
    assert!(bps != 0);
    let n = clk_freq / bps;
    assert!(n > 0, "BPS too high");
    assert!(n <= 0xFFFF, "BPS too low");

    let brs = lookup_brs(clk_freq, bps);

    if n >= 16 {
        let div = bps * 16;
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
fn lookup_brs(clk_freq: u32, bps: u32) -> u8 {
    let modulo = clk_freq % bps;

    // Fractional part lookup for the baud rate. Not extremely precise
    if modulo * 19 < bps {
        0x0
    } else if modulo * 14 < bps {
        0x1
    } else if modulo * 12 < bps {
        0x2
    } else if modulo * 10 < bps {
        0x4
    } else if modulo * 8 < bps {
        0x8
    } else if modulo * 7 < bps {
        0x10
    } else if modulo * 6 < bps {
        0x20
    } else if modulo * 5 < bps {
        0x11
    } else if modulo * 4 < bps {
        0x22
    } else if modulo * 3 < bps {
        0x44
    } else if modulo * 11 < bps * 4 {
        0x49
    } else if modulo * 5 < bps * 2 {
        0x4A
    } else if modulo * 7 < bps * 3 {
        0x92
    } else if modulo * 2 < bps {
        0x53
    } else if modulo * 7 < bps * 4 {
        0xAA
    } else if modulo * 13 < bps * 8 {
        0x6B
    } else if modulo * 3 < bps * 2 {
        0xAD
    } else if modulo * 11 < bps * 8 {
        0xD6
    } else if modulo * 4 < bps * 3 {
        0xBB
    } else if modulo * 5 < bps * 4 {
        0xDD
    } else if modulo * 9 < bps * 8 {
        0xEF
    } else {
        0xFD
    }
}

impl<USCI: SerialUsci> SerialConfig<USCI> {
    #[inline]
    fn config_hw(self) {
        let SerialConfig {
            config,
            clksel,
            freq,
        } = self;
        let usci = USCI::Periph::steal();

        let baud_config = calculate_baud_config(freq, config.baudrate);

        usci.ctl0_reset();
        usci.brw_settings(baud_config.br);
        usci.mctlw_settings(baud_config.ucos16, baud_config.brs, baud_config.brf);
        usci.loopback(config.loopback.to_bool());
        usci.ctl0_settings(UcxCtl0 {
            ucpen: config.parity.ucpen(),
            ucpar: config.parity.ucpar(),
            ucmsb: config.order.to_bool(),
            uc7bit: config.cnt.to_bool(),
            ucspb: config.stopbits.to_bool(),
            ucssel: clksel,
            // We want erroneous bytes to trigger RXIFG so all errors can be caught
            ucrxeie: true,
        });
    }

    /// Perform hardware configuration and split into Tx and Rx pins from appropriate GPIOs
    ///
    /// # Panics
    /// Panics if configured baudrate is higher than the clock rate or less than clock rate / 0xFFFF.
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
    ///
    /// # Panics
    /// Panics if configured baudrate is higher than the clock rate or less than clock rate / 0xFFFF.
    #[inline]
    pub fn tx_only<T: Into<USCI::TxPin>>(self, _tx: T) -> (Tx<USCI>) {
        self.config_hw();
        Tx(PhantomData)
    }

    /// Perform hardware configuration and create Rx pin from appropriate GPIO
    ///
    /// # Panics
    /// Panics if configured baudrate is higher than the clock rate or less than clock rate / 0xFFFF.
    #[inline]
    pub fn rx_only<R: Into<USCI::RxPin>>(self, _rx: R) -> (Rx<USCI>) {
        self.config_hw();
        Rx(PhantomData)
    }
}

/// Serial transmitter
pub struct Tx<USCI: SerialUsci>(PhantomData<USCI>);

impl<USCI: SerialUsci> Tx<USCI> {
    /// Enable Tx interrupts, which fire when ready to send
    #[inline(always)]
    pub fn enable_tx_interrupts(&mut self) {
        let usci = USCI::Periph::steal();
        usci.txie_set();
    }

    /// Disable Tx interrupts
    #[inline(always)]
    pub fn disable_tx_interrupts(&mut self) {
        let usci = USCI::Periph::steal();
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
        let usci = USCI::Periph::steal();
        if usci.txifg_rd() {
            Ok(())
        } else {
            Err(nb::Error::WouldBlock)
        }
    }

    #[inline]
    fn write(&mut self, data: u8) -> nb::Result<(), Self::Error> {
        let usci = USCI::Periph::steal();
        if usci.txifg_rd() {
            usci.tx_wr(data);
            Ok(())
        } else {
            Err(nb::Error::WouldBlock)
        }
    }
}

impl<USCI: SerialUsci> embedded_hal::blocking::serial::write::Default<u8> for Tx<USCI> {}

/// Serial receiver
pub struct Rx<USCI: SerialUsci>(PhantomData<USCI>);

impl<USCI: SerialUsci> Rx<USCI> {
    /// Enable Rx interrupts, which fire when ready to read
    #[inline(always)]
    pub fn enable_rx_interrupts(&mut self) {
        let usci = USCI::Periph::steal();
        usci.rxie_set();
    }

    /// Disable Rx interrupts
    #[inline(always)]
    pub fn disable_rx_interrupts(&mut self) {
        let usci = USCI::Periph::steal();
        usci.rxie_clear();
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
    fn read(&mut self) -> nb::Result<u8, Self::Error> {
        let usci = USCI::Periph::steal();

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
