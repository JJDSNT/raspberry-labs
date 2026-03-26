// src/kernel/console.rs

use core::fmt::{self, Write};

use crate::drivers::uart::Uart;
use crate::platform::raspi3::memory_map::UART0_BASE;
use crate::platform::raspi3::mmio::{read, write};
use crate::platform::raspi3::peripheral::gpio::{
    FSEL_ALT0, GPFSEL1, GPPUD, GPPUDCLK0, PUD_OFF,
};

#[inline(always)]
fn early_uart() -> Uart {
    Uart::new(UART0_BASE)
}

fn init_gpio_uart() {
    // GPIO14 e GPIO15 ficam em GPFSEL1:
    // GPIO14 -> bits 14*3 % 30 = 12..14
    // GPIO15 -> bits 15*3 % 30 = 15..17
    let mut val = read(GPFSEL1);

    // limpa os 3 bits de função dos pinos 14 e 15
    val &= !((0b111 << 12) | (0b111 << 15));

    // ALT0 para TXD0/RXD0
    val |= (FSEL_ALT0 << 12) | (FSEL_ALT0 << 15);

    write(GPFSEL1, val);

    // Desabilita pull-up/down nos pinos 14 e 15
    write(GPPUD, PUD_OFF);

    for _ in 0..150 {
        core::hint::spin_loop();
    }

    write(GPPUDCLK0, (1 << 14) | (1 << 15));

    for _ in 0..150 {
        core::hint::spin_loop();
    }

    write(GPPUDCLK0, 0);
}

pub fn init() {
    init_gpio_uart();
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