//! embedded_hal SPI implmentation
use core::marker::PhantomData;
use core::cell::RefCell;
use embedded_hal::spi::FullDuplex;
use msp430::critical_section::with;
use msp430fr2355 as pac;
use crate::hal::{
    spi::{Mode, Polarity, Phase},
    blocking::spi::write,
};
use crate::{
    hw_traits::Steal, hw_traits::eusci::{EusciSPI, UcxSpiCtw0, Ucmode, Ucssel, SPIInterruptVector},
    gpio::{Alternate1, Pin, P1, P4, Pin0, Pin1, Pin2, Pin3, Pin4, Pin5, Pin6, Pin7},
    clock::{Smclk, Aclk},
    hal
};
use msp430::interrupt::{Mutex};
use nb::Error::{WouldBlock};
use crate::hw_traits::eusci::EusciSPIInterrupter;

/// Marks a eUSCI capable of SPI communication (in this case, all euscis do)
pub trait EUsciSPIBus : EusciSPIPriv{
    /// Master In Slave Out (refered to as SOMI in datasheet)
    type MISO;
    /// Master Out Slave In (refered to as SIMO in datasheet)
    type MOSI;
    /// Serial Clock
    type SCLK;
    /// Slave Transmit Enable (acts like CS)
    type STE;
}

#[doc(hidden)]
pub trait EusciSPIPriv : EusciSPI{
    fn get_rx_buf() -> &'static Mutex<RefCell<QueueBuf>>;
    fn get_tx_buf() -> &'static Mutex<RefCell<QueueBuf>>;
}

#[doc(hidden)]
pub struct QueueBuf{
    buf: [u8;16],
    curr: u8, //ptr to current slot to get from
    next: u8, //ptr to next available slot
}

impl QueueBuf{
    const fn new() -> Self{
        QueueBuf{
            buf: [0;16],
            curr: 0,
            next: 0,
        }
    }

    #[inline]
    fn inc(val:u8) -> u8{
        (val+1) & 0xF
    }

    #[inline]
    fn has_data(&self) -> bool{
        return self.curr != self.next;
    }

    #[inline]
    fn slots_left(&self) -> u8{
        15u8 - (self.next - self.curr)
    }

    #[inline]
    fn is_full(&self) -> bool{
        return self.curr == Self::inc(self.next);
    }

    #[inline]
    fn is_empty(&self) -> bool{
        return  self.curr == self.next;
    }

    //make sure to check for fullness before calling
    fn put(&mut self, val: u8){
        self.buf[self.next as usize] = val;
        self.next = Self::inc(self.next);
    }

    //make sure to check for data before calling
    fn get(&mut self) -> u8{
        let val = self.buf[self.curr as usize];
        self.curr = Self::inc(self.curr);
        val
    }
}



static SPI_RX_BUF_A0: Mutex<RefCell<QueueBuf>> = Mutex::new(RefCell::new(QueueBuf::new()));
static SPI_TX_BUF_A0: Mutex<RefCell<QueueBuf>> = Mutex::new(RefCell::new(QueueBuf::new()));

impl EusciSPIPriv for pac::E_USCI_A0 {
    #[inline]
    fn get_rx_buf() -> &'static Mutex<RefCell<QueueBuf>>{
        & SPI_RX_BUF_A0
    }

    #[inline]
    fn get_tx_buf() ->&'static Mutex<RefCell<QueueBuf>>{
        & SPI_TX_BUF_A0
    }

}

impl EusciSPIInterrupter for pac::E_USCI_A0 {
    #[inline]
    unsafe fn spi_interrupt(){
        spi_interrupt_shared(pac::E_USCI_A0::steal());
    }
}

impl EUsciSPIBus for pac::E_USCI_A0 {
    type MISO = UsciA0MISOPin;
    type MOSI = UsciA0MOSIPin;
    type SCLK = UsciA0SCLKPin;
    type STE  = UsciA0STEPin;
}

static SPI_RX_BUF_A1: Mutex<RefCell<QueueBuf>> = Mutex::new(RefCell::new(QueueBuf::new()));
static SPI_TX_BUF_A1: Mutex<RefCell<QueueBuf>> = Mutex::new(RefCell::new(QueueBuf::new()));

impl EusciSPIPriv for pac::E_USCI_A1 {
    #[inline]
    fn get_rx_buf() -> &'static Mutex<RefCell<QueueBuf>>{
        & SPI_RX_BUF_A1
    }

    #[inline]
    fn get_tx_buf() -> &'static Mutex<RefCell<QueueBuf>>{
        & SPI_TX_BUF_A1
    }

}

