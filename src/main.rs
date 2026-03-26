// src/main.rs

#![no_std]
#![no_main]

use core::arch::global_asm;

// boot.S só é necessário no caminho bare-metal (_start @ 0x80000).
// No boot UEFI o firmware chama efi_main diretamente; boot.S seria conflito.
#[cfg(not(target_os = "uefi"))]
global_asm!(include_str!("arch/aarch64/boot.S"));

// vectors.S define __exception_vectors_start usado por exception::init().
// Necessário em ambos os caminhos (instalado após ExitBootServices no UEFI).
global_asm!(include_str!("arch/aarch64/vectors.S"));

// Kernel
#[macro_use]
mod kernel;

// Core architecture / platform
mod arch;
mod boot;
mod platform;

// Drivers / subsystems
mod drivers;
mod diagnostics;
mod gfx;
mod gfx3d;
mod gpu;
mod audio;
mod media;

// Support / demos
mod demos;
mod math;
mod panic;
mod fs;

// Emulador depende de código C compilado pelo build.rs.
// Para o target UEFI o código C não é compilado — ver build.rs.
#[cfg(not(target_os = "uefi"))]
mod emu;

// Boot via UEFI (target aarch64-unknown-uefi → make uefi)
#[cfg(target_os = "uefi")]
mod uefi;