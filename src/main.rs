#![no_std]
#![no_main]

use core::arch::global_asm;
use core::panic::PanicInfo;

global_asm!(include_str!("arch/aarch64/boot.S"));

#[macro_use]
mod platform;
mod drivers;

use crate::drivers::framebuffer::Framebuffer;
use crate::platform::uart::uart_init;

#[unsafe(no_mangle)]
pub extern "C" fn kernel_main() -> ! {
    uart_init();

    log!("BOOT", "Kernel start");
    log!("BOOT", "Initializing framebuffer...");

    match Framebuffer::init(1024, 768, 32) {
        Some(mut fb) => {
            log!("BOOT", "Framebuffer ready");
            fb.clear(0x0000FF00);
        }
        None => {
            log!("BOOT", "Framebuffer init failed");
        }
    }

    loop {
        core::hint::spin_loop();
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    log!("PANIC", "Kernel panic");
    loop {
        core::hint::spin_loop();
    }
}