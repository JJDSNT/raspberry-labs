// src/emu/mod.rs

mod host;
pub mod task;

use core::ffi::c_void;

extern "C" {
    fn omega_init();
    fn omega_run_frame();
}

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
