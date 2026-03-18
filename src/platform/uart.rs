use core::fmt::{self, Write};

use crate::platform::mmio::{read, write};

// ======================
// REGISTERS
// ======================

const UART0_BASE: usize = 0x3F201000;

const UART_DR: usize = UART0_BASE + 0x00;
const UART_FR: usize = UART0_BASE + 0x18;
const UART_IBRD: usize = UART0_BASE + 0x24;
const UART_FBRD: usize = UART0_BASE + 0x28;
const UART_LCRH: usize = UART0_BASE + 0x2C;
const UART_CR: usize = UART0_BASE + 0x30;
const UART_ICR: usize = UART0_BASE + 0x44;

const FR_TXFF: u32 = 1 << 5;

// ======================
// UART CORE
// ======================

pub fn uart_init() {
    // disable UART
    write(UART_CR, 0x0000);

    // clear interrupts
    write(UART_ICR, 0x07FF);

    // baud rate (115200 @ 48MHz)
    write(UART_IBRD, 26);
    write(UART_FBRD, 3);

    // 8N1 + FIFO enable
    write(UART_LCRH, (1 << 4) | (3 << 5));

    // enable UART, TX, RX
    write(UART_CR, (1 << 0) | (1 << 8) | (1 << 9));
}

pub fn uart_putc(c: u8) {
    while read(UART_FR) & FR_TXFF != 0 {}
    write(UART_DR, c as u32);
}

pub fn uart_write_str(s: &str) {
    for b in s.bytes() {
        if b == b'\n' {
            uart_putc(b'\r');
        }
        uart_putc(b);
    }
}

// ======================
// FORMATTING HELPERS
// ======================

pub fn uart_write_hex(value: u32) {
    let hex_chars = b"0123456789ABCDEF";

    for i in (0..8).rev() {
        let shift = i * 4;
        let digit = ((value >> shift) & 0xF) as usize;
        uart_putc(hex_chars[digit]);
    }
}

pub fn uart_write_dec(mut value: u32) {
    let mut buffer = [0u8; 10];
    let mut i = 0;

    if value == 0 {
        uart_putc(b'0');
        return;
    }

    while value > 0 {
        buffer[i] = b'0' + (value % 10) as u8;
        value /= 10;
        i += 1;
    }

    while i > 0 {
        i -= 1;
        uart_putc(buffer[i]);
    }
}

// ======================
// LOW LEVEL LOG (opcional)
// ======================

pub fn log_raw(tag: &str, msg: &str) {
    uart_write_str("[");
    uart_write_str(tag);
    uart_write_str("] ");
    uart_write_str(msg);
    uart_write_str("\n");
}

// ======================
// PRINTLN SUPPORT
// ======================

pub struct Uart;

impl Write for Uart {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        uart_write_str(s);
        Ok(())
    }
}

pub fn _print(args: fmt::Arguments) {
    let mut uart = Uart;
    let _ = uart.write_fmt(args);
}

// ======================
// MACROS
// ======================

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {
        $crate::platform::uart::_print(format_args!($($arg)*));
    };
}

#[macro_export]
macro_rules! println {
    () => {
        $crate::print!("\n");
    };
    ($fmt:expr) => {
        $crate::print!(concat!($fmt, "\n"));
    };
    ($fmt:expr, $($arg:tt)*) => {
        $crate::print!(concat!($fmt, "\n"), $($arg)*);
    };
}

// ======================
// HIGH LEVEL LOG MACRO
// ======================

#[macro_export]
macro_rules! log {
    ($tag:expr, $fmt:expr) => {
        $crate::println!(concat!("[", $tag, "] ", $fmt));
    };
    ($tag:expr, $fmt:expr, $($arg:tt)*) => {
        $crate::println!(concat!("[", $tag, "] ", $fmt), $($arg)*);
    };
}