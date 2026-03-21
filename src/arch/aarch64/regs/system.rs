// src/arch/aarch64/regs/system.rs
//
// Acesso tipado aos registradores do sistema AArch64 (EL1).
// Cada função é um wrapper fino sobre MSR/MRS — sem lógica adicional.
//

use core::arch::asm;

// ---------------------------------------------------------------------------
// SCTLR_EL1 — System Control Register
// ---------------------------------------------------------------------------

pub struct Sctlr;

impl Sctlr {
    pub const M:   u64 = 1 << 0;  // MMU enable
    pub const C:   u64 = 1 << 2;  // D-cache enable
    pub const SA:  u64 = 1 << 3;  // Stack Alignment Check EL1
    pub const SA0: u64 = 1 << 4;  // Stack Alignment Check EL0
    pub const I:   u64 = 1 << 12; // I-cache enable

    #[inline(always)]
    pub fn read() -> u64 {
        let v: u64;
        unsafe {
            asm!("mrs {}, SCTLR_EL1", out(reg) v, options(nomem, nostack, preserves_flags));
        }
        v
    }

    #[inline(always)]
    pub fn write(v: u64) {
        unsafe {
            asm!("msr SCTLR_EL1, {}", "isb", in(reg) v, options(nomem, nostack, preserves_flags));
        }
    }
}

// ---------------------------------------------------------------------------
// TCR_EL1 — Translation Control Register
// ---------------------------------------------------------------------------

pub struct Tcr;

impl Tcr {
    #[inline(always)]
    pub fn write(v: u64) {
        unsafe {
            asm!("msr TCR_EL1, {}", "isb", in(reg) v, options(nomem, nostack, preserves_flags));
        }
    }
}

// ---------------------------------------------------------------------------
// MAIR_EL1 — Memory Attribute Indirection Register
// ---------------------------------------------------------------------------

pub struct Mair;

impl Mair {
    /// Atributo Normal WB cacheable (índice 0)
    pub const NORMAL:    u64 = 0xFF;
    /// Atributo Device-nGnRnE (índice 1)
    pub const DEVICE:    u64 = 0x00;
    /// Atributo Normal Non-Cacheable (índice 2)
    pub const NORMAL_NC: u64 = 0x44;

    pub const IDX_NORMAL:    u64 = 0;
    pub const IDX_DEVICE:    u64 = 1;
    pub const IDX_NORMAL_NC: u64 = 2;

    #[inline(always)]
    pub fn write(v: u64) {
        unsafe {
            asm!("msr MAIR_EL1, {}", "isb", in(reg) v, options(nomem, nostack, preserves_flags));
        }
    }
}

// ---------------------------------------------------------------------------
// TTBR0_EL1 — Translation Table Base Register 0
// ---------------------------------------------------------------------------

pub struct Ttbr0;

impl Ttbr0 {
    #[inline(always)]
    pub fn write(v: u64) {
        unsafe {
            asm!("msr TTBR0_EL1, {}", "isb", in(reg) v, options(nomem, nostack, preserves_flags));
        }
    }

    #[inline(always)]
    pub fn read() -> u64 {
        let v: u64;
        unsafe {
            asm!("mrs {}, TTBR0_EL1", out(reg) v, options(nomem, nostack, preserves_flags));
        }
        v
    }
}

// ---------------------------------------------------------------------------
// VBAR_EL1 — Vector Base Address Register
// ---------------------------------------------------------------------------

pub struct Vbar;

impl Vbar {
    #[inline(always)]
    pub fn write(v: u64) {
        unsafe {
            asm!("msr VBAR_EL1, {}", "isb", in(reg) v, options(nomem, nostack, preserves_flags));
        }
    }

    #[inline(always)]
    pub fn read() -> u64 {
        let v: u64;
        unsafe {
            asm!("mrs {}, VBAR_EL1", out(reg) v, options(nomem, nostack, preserves_flags));
        }
        v
    }
}

// ---------------------------------------------------------------------------
// VBAR_EL2 — Vector Base Address Register EL2
// ---------------------------------------------------------------------------

