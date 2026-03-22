// src/emu/host.rs
// Implementações Rust dos callbacks omega_host_* chamados pelo C.
//
// O ponteiro do framebuffer é configurado antes de iniciar o emulador
// via set_framebuffer().

use core::sync::atomic::{AtomicPtr, AtomicI32, Ordering};

static FB_PTR:   AtomicPtr<u32> = AtomicPtr::new(core::ptr::null_mut());
static FB_PITCH: AtomicI32      = AtomicI32::new(0);

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

#[no_mangle]
pub extern "C" fn omega_host_poll_key(
    _scancode: *mut u8,
    _pressed: *mut core::ffi::c_int,
) -> core::ffi::c_int {
    // No input yet — stub returns empty
    0
}
