// src/kernel/init.rs

use crate::boot::boot_info::{BootConfig, BootInfo};
use crate::gfx::renderer::{MAX_HEIGHT, MAX_WIDTH};

/// Inicialização mais cedo possível (antes de qualquer subsistema)
pub fn early_init(info: &BootInfo) {
    crate::kernel::console::init();

    crate::log!("BOOT", "Kernel start");
    crate::log!("BOOT", "UART ready");

    match info.dtb {
        Some(dtb) => crate::log!("BOOT", "DTB ptr: 0x{:016X}", dtb.as_ptr() as u64),
        None => crate::log!("BOOT", "DTB ptr: none"),
    }

    match info.cmdline {
        Some(args) => crate::log!("BOOT", "bootargs: {}", args),
        None => crate::log!("BOOT", "No bootargs, using defaults"),
    }
}

/// Normaliza configuração de vídeo
pub fn normalize_config(mut config: BootConfig) -> BootConfig {
    if config.depth != 32 {
        crate::log!("BOOT", "Forcing depth to 32bpp");
        config.depth = 32;
    }

    if config.width == 0 {
        config.width = 320;
    }

    if config.height == 0 {
        config.height = 240;
    }

    if config.width as usize > MAX_WIDTH {
        config.width = MAX_WIDTH as u32;
    }

    if config.height as usize > MAX_HEIGHT {
        config.height = MAX_HEIGHT as u32;
    }

    config
}

/// Inicializa infraestrutura de tasks (scheduler, etc)
pub fn init_tasking() {
    crate::kernel::scheduler::init();
    crate::log!("BOOT", "Scheduler initialized");
}

/// Futuro: inicialização de drivers básicos
#[allow(dead_code)]
pub fn init_drivers() {
    crate::log!("BOOT", "Drivers init (stub)");
}

/// Futuro: inicialização de interrupções
#[allow(dead_code)]
pub fn init_interrupts() {
    crate::log!("BOOT", "Interrupts init (stub)");
}