pub struct VbarEl2;

impl VbarEl2 {
    #[inline(always)]
    pub fn write(v: u64) {
        unsafe {
            asm!("msr VBAR_EL2, {}", "isb", in(reg) v, options(nomem, nostack, preserves_flags));
        }
    }
}

// ---------------------------------------------------------------------------
// CurrentEL
// ---------------------------------------------------------------------------

pub struct CurrentEl;

impl CurrentEl {
    #[inline(always)]
    pub fn read() -> u64 {
        let v: u64;
        unsafe {
            asm!("mrs {}, CurrentEL", out(reg) v, options(nomem, nostack, preserves_flags));
        }
        (v >> 2) & 0b11
    }
}

// ---------------------------------------------------------------------------
// DAIF — Interrupt mask bits
// ---------------------------------------------------------------------------

pub struct Daif;

impl Daif {
    pub const IRQ_BIT: u64 = 1 << 7;

    #[inline(always)]
    pub fn read() -> u64 {
        let v: u64;
        unsafe {
            asm!("mrs {}, DAIF", out(reg) v, options(nomem, nostack, preserves_flags));
        }
        v
    }

    /// Desabilita IRQs (seta bit I do DAIF).
    #[inline(always)]
    pub fn disable_irq() {
        unsafe {
            asm!("msr daifset, #2", options(nomem, nostack, preserves_flags));
        }
    }

    /// Habilita IRQs (limpa bit I do DAIF).
    #[inline(always)]
    pub fn enable_irq() {
        unsafe {
            asm!("msr daifclr, #2", options(nomem, nostack, preserves_flags));
        }
    }

    #[inline(always)]
    pub fn irq_enabled() -> bool {
        (Self::read() & Self::IRQ_BIT) == 0
    }
}

// ---------------------------------------------------------------------------
// ESR_EL1 / FAR_EL1 — Exception syndrome / Fault address
// ---------------------------------------------------------------------------

pub struct Esr;

impl Esr {
    #[inline(always)]
    pub fn read() -> u64 {
        let v: u64;
        unsafe {
            asm!("mrs {}, ESR_EL1", out(reg) v, options(nomem, nostack, preserves_flags));
        }
        v
    }

    /// Exception Class — bits [31:26]
    pub fn ec(esr: u64) -> u64 {
        (esr >> 26) & 0x3F
    }

    /// Instruction Specific Syndrome — bits [24:0]
    pub fn iss(esr: u64) -> u64 {
        esr & 0x1FF_FFFF
    }
}

pub struct Far;

impl Far {
    #[inline(always)]
    pub fn read() -> u64 {
        let v: u64;
        unsafe {
            asm!("mrs {}, FAR_EL1", out(reg) v, options(nomem, nostack, preserves_flags));
        }
        v
    }
}

// ---------------------------------------------------------------------------
// ELR_EL1 / SPSR_EL1 — Exception Link Register / Saved Program Status
// ---------------------------------------------------------------------------

pub struct Elr;

impl Elr {
    #[inline(always)]
    pub fn read() -> u64 {
        let v: u64;
        unsafe {
            asm!("mrs {}, ELR_EL1", out(reg) v, options(nomem, nostack, preserves_flags));
        }
        v
    }

    #[inline(always)]
    pub fn write(v: u64) {
        unsafe {
            asm!("msr ELR_EL1, {}", in(reg) v, options(nomem, nostack, preserves_flags));
        }
    }
}

pub struct Spsr;

impl Spsr {
    /// EL1h com IRQs habilitadas — valor inicial para tasks
    pub const EL1H_IRQ_ENABLED: u64 = 0b0101;

    #[inline(always)]
    pub fn read() -> u64 {
        let v: u64;
        unsafe {
            asm!("mrs {}, SPSR_EL1", out(reg) v, options(nomem, nostack, preserves_flags));
        }
        v
    }

    #[inline(always)]
    pub fn write(v: u64) {
        unsafe {
            asm!("msr SPSR_EL1, {}", in(reg) v, options(nomem, nostack, preserves_flags));
        }
    }
}