impl EusciSPIInterrupter for pac::E_USCI_A1 {
    #[inline]
    unsafe fn spi_interrupt(){
        spi_interrupt_shared(pac::E_USCI_A1::steal());
    }
}

impl EUsciSPIBus for pac::E_USCI_A1 {
    type MISO = UsciA1MISOPin;
    type MOSI = UsciA1MOSIPin;
    type SCLK = UsciA1SCLKPin;
    type STE  = UsciA1STEPin;
}

static SPI_RX_BUF_B0: Mutex<RefCell<QueueBuf>> = Mutex::new(RefCell::new(QueueBuf::new()));
static SPI_TX_BUF_B0: Mutex<RefCell<QueueBuf>> = Mutex::new(RefCell::new(QueueBuf::new()));

impl EusciSPIPriv for pac::E_USCI_B0 {
    #[inline]
    fn get_rx_buf() -> &'static Mutex<RefCell<QueueBuf>>{
        & SPI_RX_BUF_B0
    }

    #[inline]
    fn get_tx_buf() -> &'static Mutex<RefCell<QueueBuf>>{
        & SPI_TX_BUF_B0
    }

}

impl EusciSPIInterrupter for pac::E_USCI_B0 {
    #[inline]
    unsafe fn spi_interrupt(){
        spi_interrupt_shared(pac::E_USCI_B0::steal());
    }
}

impl EUsciSPIBus for pac::E_USCI_B0 {
    type MISO = UsciB0MISOPin;
    type MOSI = UsciB0MOSIPin;
    type SCLK = UsciB0SCLKPin;
    type STE  = UsciB0STEPin;
}

static SPI_RX_BUF_B1: Mutex<RefCell<QueueBuf>> = Mutex::new(RefCell::new(QueueBuf::new()));
static SPI_TX_BUF_B1: Mutex<RefCell<QueueBuf>> = Mutex::new(RefCell::new(QueueBuf::new()));

impl EusciSPIPriv for pac::E_USCI_B1 {
    #[inline]
    fn get_rx_buf() -> &'static Mutex<RefCell<QueueBuf>>{
        & SPI_RX_BUF_B1
    }

    #[inline]
    fn get_tx_buf() -> &'static Mutex<RefCell<QueueBuf>>{
        & SPI_TX_BUF_B1
    }

}

impl EusciSPIInterrupter for pac::E_USCI_B1 {
    #[inline]
    unsafe fn spi_interrupt(){
        spi_interrupt_shared(pac::E_USCI_B1::steal());
    }
}

impl EUsciSPIBus for pac::E_USCI_B1 {
    type MISO = UsciB1MISOPin;
    type MOSI = UsciB1MOSIPin;
    type SCLK = UsciB1SCLKPin;
    type STE  = UsciB1STEPin;
}

/// SPI MISO pin for eUSCI A0
pub struct UsciA0MISOPin;
impl<DIR> Into<UsciA0MISOPin> for Pin<P1, Pin7, Alternate1<DIR>> {
    #[inline(always)]
    fn into(self) -> UsciA0MISOPin {
        UsciA0MISOPin
    }
}

/// SPI MOSI pin for eUSCI A0
pub struct UsciA0MOSIPin;
impl<DIR> Into<UsciA0MOSIPin> for Pin<P1, Pin6, Alternate1<DIR>> {
    #[inline(always)]
    fn into(self) -> UsciA0MOSIPin {
        UsciA0MOSIPin
    }
}

/// SPI SCLK pin for eUSCI A0
pub struct UsciA0SCLKPin;
impl<DIR> Into<UsciA0SCLKPin> for Pin<P1, Pin5, Alternate1<DIR>> {
    #[inline(always)]
    fn into(self) -> UsciA0SCLKPin {
        UsciA0SCLKPin
    }
}

/// SPI STE pin for eUSCI A0
pub struct UsciA0STEPin;
impl<DIR> Into<UsciA0STEPin> for Pin<P1, Pin4, Alternate1<DIR>> {
    #[inline(always)]
    fn into(self) -> UsciA0STEPin {
        UsciA0STEPin
    }
}

