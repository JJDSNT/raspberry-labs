// src/emu/task.rs

use super::OmegaEmu;

pub fn omega_task(_arg: usize) -> ! {
    let mut emu = OmegaEmu::new();
    loop {
        emu.run_frame();
    }
}
