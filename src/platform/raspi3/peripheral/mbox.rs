// src/platform/raspi3/peripheral/mbox.rs
//
// Registradores do Mailbox do Raspberry Pi 3.
//

const MMIO_BASE: usize = 0x3F00_0000;

pub const BASE:   usize = MMIO_BASE + 0x0000_B880;

pub const READ:   usize = BASE + 0x00;
pub const STATUS: usize = BASE + 0x18;
pub const WRITE:  usize = BASE + 0x20;

pub const STATUS_FULL:  u32 = 0x8000_0000;
pub const STATUS_EMPTY: u32 = 0x4000_0000;

pub const CHANNEL_PROP: u8 = 8;