// src/drivers/usb.rs
//
// Integração do TinyUSB com o kernel.
//

use crate::arch::aarch64::regs::{cache, CntFrq, CntPct};
use crate::kernel::time;
use crate::platform::raspi3::interrupts;

// ---------------------------------------------------------------------------
// FFI — nomes corretos da versão atual do TinyUSB
// ---------------------------------------------------------------------------

unsafe extern "C" {
    /// Inicializa o TinyUSB para um rhport específico.
    /// Substitui o antigo tusb_init().
    fn tusb_rhport_init(rhport: u8, config: *const core::ffi::c_void) -> bool;

    /// Processa eventos USB pendentes.
    /// Substitui o antigo tuh_task().
    fn tuh_task_ext(timeout_ms: u32, in_isr: bool);

    /// Handler unificado de IRQ USB.
    /// Substitui o antigo hcd_int_handler().
    fn tusb_int_handler(rhport: u8, in_isr: bool);
}

// ---------------------------------------------------------------------------
// Funções exportadas para o HAL C
// ---------------------------------------------------------------------------

#[unsafe(no_mangle)]
pub extern "C" fn kernel_time_ms() -> u64 {
    time::ticks() * 10
}

#[unsafe(no_mangle)]
pub extern "C" fn kernel_delay_us(us: u32) {
    let freq = CntFrq::read();
    let cycles = (freq / 1_000_000) * us as u64;
    let start = CntPct::read();
    while CntPct::read().wrapping_sub(start) < cycles {
        core::hint::spin_loop();
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn kernel_cache_flush(start: usize, end: usize) {
    cache::flush_range(start, end);
}

#[unsafe(no_mangle)]
pub extern "C" fn mailbox_usb_power(on: i32) -> i32 {
    if usb_power_via_mailbox(on != 0) { 0 } else { -1 }
}

// ---------------------------------------------------------------------------
// Power USB via mailbox
// ---------------------------------------------------------------------------

fn usb_power_via_mailbox(on: bool) -> bool {
    #[repr(align(16))]
    struct MboxBuf([u32; 8]);

    let state: u32 = if on { 0x3 } else { 0x2 };

    let mut buf = MboxBuf([
        8 * 4,
        0x0000_0000,
        0x0002_8001, // Set Power State
        8,
        0,
        0x0000_0003, // USB HCD
        state,
        0x0000_0000,
    ]);

    let ok = crate::platform::raspi3::mailbox::mailbox_call(8, buf.0.as_mut_ptr());

    crate::log!("USB", "power {} mailbox={}", if on { "on" } else { "off" }, if ok { "ok" } else { "FAILED" });

    ok
}

// ---------------------------------------------------------------------------
// Inicialização
// ---------------------------------------------------------------------------

pub fn init() {
    crate::log!("USB", "initializing");

    if !usb_power_via_mailbox(true) {
        crate::log!("USB", "power on failed");
        return;
    }

    interrupts::vic_enable_usb_irq();
    crate::log!("USB", "IRQ enabled");

    // config = NULL usa configuração padrão do tusb_config.h
    let ok = unsafe { tusb_rhport_init(0, core::ptr::null()) };

    if ok {
        crate::log!("USB", "TinyUSB initialized");
    } else {
        crate::log!("USB", "TinyUSB init FAILED");
    }
}

// ---------------------------------------------------------------------------
// Task USB
// ---------------------------------------------------------------------------

pub fn usb_task() {
    crate::log!("USB", "task started");

    loop {
        // timeout_ms=0 → não bloqueia, processa eventos pendentes e retorna
        unsafe { tuh_task_ext(0, false) };
        crate::kernel::scheduler::yield_now();
    }
}

// ---------------------------------------------------------------------------
// Handler de IRQ
// ---------------------------------------------------------------------------

pub fn handle_irq() {
    unsafe { tusb_int_handler(0, true) };
}