// src/arch/aarch64/regs/timer.rs
//
// Registradores do timer genérico AArch64 (CNTP_* em EL1).
//

use core::arch::asm;

pub struct CntFrq;

impl CntFrq {
    /// Frequência do counter em Hz (ex: 62_500_000 no Pi 3).
    #[inline(always)]
    pub fn read() -> u64 {
        let v: u64;
        unsafe {
            asm!("mrs {}, CNTFRQ_EL0", out(reg) v, options(nomem, nostack, preserves_flags));
        }
        v
    }
}

pub struct CntPct;

impl CntPct {
    /// Valor atual do counter físico.
    #[inline(always)]
    pub fn read() -> u64 {
        let v: u64;
        unsafe {
            asm!("mrs {}, CNTPCT_EL0", out(reg) v, options(nomem, nostack, preserves_flags));
        }
        v
    }
}

pub struct CntpCtl;

impl CntpCtl {
    pub const ENABLE:  u64 = 1 << 0;
    pub const IMASK:   u64 = 1 << 1;
    pub const ISTATUS: u64 = 1 << 2;

    #[inline(always)]
    pub fn read() -> u64 {
        let v: u64;
        unsafe {
            asm!("mrs {}, CNTP_CTL_EL0", out(reg) v, options(nomem, nostack, preserves_flags));
        }
        v
    }

    #[inline(always)]
    pub fn write(v: u64) {
        unsafe {
            asm!("msr CNTP_CTL_EL0, {}", "isb", in(reg) v, options(nomem, nostack, preserves_flags));
        }
    }

    #[inline(always)]
    pub fn is_pending() -> bool {
        (Self::read() & Self::ISTATUS) != 0
    }
}

pub struct CntpTval;

impl CntpTval {
    /// Programa o próximo disparo em `ticks` ciclos.
    #[inline(always)]
    pub fn write(ticks: u64) {
        unsafe {
            asm!("msr CNTP_TVAL_EL0, {}", "isb", in(reg) ticks, options(nomem, nostack, preserves_flags));
        }
    }
}