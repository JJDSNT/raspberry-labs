#![no_std]
#![no_main]

use core::arch::global_asm;
use core::panic::PanicInfo;

global_asm!(include_str!("arch/aarch64/boot.S"));

#[macro_use]
mod platform;
mod drivers;
mod demos;
mod gfx;
mod audio;
mod math;

use crate::demos::{run_demo, DemoKind};
use crate::drivers::framebuffer::Framebuffer;
use crate::gfx::renderer::{MAX_HEIGHT, MAX_WIDTH};
use crate::platform::bootargs::apply_bootargs;
use crate::platform::dtb::Fdt;
use crate::platform::uart::uart_init;

#[derive(Clone, Copy, Debug)]
pub struct BootConfig {
    pub width: u32,
    pub height: u32,
    pub depth: u32,
    pub demo: DemoKind,
}

impl BootConfig {
    pub const fn default() -> Self {
        Self {
            width: 1024,
            height: 768,
            depth: 32,
            demo: DemoKind::Gradient,
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn kernel_main(dtb_ptr: usize) -> ! {
    uart_init();
    log!("BOOT", "Kernel start");
    log!("BOOT", "UART ready");
    log!("BOOT", "DTB ptr: 0x{:016X}", dtb_ptr as u64);

    let mut config = BootConfig::default();

    let bootargs = unsafe { Fdt::from_ptr(dtb_ptr).and_then(|fdt| fdt.bootargs()) };

    match bootargs {
        Some(args) => {
            log!("BOOT", "bootargs: {}", args);
            apply_bootargs(args, &mut config);
        }
        None => {
            log!("BOOT", "No bootargs (or no DTB), using defaults");
        }
    }

    if config.depth != 32 {
        log!("BOOT", "Forcing depth to 32bpp");
        config.depth = 32;
    }

    if config.width == 0 {
        config.width = 320;
    }
    if config.height == 0 {
        config.height = 240;
    }

    if config.width as usize > MAX_WIDTH {
        log!(
            "BOOT",
            "Requested width {} exceeds MAX_WIDTH {}, clamping",
            config.width,
            MAX_WIDTH
        );
        config.width = MAX_WIDTH as u32;
    }

    if config.height as usize > MAX_HEIGHT {
        log!(
            "BOOT",
            "Requested height {} exceeds MAX_HEIGHT {}, clamping",
            config.height,
            MAX_HEIGHT
        );
        config.height = MAX_HEIGHT as u32;
    }

    log!(
        "BOOT",
        "Config: {}x{}x{}",
        config.width,
        config.height,
        config.depth
    );
    log!("BOOT", "Selected demo: {}", config.demo.as_str());

    log!("BOOT", "Initializing framebuffer...");
    match Framebuffer::init(config.width, config.height, config.depth) {
        Some(fb) => {
            log!("BOOT", "Framebuffer ready");
            log!("BOOT", "Resolution: {}x{}", fb.width, fb.height);
            log!("BOOT", "Pitch: {}", fb.pitch);
            log!("BOOT", "Depth: {}", fb.depth);
            log!("BOOT", "RGB order: {}", fb.isrgb);

            run_demo(config.demo, fb);
        }
        None => {
            log!("BOOT", "Framebuffer init failed");
            loop {
                core::hint::spin_loop();
            }
        }
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

    log!("PANIC", "message: {}", info.message());

    loop {
        core::hint::spin_loop();
    }
}