/// SPI MISO pin for eUSCI A1
pub struct UsciA1MISOPin;
impl<DIR> Into<UsciA1MISOPin> for Pin<P4, Pin3, Alternate1<DIR>> {
    #[inline(always)]
    fn into(self) -> UsciA1MISOPin {
        UsciA1MISOPin
    }
}

/// SPI MOSI pin for eUSCI A1
pub struct UsciA1MOSIPin;
impl<DIR> Into<UsciA1MOSIPin> for Pin<P4, Pin2, Alternate1<DIR>> {
    #[inline(always)]
    fn into(self) -> UsciA1MOSIPin {
        UsciA1MOSIPin
    }
}

/// SPI SCLK pin for eUSCI A1
pub struct UsciA1SCLKPin;
impl<DIR> Into<UsciA1SCLKPin> for Pin<P4, Pin1, Alternate1<DIR>> {
    #[inline(always)]
    fn into(self) -> UsciA1SCLKPin {
        UsciA1SCLKPin
    }
}

/// SPI STE pin for eUSCI A1
pub struct UsciA1STEPin;
impl<DIR> Into<UsciA1STEPin> for Pin<P4, Pin0, Alternate1<DIR>> {
    #[inline(always)]
    fn into(self) -> UsciA1STEPin {
        UsciA1STEPin
    }
}

/// SPI MISO pin for eUSCI B0
pub struct UsciB0MISOPin;
impl<DIR> Into<UsciB0MISOPin> for Pin<P1, Pin3, Alternate1<DIR>> {
    #[inline(always)]
    fn into(self) -> UsciB0MISOPin {
        UsciB0MISOPin
    }
}

/// SPI MOSI pin for eUSCI B0
pub struct UsciB0MOSIPin;
impl<DIR> Into<UsciB0MOSIPin> for Pin<P1, Pin2, Alternate1<DIR>> {
    #[inline(always)]
    fn into(self) -> UsciB0MOSIPin {
        UsciB0MOSIPin
    }
}

/// SPI SCLK pin for eUSCI B0
pub struct UsciB0SCLKPin;
impl<DIR> Into<UsciB0SCLKPin> for Pin<P1, Pin1, Alternate1<DIR>> {
    #[inline(always)]
    fn into(self) -> UsciB0SCLKPin {
        UsciB0SCLKPin
    }
}

/// SPI STE pin for eUSCI B0
pub struct UsciB0STEPin;
impl<DIR> Into<UsciB0STEPin> for Pin<P1, Pin0, Alternate1<DIR>> {
    #[inline(always)]
    fn into(self) -> UsciB0STEPin {
        UsciB0STEPin
    }
}

/// SPI MISO pin for eUSCI B1
pub struct UsciB1MISOPin;
impl<DIR> Into<UsciB1MISOPin> for Pin<P4, Pin7, Alternate1<DIR>> {
    #[inline(always)]
    fn into(self) -> UsciB1MISOPin {
        UsciB1MISOPin
    }
}

/// SPI MOSI pin for eUSCI B1
pub struct UsciB1MOSIPin;
impl<DIR> Into<UsciB1MOSIPin> for Pin<P4, Pin6, Alternate1<DIR>> {
    #[inline(always)]
    fn into(self) -> UsciB1MOSIPin {
        UsciB1MOSIPin
    }
}

/// SPI SCLK pin for eUSCI B1
pub struct UsciB1SCLKPin;
impl<DIR> Into<UsciB1SCLKPin> for Pin<P4, Pin5, Alternate1<DIR>> {
    #[inline(always)]
    fn into(self) -> UsciB1SCLKPin {
        UsciB1SCLKPin
    }
}

/// SPI STE pin for eUSCI B1
pub struct UsciB1STEPin;
impl<DIR> Into<UsciB1STEPin> for Pin<P4, Pin4, Alternate1<DIR>> {
    #[inline(always)]
    fn into(self) -> UsciB1STEPin {
        UsciB1STEPin
    }
}

/// Struct used to configure a SPI bus
pub struct SPIBusConfig<USCI: EUsciSPIBus>{
    usci: USCI,
    prescaler: u16,

    // Register configs
    ctlw0: UcxSpiCtw0,
}

