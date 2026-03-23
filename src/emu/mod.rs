// src/emu/mod.rs

mod host;
pub mod task;

use crate::drivers::framebuffer::Framebuffer;
use crate::platform::raspi3::bootargs;

extern "C" {
    fn omega_init();
    fn omega_run_frame();
    fn FloppyInsert(number: i32, adf: *mut u8);
    fn omega_probe_dump(last_n: u32);
}

/// Dump the last `n` probe events to serial (callable from Rust).
pub fn probe_dump(n: u32) {
    unsafe { omega_probe_dump(n) };
}

// Buffers físicos fixos para os ADFs (880 KB cada, fora do kernel)
// ADF padrão: 80 cilindros × 2 lados × 11 setores × 512 bytes = 901120 bytes
const ADF_SIZE: usize = 901_120;
const DF0_ADDR: usize = 0x0200_0000; // 32 MB mark
const DF1_ADDR: usize = 0x0210_0000; // 33 MB mark

// Buffer para ROM carregada do SD card (512 KB = Kickstart padrão)
const ROM_SIZE:  usize = 512 * 1024;
const ROM_ADDR:  usize = 0x0220_0000; // 34 MB mark

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

    // 1. Carrega ROM do SD card (se especificada), antes de omega_init().
    if let Some(name) = bootargs::rom() {
        crate::log!("EMU", "rom: loading '{}'", name);
        let buf = unsafe { core::slice::from_raw_parts_mut(ROM_ADDR as *mut u8, ROM_SIZE) };
        let n = crate::fs::fat32::load(name, buf);
        if n > 0 {
            crate::log!("EMU", "rom: {} bytes loaded", n);
            host::set_rom(ROM_ADDR as *const u8, n);
        } else {
            crate::log!("EMU", "rom: load failed, using built-in");
        }
    }

    // 2. Inicializa o emulador: carrega ROM, inicializa chipset e slots de disco.
    let mut emu = OmegaEmu::new();

    // 3. Lê ADFs do SD e insere nos slots — após FloppyInit(), ordem natural.
    //    EMMC já foi inicializado em kernel_main; read_blocks falha silenciosamente
    //    se o controlador não estiver disponível.
    if let Some(name) = bootargs::df0() {
        if read_adf(0, name, DF0_ADDR) {
            unsafe { FloppyInsert(0, DF0_ADDR as *mut u8); }
        }
    }
    if let Some(name) = bootargs::df1() {
        if read_adf(1, name, DF1_ADDR) {
            unsafe { FloppyInsert(1, DF1_ADDR as *mut u8); }
        }
    }

    // 4. Spawna USB após o SD para evitar contenção de IRQ durante a leitura.
    let _ = crate::kernel::scheduler::spawn("usb", crate::drivers::usb::usb_task);

    loop {
        emu.run_frame();
    }
}
