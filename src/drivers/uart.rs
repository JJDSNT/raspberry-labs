// src/drivers/uart.rs

use core::fmt::{self, Write};

use crate::platform::raspi3::mmio::{read, write};

// Offsets relativos ao base address
const UART_DR: usize = 0x00;
const UART_FR: usize = 0x18;
const UART_IBRD: usize = 0x24;
const UART_FBRD: usize = 0x28;
const UART_LCRH: usize = 0x2C;
const UART_CR: usize = 0x30;
const UART_ICR: usize = 0x44;

const FR_TXFF: u32 = 1 << 5;

pub struct Uart {
    base: usize,
}

impl Uart {
    pub const fn new(base: usize) -> Self {
        Self { base }
    }

    #[inline(always)]
    fn reg(&self, offset: usize) -> usize {
        self.base + offset
    }

    pub fn init(&self) {
        // disable UART
        write(self.reg(UART_CR), 0x0000);

        // clear interrupts
        write(self.reg(UART_ICR), 0x07FF);

        // baud rate (115200 @ 48MHz)
        write(self.reg(UART_IBRD), 26);
        write(self.reg(UART_FBRD), 3);

        // 8N1 + FIFO enable
        write(self.reg(UART_LCRH), (1 << 4) | (3 << 5));

        // enable UART, TX, RX
        write(self.reg(UART_CR), (1 << 0) | (1 << 8) | (1 << 9));
    }

    pub fn putc(&self, c: u8) {
        while read(self.reg(UART_FR)) & FR_TXFF != 0 {}
        write(self.reg(UART_DR), c as u32);
    }

    pub fn write_str_raw(&self, s: &str) {
        for b in s.bytes() {
            if b == b'\n' {
                self.putc(b'\r');
            }
            self.putc(b);
        }
    }

    pub fn write_hex(&self, value: u32) {
        let hex_chars = b"0123456789ABCDEF";
        for i in (0..8).rev() {
            let shift = i * 4;
            let digit = ((value >> shift) & 0xF) as usize;
            self.putc(hex_chars[digit]);
        }
    }

    pub fn write_dec(&self, mut value: u32) {
        let mut buffer = [0u8; 10];
        let mut i = 0;

        if value == 0 {
            self.putc(b'0');
            return;
        }

        while value > 0 {
            buffer[i] = b'0' + (value % 10) as u8;
            value /= 10;
            i += 1;
        }

        while i > 0 {
            i -= 1;
            self.putc(buffer[i]);
        }
    }

    pub fn log_raw(&self, tag: &str, msg: &str) {
        self.write_str_raw("[");
        self.write_str_raw(tag);
        self.write_str_raw("] ");
        self.write_str_raw(msg);
        self.write_str_raw("\n");
    }
}

impl Write for Uart {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_str_raw(s);
        Ok(())
    }
}