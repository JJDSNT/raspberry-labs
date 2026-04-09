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
// ---------------------------------------------------------------------------

const KEY_RING_SIZE: usize = 64;

struct KeyRing {
    buf:  [(u8, u8); KEY_RING_SIZE],
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
    }

    fn pop(&mut self) -> Option<(u8, u8)> {
        if self.head == self.tail { return None; }
        let ev = self.buf[self.head];
        self.head = (self.head + 1) % KEY_RING_SIZE;
        Some(ev)
    }
}

static KEY_RING: IrqSafeSpinLock<KeyRing> = IrqSafeSpinLock::new(KeyRing::new());

/// Versão Rust de omega_host_poll_key — usada pelo launcher.
/// Retorna `Some((scancode, pressed))` ou `None` se o ring estiver vazio.
pub fn poll_key() -> Option<(u8, bool)> {
    KEY_RING.lock().pop().map(|(sc, pr)| (sc, pr != 0))
}

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
    crate::kernel::scheduler::yield_now();
}

#[no_mangle]
pub extern "C" fn omega_host_log(msg: *const core::ffi::c_char) {
    if msg.is_null() { return; }

    let bytes = unsafe {
        let mut len = 0;
        while *msg.add(len) != 0 { len += 1; }
        core::slice::from_raw_parts(msg as *const u8, len)
    };

    if let Ok(s) = core::str::from_utf8(bytes) {
        crate::log!("EMU", "{}", s);
    }
}

#[no_mangle]
pub extern "C" fn omega_host_audio_sample(
    _left:  core::ffi::c_short,
    _right: core::ffi::c_short,
) {
}

#[no_mangle]
pub extern "C" fn omega_host_push_key(scancode: u8, pressed: core::ffi::c_int) {
    KEY_RING.lock().push(scancode, if pressed != 0 { 1 } else { 0 });
}

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

// ---------------------------------------------------------------------------
// HDF backend (C ↔ Rust bridge) — leitura streaming direto do SD card
// ---------------------------------------------------------------------------

struct HdfFile {
    inner: crate::fs::fat32::Fat32File,
}

// Safety: Fat32File contém apenas campos primitivos (u32/u64), é Send.
unsafe impl Send for HdfFile {}

static HDF_FILE: IrqSafeSpinLock<Option<HdfFile>> =
    IrqSafeSpinLock::new(None);

/// Valor sentinel não-nulo devolvido como handle opaco para o C.
const HDF_SENTINEL: usize = 1;

#[no_mangle]
pub extern "C" fn omega_hdf_open(
    path: *const u8,
    size_out: *mut u64,
) -> *mut core::ffi::c_void {
    if path.is_null() || size_out.is_null() {
        return core::ptr::null_mut();
    }

    let path_str = unsafe {
        let mut len = 0;
        while *path.add(len) != 0 { len += 1; }
        core::str::from_utf8_unchecked(core::slice::from_raw_parts(path, len))
    };

    crate::log!("EMU", "HDF open '{}'", path_str);

    let file = match crate::fs::fat32::open_file(path_str) {
        Some(f) => f,
        None => {
            crate::log!("EMU", "HDF '{}' not found on SD", path_str);
            return core::ptr::null_mut();
        }
    };

    let size = file.file_size as u64;
    unsafe { *size_out = size; }

    *HDF_FILE.lock() = Some(HdfFile { inner: file });

    crate::log!("EMU", "HDF opened ({} bytes)", size);

    HDF_SENTINEL as *mut core::ffi::c_void
}

#[no_mangle]
pub extern "C" fn omega_hdf_read(
    handle: *mut core::ffi::c_void,
    offset: u64,
    buffer: *mut u8,
    size: u32,
) -> i32 {
    if handle.is_null() || buffer.is_null() { return -1; }

    let mut guard = HDF_FILE.lock();
    let Some(file) = &mut *guard else { return -2; };

    let buf = unsafe { core::slice::from_raw_parts_mut(buffer, size as usize) };
    let n   = file.inner.read_at(offset, buf);

    if n == size as usize { 0 } else { -3 }
}

#[no_mangle]
pub extern "C" fn omega_hdf_close(
    handle: *mut core::ffi::c_void,
) {
    if handle.is_null() { return; }
    *HDF_FILE.lock() = None;
    crate::log!("EMU", "HDF closed");
}