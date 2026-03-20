// src/kernel/console.rs

use core::fmt::{self, Write};

use crate::drivers::uart::Uart;
use crate::platform::raspi3::memory_map::UART0_BASE;

#[inline(always)]
fn early_uart() -> Uart {
    Uart::new(UART0_BASE)
}

pub fn init() {
    early_uart().init();
}

pub fn _print(args: fmt::Arguments) {
    let mut uart = early_uart();
    let _ = uart.write_fmt(args);
}

pub fn log_raw(tag: &str, msg: &str) {
    early_uart().log_raw(tag, msg);
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {{
        $crate::kernel::console::_print(format_args!($($arg)*));
    }};
}

#[macro_export]
macro_rules! println {
    () => {{
        $crate::print!("\n");
    }};
    ($fmt:expr) => {{
        $crate::print!(concat!($fmt, "\n"));
    }};
    ($fmt:expr, $($arg:tt)*) => {{
        $crate::print!(concat!($fmt, "\n"), $($arg)*);
    }};
}

#[macro_export]
macro_rules! log {
    ($tag:expr, $fmt:expr) => {{
        $crate::println!(concat!("[", $tag, "] ", $fmt));
    }};
    ($tag:expr, $fmt:expr, $($arg:tt)*) => {{
        $crate::println!(concat!("[", $tag, "] ", $fmt), $($arg)*);
    }};
}