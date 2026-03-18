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
    log!("BOOT", "UART ready");
    log!("BOOT", "Initializing framebuffer...");

    match Framebuffer::init(1024, 768, 32) {
        Some(mut fb) => {
            log!("BOOT", "Framebuffer ready");
            log!("BOOT", "Resolution: {}x{}", fb.width, fb.height);
            log!("BOOT", "Pitch: {}", fb.pitch);
            log!("BOOT", "Depth: {}", fb.depth);
            log!("BOOT", "RGB order: {}", fb.isrgb);

            fb.draw_gradient();

            let white = fb.color_rgb(255, 255, 255);
            let red = fb.color_rgb(255, 0, 0);
            let green = fb.color_rgb(0, 255, 0);
            let blue = fb.color_rgb(0, 0, 255);

            fb.fill_rect(40, 40, 120, 80, white);
            fb.fill_rect(200, 40, 120, 80, red);
            fb.fill_rect(360, 40, 120, 80, green);
            fb.fill_rect(520, 40, 120, 80, blue);

            fb.put_pixel(fb.width / 2, fb.height / 2, white);

            log!("BOOT", "Initial test frame drawn");
            log!("BOOT", "Entering idle loop");
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
fn panic(info: &PanicInfo) -> ! {
    log!("PANIC", "Kernel panic");

    if let Some(location) = info.location() {
        log!(
            "PANIC",
            "at {}:{}:{}",
            location.file(),
            location.line(),
            location.column()
        );
    }

    loop {
        core::hint::spin_loop();
    }
}