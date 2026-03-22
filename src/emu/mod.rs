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

/// Lê um ADF do SD card para o buffer em `addr`.
/// Retorna true se carregou com sucesso.
fn read_adf(drive: i32, name: &str, addr: usize) -> bool {
    crate::log!("EMU", "df{}: loading '{}'", drive, name);
    let buf = unsafe { core::slice::from_raw_parts_mut(addr as *mut u8, ADF_SIZE) };
    let n = crate::fs::fat32::load(name, buf);
    if n > 0 {
        crate::log!("EMU", "df{}: {} bytes loaded", drive, n);
        true
    } else {
        crate::log!("EMU", "df{}: load failed", drive);
        false
    }
}

/// Ponto de entrada principal — chamado por run_demo() com o framebuffer real.
pub fn run(fb: Framebuffer) -> ! {
    host::set_framebuffer(fb.ptr as *mut u32, fb.pitch as i32);

    // 1. Lê ADFs do SD para buffers fixos na RAM (antes do init para usar o SD
    //    enquanto o controlador ainda está exclusivo desta task).
    let df0_loaded = crate::platform::raspi3::emmc::init()
        .then(|| {
            let a = bootargs::df0().map(|n| read_adf(0, n, DF0_ADDR)).unwrap_or(false);
            let b = bootargs::df1().map(|n| read_adf(1, n, DF1_ADDR)).unwrap_or(false);
            (a, b)
        })
        .unwrap_or((false, false));

    // 2. Inicializa o emulador — carrega ROM, chama FloppyInit() que zera
    //    os slots de disco. Os dados ADF já estão nos buffers RAM.
    let _ = crate::kernel::scheduler::spawn("usb", crate::drivers::usb::usb_task);
    let mut emu = OmegaEmu::new();

    // 3. Insere os discos APÓS o init para que FloppyInit() não os descarte.
    if df0_loaded.0 { unsafe { FloppyInsert(0, DF0_ADDR as *mut u8); } }
    if df0_loaded.1 { unsafe { FloppyInsert(1, DF1_ADDR as *mut u8); } }

    loop {
        emu.run_frame();
    }
}