impl<USCI: EUsciSPIBus> SPIBusConfig<USCI>{
    /// Create a new configuration for setting up a EUSCI peripheral in SPI mode
    pub fn new(usci: USCI, mode:Mode, msbFirst:bool)->Self{
        let ctlw0 = UcxSpiCtw0{
            ucckph: match mode.phase {
                Phase::CaptureOnFirstTransition => true,
                Phase::CaptureOnSecondTransition => false,
            },
            ucckpl: match mode.polarity {
                Polarity::IdleLow => false,
                Polarity::IdleHigh => true,
            },
            ucmsb: msbFirst,
            uc7bit: false,
            ucmst: true,
            ucsync: true,
            ucstem: true,
            ucswrst: true,
            ucmode:  Ucmode::FourPinSPI0,
            ucssel: Ucssel::Uclk,
        };


        SPIBusConfig{
            usci: usci,
            prescaler: 0,
            ctlw0: ctlw0
        }
    }

    /// Configures this peripheral to use smclk
    #[inline]
    pub fn use_smclk(&mut self, _smclk:&Smclk, clk_divisor:u16){
        self.ctlw0.ucssel = Ucssel::Smclk;
        self.prescaler = clk_divisor;
    }

    /// Configures this peripheral to use aclk
    #[inline]
    pub fn use_aclk(&mut self, _aclk:&Aclk, clk_divisor:u16){
        self.ctlw0.ucssel = Ucssel::Aclk;
        self.prescaler = clk_divisor;
    }

    /// Performs hardware configuration and creates an SPI bus
    pub fn spi_pins
        <SO: Into<USCI::MISO>, SI: Into<USCI::MOSI>, CLK: Into<USCI::SCLK>, STE: Into<USCI::STE>>
        (&mut self, _miso: SO, _mosi: SI, _sclk: CLK, _cs : STE)
        -> SPIPins<USCI>{
        self.configure_hw();
        SPIPins(PhantomData)
    }

    #[inline]
    fn configure_hw(&self){
        self.usci.ctw0_wr_rst(true);

        self.usci.ctw0_wr(&self.ctlw0);
        self.usci.brw_wr(self.prescaler);
        self.usci.uclisten_set(false);

        self.usci.ctw0_wr_rst(false);

        self.usci.transmit_interrupt_set(false);
        self.usci.receive_interrupt_set(true);
    }

}

/// Represents a group of pins configured for SPI communication
pub struct SPIPins<USCI: EUsciSPIBus>(PhantomData<USCI>);




/// SPI transmit/receive errors
#[derive(Clone, Copy)]
pub enum SPIErr{
    /// Function not implemented
    Unimplemented = 0,
}

#[inline]
fn spi_interrupt_shared<USCI: EUsciSPIBus> (usci: USCI){
    with(|cs| {
        if usci.receive_flag() {
            let rx_buf = &mut *USCI::get_rx_buf().borrow_ref_mut(cs);
            rx_buf.put(usci.rxbuf_rd());
        }
        if usci.transmit_flag() {
            let tx_buf = &mut *USCI::get_tx_buf().borrow_ref_mut(cs);
            if tx_buf.has_data() {
                usci.txbuf_wr(tx_buf.get());
                if tx_buf.is_empty(){
                    usci.transmit_interrupt_set(false);
                    return;
                }
                // usci.rxbuf_rd(); //dummy read
            }else{
                usci.transmit_interrupt_set(false);
            }
        }
    });
}

impl<USCI:EUsciSPIBus> hal::blocking::spi::Write<u8> for SPIPins<USCI>{
    type Error = SPIErr;

    fn write(&mut self, words: &[u8]) -> Result<(), SPIErr> {
        for word in words {
            nb::block!(self.send(*word))?;
            nb::block!(self.read())?;
        }
        Ok(())
    }
}

impl<USCI:EUsciSPIBus> FullDuplex<u8> for SPIPins<USCI>{
    type Error = SPIErr;
    fn read(&mut self) -> nb::Result<u8, Self::Error>{
        with(|cs| {
            let rx_buf = &mut *USCI::get_rx_buf().borrow_ref_mut(cs);
            if rx_buf.has_data() {
                Ok(rx_buf.get())
            }else{
                Err(WouldBlock)
            }
        })
    }

    fn send(&mut self, word: u8) -> nb::Result<(), Self::Error>{
        with(|cs| {
            let tx_buf = &mut *USCI::get_tx_buf().borrow_ref_mut(cs);
            if tx_buf.is_full() {
                Err(WouldBlock)
            }else{
                if tx_buf.is_empty() {
                    let usci = unsafe {USCI::steal()};
                    usci.transmit_interrupt_set(true);
                }
                Ok(tx_buf.put(word))
            }
        })
    }
}












