#![no_std]
#![no_main]

use core::arch::global_asm;
use core::panic::PanicInfo;

global_asm!(include_str!("arch/aarch64/boot.S"));

mod platform;

use crate::platform::uart::{uart_init, uart_write_str};

#[unsafe(no_mangle)]
pub extern "C" fn kernel_main() -> ! {
    uart_init();
    uart_write_str("Hello from Rust!\n");

    loop {
        core::hint::spin_loop();
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    uart_write_str("PANIC!\n");
    loop {
        core::hint::spin_loop();
    }
}