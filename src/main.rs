// src/main.rs

#![no_std]
#![no_main]

use core::arch::global_asm;

// AArch64 bootstrap / low-level runtime
global_asm!(include_str!("arch/aarch64/boot.S"));
global_asm!(include_str!("arch/aarch64/vectors.S"));

// Core architecture / platform
mod arch;
mod boot;
mod platform;

// Kernel
mod kernel;

// Drivers / subsystems
mod drivers;
mod diagnostics;
mod gfx;
mod audio;
mod media;

// Support / demos
mod demos;
mod math;
mod panic;
mod fs;

// Emulador
mod emu;