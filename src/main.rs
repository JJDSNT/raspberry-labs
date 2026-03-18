#![no_std]
#![no_main]

use core::arch::global_asm;
use core::panic::PanicInfo;

global_asm!(include_str!("arch/aarch64/boot.S"));

mod drivers;
mod platform;

use crate::drivers::framebuffer::Framebuffer;
use crate::platform::uart::{uart_init, uart_write_str};

#[unsafe(no_mangle)]
pub extern "C" fn kernel_main() -> ! {
    uart_init();
    uart_write_str("Booting...\n");

    match Framebuffer::init(1024, 768, 32) {
        Some(mut fb) => {
            uart_write_str("Framebuffer OK\n");
            fb.clear(0x0000FF00);
        }
        None => {
            uart_write_str("Framebuffer FAIL\n");
        }
    }

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