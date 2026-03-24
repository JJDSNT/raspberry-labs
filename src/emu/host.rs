// src/emu/host.rs
// Implementações Rust dos callbacks omega_host_* chamados pelo C.
//
// O ponteiro do framebuffer é configurado antes de iniciar o emulador
// via set_framebuffer().

use core::sync::atomic::{AtomicPtr, AtomicI32, AtomicUsize, Ordering};
use crate::kernel::sync::IrqSafeSpinLock;

static FB_PTR:   AtomicPtr<u32> = AtomicPtr::new(core::ptr::null_mut());
static FB_PITCH: AtomicI32      = AtomicI32::new(0);

static ROM_PTR:     AtomicPtr<u8>  = AtomicPtr::new(core::ptr::null_mut());
static ROM_SIZE:    AtomicUsize    = AtomicUsize::new(0);

pub fn set_rom(ptr: *const u8, size: usize) {
    ROM_PTR.store(ptr as *mut u8, Ordering::Release);
    ROM_SIZE.store(size, Ordering::Release);
}

#[no_mangle]
pub extern "C" fn omega_host_rom_ptr() -> *const u8 {
    ROM_PTR.load(Ordering::Acquire)
}

#[no_mangle]
pub extern "C" fn omega_host_rom_size() -> usize {
    ROM_SIZE.load(Ordering::Acquire)
}

// ---------------------------------------------------------------------------
// Ring buffer de eventos de teclado (USB HID → emulador)
// Produzido pelo callback TinyUSB (omega_host_push_key).
// Consumido pelo loop do emulador (omega_host_poll_key).
// ---------------------------------------------------------------------------

const KEY_RING_SIZE: usize = 64;

struct KeyRing {
    buf:  [(u8, u8); KEY_RING_SIZE], // (scancode, pressed)
    head: usize,
    tail: usize,
}

impl KeyRing {
    const fn new() -> Self {
        Self { buf: [(0, 0); KEY_RING_SIZE], head: 0, tail: 0 }
    }

    fn push(&mut self, scancode: u8, pressed: u8) {
        let next = (self.tail + 1) % KEY_RING_SIZE;
        if next != self.head {
            self.buf[self.tail] = (scancode, pressed);
            self.tail = next;
        }
        // silently drops events when full
    }

    fn pop(&mut self) -> Option<(u8, u8)> {
        if self.head == self.tail { return None; }
        let ev = self.buf[self.head];
        self.head = (self.head + 1) % KEY_RING_SIZE;
        Some(ev)
    }
}

static KEY_RING: IrqSafeSpinLock<KeyRing> = IrqSafeSpinLock::new(KeyRing::new());

/// Chamado pelo kernel antes de spawnar a task do emulador.
pub fn set_framebuffer(ptr: *mut u32, pitch: i32) {
    FB_PTR.store(ptr, Ordering::Release);
    FB_PITCH.store(pitch, Ordering::Release);
}

#[no_mangle]
pub extern "C" fn omega_host_framebuffer() -> *mut u32 {
    FB_PTR.load(Ordering::Acquire)
}

#[no_mangle]
pub extern "C" fn omega_host_pitch() -> i32 {
    FB_PITCH.load(Ordering::Acquire)
}

#[no_mangle]
pub extern "C" fn omega_host_vsync() {
    // Yield to kernel scheduler — gives other tasks a turn
    crate::kernel::scheduler::yield_now();
}

#[no_mangle]
pub extern "C" fn omega_host_log(msg: *const core::ffi::c_char) {
    if msg.is_null() { return; }
    // Convert C string and log via kernel
    let bytes = unsafe {
        let mut len = 0;
        while *msg.add(len) != 0 { len += 1; }
        core::slice::from_raw_parts(msg as *const u8, len)
    };
    if let Ok(s) = core::str::from_utf8(bytes) {
        crate::log!("EMU", "{}", s);
    }
}

/// Chamado pelo emulador com um par de amostras de áudio PCM estéreo.
/// No-op por enquanto — o driver PWM/I2S para Pi 3 ainda não está implementado.
#[no_mangle]
pub extern "C" fn omega_host_audio_sample(
    _left:  core::ffi::c_short,
    _right: core::ffi::c_short,
) {
    // TODO: feed into Pi 3 PWM audio ring buffer when audio driver is ready
}

/// Chamado pelo TinyUSB HID (omega_input.c) para enfileirar um evento de tecla.
#[no_mangle]
pub extern "C" fn omega_host_push_key(scancode: u8, pressed: core::ffi::c_int) {
    KEY_RING.lock().push(scancode, if pressed != 0 { 1 } else { 0 });
}

/// Chamado pelo emulador (omega_glue.c) para retirar um evento da fila.
/// Retorna 1 se havia evento, 0 se a fila estava vazia.
#[no_mangle]
pub extern "C" fn omega_host_poll_key(
    scancode: *mut u8,
    pressed: *mut core::ffi::c_int,
) -> core::ffi::c_int {
    match KEY_RING.lock().pop() {
        Some((sc, pr)) => {
            if !scancode.is_null() { unsafe { *scancode = sc; } }
            if !pressed.is_null()  { unsafe { *pressed  = pr as core::ffi::c_int; } }
            1
        }
        None => 0,
    }
}
