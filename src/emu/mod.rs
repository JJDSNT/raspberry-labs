// src/emu/mod.rs

mod host;
pub mod task;

use crate::drivers::framebuffer::Framebuffer;

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

/// Ponto de entrada principal — chamado por run_demo() com o framebuffer real.
pub fn run(fb: Framebuffer) -> ! {
    host::set_framebuffer(fb.ptr as *mut u32, fb.pitch as i32);
    let mut emu = OmegaEmu::new();
    loop {
        emu.run_frame();
    }
}
