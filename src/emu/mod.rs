// src/emu/mod.rs

mod host;
pub mod task;

use crate::drivers::framebuffer::Framebuffer;
use crate::platform::raspi3::bootargs;

extern "C" {
    fn omega_init();
    fn omega_run_frame();
    fn FloppyInsert(number: i32, adf: *mut u8);
}

// Buffers físicos fixos para os ADFs (880 KB cada, fora do kernel)
// ADF padrão: 80 cilindros × 2 lados × 11 setores × 512 bytes = 901120 bytes
const ADF_SIZE: usize = 901_120;
const DF0_ADDR: usize = 0x0200_0000; // 32 MB mark
const DF1_ADDR: usize = 0x0210_0000; // 33 MB mark

pub struct OmegaEmu {
    _private: (),
}

impl OmegaEmu {
    pub fn new() -> Self {
        unsafe { omega_init() };
        Self { _private: () }
    }

    pub fn run_frame(&mut self) {
        unsafe { omega_run_frame() };
    }
}

/// Carrega um ADF do SD card em `addr` e chama FloppyInsert.
fn load_adf(drive: i32, name: &str, addr: usize) {
    crate::log!("EMU", "df{}: loading '{}'", drive, name);
    let buf = unsafe { core::slice::from_raw_parts_mut(addr as *mut u8, ADF_SIZE) };
    let n = crate::fs::fat32::load(name, buf);
    if n > 0 {
        crate::log!("EMU", "df{}: {} bytes loaded", drive, n);
        unsafe { FloppyInsert(drive, addr as *mut u8); }
    } else {
        crate::log!("EMU", "df{}: load failed", drive);
    }
}

/// Ponto de entrada principal — chamado por run_demo() com o framebuffer real.
pub fn run(fb: Framebuffer) -> ! {
    host::set_framebuffer(fb.ptr as *mut u32, fb.pitch as i32);

    // Inicializa SD e carrega ADFs antes de iniciar o emulador
    if !crate::platform::raspi3::emmc::init() {
        crate::log!("EMU", "SD card init failed — rodando sem disco");
    } else {
        if let Some(name) = bootargs::df0() { load_adf(0, name, DF0_ADDR); }
        if let Some(name) = bootargs::df1() { load_adf(1, name, DF1_ADDR); }
    }

    // Spawna a task USB para que os callbacks HID sejam processados
    // enquanto o emulador roda.
    let _ = crate::kernel::scheduler::spawn("usb", crate::drivers::usb::usb_task);

    let mut emu = OmegaEmu::new();
    loop {
        emu.run_frame();
    }
}
