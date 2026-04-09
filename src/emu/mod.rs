// src/emu/mod.rs

mod host;
mod launcher;
pub mod task;

use crate::drivers::framebuffer::Framebuffer;
use crate::platform::raspi3::bootargs;

extern "C" {
    fn omega_init();
    fn omega_run_frame();
    fn omega_attach_hdf(path: *const u8) -> i32;
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

// Buffer para ROM carregada do SD card.
// 1 MB para acomodar AROS (ext 512KB + main 512KB concatenados);
// Kickstart 1.2 / 1.3 usa apenas os primeiros 512 KB.
const ROM_SIZE: usize = 1024 * 1024;
const ROM_ADDR: usize = 0x0220_0000; // 34 MB mark

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

/// Converte &str para C string temporária em stack/local buffer.
/// Retorna ponteiro para buffer terminado em NUL, ou None se não couber
/// ou se contiver NUL embutido.
fn with_c_path<F, R>(path: &str, f: F) -> Option<R>
where
    F: FnOnce(*const u8) -> R,
{
    const MAX_PATH: usize = 256;

    if path.is_empty() || path.len() >= MAX_PATH {
        return None;
    }

    let bytes = path.as_bytes();

    for &b in bytes {
        if b == 0 {
            return None;
        }
    }

    let mut buf = [0u8; MAX_PATH];
    let len = bytes.len();

    buf[..len].copy_from_slice(bytes);
    buf[len] = 0;

    Some(f(buf.as_ptr()))
}

/// Ponto de entrada principal — chamado por run_demo() com o framebuffer real.
pub fn run(fb: Framebuffer) -> ! {
    // Grava ptr/pitch para o emulador usar depois (via omega_host_framebuffer).
    host::set_framebuffer(fb.ptr as *mut u32, fb.pitch as i32);

    crate::log!("EMU", "starting — sd_ok={}", crate::drivers::sdcard::is_ready());

    // 1. ROM — bootarg "rom=" ou built-in AROS.
    if let Some(name) = bootargs::rom() {
        crate::log!("EMU", "rom: loading '{}'", name);
        let buf = unsafe { core::slice::from_raw_parts_mut(ROM_ADDR as *mut u8, ROM_SIZE) };
        let n = crate::fs::fat32::load(name, buf);
        if n > 0 {
            crate::log!("EMU", "rom: {} bytes loaded", n);
            host::set_rom(ROM_ADDR as *const u8, n);
        } else {
            crate::log!("EMU", "rom: '{}' not found on SD, using built-in", name);
        }
    }

    // 2. Resolve seleção de disco/HDF.
    //    Prioridade: bootargs pré-configurados (Go launcher / cmdline.txt)
    //    Fallback: launcher interativo bare-metal (teclado USB necessário).
    let (df0_name, df1_name, hd0_name) = resolve_media(fb);

    // 3. Inicializa o emulador.
    let mut emu = OmegaEmu::new();

    // 4. Anexa HDF.
    if let Some(name) = hd0_name {
        crate::log!("EMU", "hd0: attaching '{}'", name);
        match with_c_path(name, |ptr| unsafe { omega_attach_hdf(ptr) }) {
            Some(0) => crate::log!("EMU", "hd0: attached"),
            Some(rc) => crate::log!("EMU", "hd0: attach failed rc={}", rc),
            None     => crate::log!("EMU", "hd0: invalid path"),
        }
    }

    // 5. Carrega e insere ADFs.
    if let Some(name) = df0_name {
        if read_adf(0, name, DF0_ADDR) {
            unsafe { FloppyInsert(0, DF0_ADDR as *mut u8); }
        }
    }
    if let Some(name) = df1_name {
        if read_adf(1, name, DF1_ADDR) {
            unsafe { FloppyInsert(1, DF1_ADDR as *mut u8); }
        }
    }

    loop {
        emu.run_frame();
    }
}

// ---------------------------------------------------------------------------
// Resolução de mídia
// ---------------------------------------------------------------------------

/// Retorna (df0, df1, hd0) como Option<&'static str>.
///
/// Caminho 1 — bootargs pré-configurados (Go launcher ou cmdline.txt):
///   df0=/hd0= já estão definidos; spawna USB e usa esses valores.
///
/// Caminho 2 — launcher interativo (bare-metal, teclado USB necessário):
///   Quando nenhum bootarg de disco está definido, mostra o launcher na tela,
///   spawna USB para receber teclas e aguarda confirmação do utilizador.
fn resolve_media(
    fb: Framebuffer,
) -> (Option<&'static str>, Option<&'static str>, Option<&'static str>) {
    let pre_df0 = bootargs::df0();
    let pre_df1 = bootargs::df1();
    let pre_hd0 = bootargs::hd0();

    if pre_df0.is_some() || pre_df1.is_some() || pre_hd0.is_some() {
        // Bootargs configurados — usa direto, spawna USB para teclado no emu.
        let _ = crate::kernel::scheduler::spawn("usb", crate::drivers::usb::usb_task);
        return (pre_df0, pre_df1, pre_hd0);
    }

    // Launcher interativo: spawna USB primeiro (teclado), depois mostra UI.
    let _ = crate::kernel::scheduler::spawn("usb", crate::drivers::usb::usb_task);

    crate::log!("EMU", "launcher: no bootargs, showing selection UI");

    let cfg = launcher::run(fb);

    // Promove strings do resultado para 'static via buffers estáticos.
    static mut DF0_BUF: [u8; 64] = [0u8; 64];
    static mut DF1_BUF: [u8; 64] = [0u8; 64];
    static mut HD0_BUF: [u8; 64] = [0u8; 64];

    let df0 = promote_static(unsafe { &mut *core::ptr::addr_of_mut!(DF0_BUF) }, &cfg.df0, cfg.df0_len);
    let df1 = promote_static(unsafe { &mut *core::ptr::addr_of_mut!(DF1_BUF) }, &cfg.df1, cfg.df1_len);
    let hd0 = promote_static(unsafe { &mut *core::ptr::addr_of_mut!(HD0_BUF) }, &cfg.hd0, cfg.hd0_len);

    (df0, df1, hd0)
}

/// Copia `src[..len]` para `dst` e retorna Option<&'static str>.
fn promote_static(dst: &'static mut [u8; 64], src: &[u8; 64], len: usize) -> Option<&'static str> {
    if len == 0 { return None; }
    let l = len.min(64);
    dst[..l].copy_from_slice(&src[..l]);
    core::str::from_utf8(&dst[..l]).ok()
}