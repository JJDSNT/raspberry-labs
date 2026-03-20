// src/arch/aarch64/timer.rs

use core::arch::asm;

const CNTP_CTL_ENABLE: u64 = 1 << 0;
const CNTP_CTL_IMASK: u64 = 1 << 1;
const CNTP_CTL_ISTATUS: u64 = 1 << 2;

static mut TICKS_PER_IRQ: u64 = 0;

pub fn init(tick_hz: u64) {
    assert!(tick_hz != 0);

    let freq = counter_frequency();
    let interval = freq / tick_hz;

    assert!(interval != 0);

    unsafe {
        TICKS_PER_IRQ = interval;
    }

    program_next_tick();
    enable();
}

#[inline(always)]
pub fn counter_frequency() -> u64 {
    let freq: u64;
    unsafe {
        asm!("mrs {0}, CNTFRQ_EL0", out(reg) freq, options(nomem, nostack, preserves_flags));
    }
    freq
}

#[inline(always)]
pub fn current_count() -> u64 {
    let cnt: u64;
    unsafe {
        asm!("mrs {0}, CNTPCT_EL0", out(reg) cnt, options(nomem, nostack, preserves_flags));
    }
    cnt
}

#[inline(always)]
pub fn control() -> u64 {
    let ctl: u64;
    unsafe {
        asm!("mrs {0}, CNTP_CTL_EL0", out(reg) ctl, options(nomem, nostack, preserves_flags));
    }
    ctl
}

#[inline(always)]
fn enable() {
    let ctl = CNTP_CTL_ENABLE;

    unsafe {
        asm!(
            "msr CNTP_CTL_EL0, {0}",
            "isb",
            in(reg) ctl,
            options(nomem, nostack, preserves_flags)
        );
    }
}

#[inline(always)]
pub fn disable() {
    unsafe {
        asm!(
            "msr CNTP_CTL_EL0, {0}",
            "isb",
            in(reg) 0u64,
            options(nomem, nostack, preserves_flags)
        );
    }
}

#[inline(always)]
fn program_next_tick() {
    let interval = unsafe { TICKS_PER_IRQ };

    unsafe {
        asm!(
            "msr CNTP_TVAL_EL0, {0}",
            "isb",
            in(reg) interval,
            options(nomem, nostack, preserves_flags)
        );
    }
}

/// Tenta consumir a IRQ do timer.
/// Retorna true se a interrupção era nossa e foi rearmada.
pub fn handle_irq() -> bool {
    if !is_pending() {
        return false;
    }

    program_next_tick();
    true
}

#[inline(always)]
fn is_pending() -> bool {
    (control() & CNTP_CTL_ISTATUS) != 0
}