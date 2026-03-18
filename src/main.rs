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
use crate::drivers::framebuffer::Framebuffer;
use crate::platform::uart::uart_init;
use crate::platform::dtb::Fdt;
use crate::platform::bootargs::apply_bootargs;

#[derive(Clone, Copy, Debug)]
pub enum DemoKind {
    Gradient,
    TestPattern,
    RasterBars,
}

#[derive(Clone, Copy, Debug)]
pub struct BootConfig {
    pub width:  u32,
    pub height: u32,
    pub depth:  u32,
    pub demo:   DemoKind,
}

impl BootConfig {
    pub const fn default() -> Self {
        Self {
            width:  1024,
            height: 768,
            depth:  32,
            demo:   DemoKind::Gradient,
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn kernel_main(dtb_ptr: usize) -> ! {
    uart_init();
    log!("BOOT", "Kernel start");
    log!("BOOT", "UART ready");
    log!("BOOT", "DTB ptr: 0x{:016X}", dtb_ptr as u64);

    // Começa com defaults e sobrescreve com o que vier do DTB.
    let mut config = BootConfig::default();

    // SAFETY: dtb_ptr vem diretamente do bootloader (x0 em boot.S).
    // O blob permanece mapeado para leitura durante todo o boot.
    let bootargs = unsafe {
        Fdt::from_ptr(dtb_ptr).and_then(|fdt| fdt.bootargs())
    };

    match bootargs {
        Some(args) => {
            log!("BOOT", "bootargs: {}", args);
            apply_bootargs(args, &mut config);
        }
        None => {
            log!("BOOT", "No bootargs (or no DTB), using defaults");
        }
    }

    log!(
        "BOOT",
        "Config: {}x{}x{}",
        config.width,
        config.height,
        config.depth
    );

    // fix: log! com ; interno não pode ficar em posição de expressão nua
    // dentro de match — envolve em bloco {}
    match config.demo {
        DemoKind::Gradient    => { log!("BOOT", "Selected demo: Gradient") }
        DemoKind::TestPattern => { log!("BOOT", "Selected demo: TestPattern") }
        DemoKind::RasterBars  => { log!("BOOT", "Selected demo: RasterBars") }
    }

    log!("BOOT", "Initializing framebuffer...");
    match Framebuffer::init(config.width, config.height, config.depth) {
        Some(mut fb) => {
            log!("BOOT", "Framebuffer ready");
            log!("BOOT", "Resolution: {}x{}", fb.width, fb.height);
            log!("BOOT", "Pitch: {}", fb.pitch);
            log!("BOOT", "Depth: {}", fb.depth);
            log!("BOOT", "RGB order: {}", fb.isrgb);
            run_selected_demo(&mut fb, config);
            log!("BOOT", "Initial frame drawn");
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

fn run_selected_demo(fb: &mut Framebuffer, config: BootConfig) {
    match config.demo {
        DemoKind::Gradient => {
            log!("DEMO", "Running gradient");
            fb.draw_gradient();
            let white = fb.color_rgb(255, 255, 255);
            let red   = fb.color_rgb(255,   0,   0);
            let green = fb.color_rgb(  0, 255,   0);
            let blue  = fb.color_rgb(  0,   0, 255);
            fb.fill_rect( 40, 40, 120, 80, white);
            fb.fill_rect(200, 40, 120, 80, red);
            fb.fill_rect(360, 40, 120, 80, green);
            fb.fill_rect(520, 40, 120, 80, blue);
            fb.put_pixel(fb.width / 2, fb.height / 2, white);
        }
        DemoKind::TestPattern => {
            log!("DEMO", "Running test pattern");
            fb.test_pattern();
        }
        DemoKind::RasterBars => {
            log!("DEMO", "RasterBars not wired in main yet");
            fb.test_pattern();
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
    // fix: info.message() não retorna Option desde Rust 1.73 — usa direto
    log!("PANIC", "message: {}", info.message());
    loop {
        core::hint::spin_loop();
    }
}