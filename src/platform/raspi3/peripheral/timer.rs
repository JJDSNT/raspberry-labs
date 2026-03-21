// src/platform/raspi3/peripheral/timer.rs
//
// Registradores do bloco de periféricos locais do BCM2836/BCM2837.
// Base: 0x4000_0000 (diferente do MMIO principal em 0x3F00_0000)
//

pub const LOCAL_BASE: usize = 0x4000_0000;

// Timer interrupt control por core (offset 0x40 + 4 * core)
pub const TIMER_INT_CTRL0: usize = LOCAL_BASE + 0x40;
pub const TIMER_INT_CTRL1: usize = LOCAL_BASE + 0x44;
pub const TIMER_INT_CTRL2: usize = LOCAL_BASE + 0x48;
pub const TIMER_INT_CTRL3: usize = LOCAL_BASE + 0x4C;

// IRQ pending por core (offset 0x60 + 4 * core)
pub const IRQ_PENDING0: usize = LOCAL_BASE + 0x60;
pub const IRQ_PENDING1: usize = LOCAL_BASE + 0x64;
pub const IRQ_PENDING2: usize = LOCAL_BASE + 0x68;
pub const IRQ_PENDING3: usize = LOCAL_BASE + 0x6C;

// Bits do timer interrupt control
pub const CNTPSIRQ_BIT:   u32 = 1 << 0; // Secure physical timer
pub const CNTPNSIRQ_BIT:  u32 = 1 << 1; // Non-secure physical timer (usado em EL1)
pub const CNTHPIRQ_BIT:   u32 = 1 << 2; // Hypervisor physical timer
pub const CNTVIRQ_BIT:    u32 = 1 << 3; // Virtual timer