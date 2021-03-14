/* read and write SiFive's standard MMIO-based UART
 * 
 * This UART controller is used in various SiFive system-on-chips, such as:
 * FU740-C000, FU540-C000, FE310-G002
 * 
 * Datasheet: Chapter 13, SiFive FU540-C000 Manual
 * https://sifive.cdn.prismic.io/sifive%2F834354f0-08e6-423c-bf1f-0cb58ef14061_fu540-c000-v1.0.pdf
 * 
 * Follows the same API as mmio_16550_uart:
 * https://github.com/diodesign/mmio_16550_uart/blob/main/src/lib.rs
 * 
 * (c) Chris Williams, 2021.
 *
 * See README and LICENSE for usage and copying.
 */

/* we're on our own here */
#![cfg_attr(not(test), no_std)]
#![allow(dead_code)]

use core::ptr::{write_volatile, read_volatile};

const REG_TOTAL_SIZE: usize = 7 * 4; /* 7 x 32-bit registers */

/* registers in the controller and their offsets */
const REG_TXDATA: usize = 0 * 4; /* transmit data register */ 
const REG_RXDATA: usize = 1 * 4; /* receive data register */
const REG_TXCTRL: usize = 2 * 4; /* transmit control register */
const REG_RXCTRL: usize = 3 * 4; /* receive control register */
const REG_IE:     usize = 4 * 4; /* UART interrupt enable */
const REG_IP:     usize = 5 * 4; /* UART interrupt pending */
const REG_DIV:    usize = 6 * 4; /* baud rate divisor */

/* individual control bits */
const REG_IE_TXWM:      u32 = 1 << 0;  /* transmit watermark interrupt enable */
const REG_IE_RXWM:      u32 = 1 << 1;  /* receive watermark interrupt enable */
const REG_TXCTRL_TXEN:  u32 = 1 << 0;  /* transmit enable */
const REG_TXCTRL_TXCNT: u32 = 1 << 16; /* tx FIFO irq watermark level of 1 */
const REG_RXCTRL_RXEN:  u32 = 1 << 0;  /* receive enable */
const REG_RXCTRL_RXCNT: u32 = 6 << 16; /* rx FIFO irq watermark level of 6 */
const REG_TXDATA_FULL:  u32 = 1 << 31;
const REG_RXDATA_EMPTY: u32 = 1 << 31;

/* to avoid infinite loops, give up checking
   for a byte to arrive or for a byte to be
   transmitted after this many check iterations */
const LOOP_MAX: usize = 1000;

/* possible error conditions supported at this time */
#[derive(Debug)]
pub enum Fault
{
    TxNotEmpty,     /* gave up waiting to transmit */
    DataNotReady    /* gave up waiting to send */
}

#[derive(Debug)]
pub struct UART
{
    base_addr: usize
}

impl UART
{
    /* create and initialize a standard 8-n-1 UART object, or fail with a reason code.
    this used the previously configured baud rate, which is derived from the
    CPU core speed. the baud should be set separately */
    pub fn new(base_addr: usize) -> Result<Self, Fault>
    {
        let uart = UART { base_addr };

        /* enable transmission, one stop bit, set tx irq watermark.
           when the number of bytes to transmit drops below the
           watermark, raise an irq (if enabled) */
        uart.write_reg(REG_TXCTRL, REG_TXCTRL_TXCNT | REG_TXCTRL_TXEN);

        /* enable receive, set rx irq watermark.
           when the number of received bytes goes above the
           watermark, raise an irq (if enabled) */
        uart.write_reg(REG_RXCTRL, REG_RXCTRL_RXCNT | REG_RXCTRL_RXEN);

        Ok(uart)
    }

    /* enable or disable the tx watermark irqs */
    pub fn enable_tx_watermark_irq(&self, enable: bool)
    {
        let flags = self.read_reg(REG_IE);
        if enable == true
        {
            self.write_reg(REG_IE, flags | REG_IE_TXWM);
        }
        else
        {
            self.write_reg(REG_IE, flags & !REG_IE_TXWM);
        }
    }

    /* enable or disable the rx watermark irqs */
    pub fn enable_rx_watermark_irq(&self, enable: bool)
    {
        let flags = self.read_reg(REG_IE);
        if enable == true
        {
            self.write_reg(REG_IE, flags | REG_IE_RXWM);
        }
        else
        {
            self.write_reg(REG_IE, flags & !REG_IE_RXWM);
        }
    }

    /* set the divisor for the required baud given the bus frequency.
       baud and bus_freq are both in Hz */
    pub fn set_baud(&self, baud: u32, bus_freq: u32)
    {
        self.write_reg(REG_DIV, bus_freq / baud);
    }

    /* return size of this controller's MMIO space in bytes */
    pub fn size(&self) -> usize
    {
        REG_TOTAL_SIZE
    }

    /* centralize reading and writing of registers to these unsafe functions */
    fn write_reg(&self, reg: usize, val: u32)
    {
        /* assumes reg is in range */
        unsafe { write_volatile((self.base_addr + reg) as *mut u32, val) }
    }

    fn read_reg(&self, reg: usize) -> u32
    {
        /* assumes reg is in range */
        unsafe { read_volatile((self.base_addr + reg) as *const u32) }
    }

    pub fn send_byte(&self, to_send: u8) -> Result<(), Fault>
    {
        for _ in 0..LOOP_MAX
        {
            if self.is_transmit_full() == false
            {
                self.write_reg(REG_TXDATA, to_send as u32);
                return Ok(());
            }
        }

        Err(Fault::TxNotEmpty)
    }

    pub fn read_byte(&self) -> Result<u8, Fault>
    {
        for _ in 0..LOOP_MAX
        {
            if self.is_data_empty() == false
            {
                return Ok((self.read_reg(REG_RXDATA) & 0xff) as u8);
            }   
        }

        Err(Fault::DataNotReady)
    }

    /* return true if data can't be sent */
    fn is_transmit_full(&self) -> bool
    {
        let val = self.read_reg(REG_TXDATA);
        return val & REG_TXDATA_FULL != 0
    }

    /* return false if data is ready to be read */
    fn is_data_empty(&self) -> bool
    {
        let val = self.read_reg(REG_RXDATA);
        return val & REG_RXDATA_EMPTY != 0
    }
}

#[cfg(test)]
mod tests
{
    #[test]
    fn it_works()
    {
        assert_eq!(2 + 2, 4);
    }
}
