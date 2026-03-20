// src/kernel/time.rs

use core::sync::atomic::{AtomicU64, Ordering};

static TICKS: AtomicU64 = AtomicU64::new(0);

#[inline(always)]
pub fn ticks() -> u64 {
    TICKS.load(Ordering::Relaxed)
}

#[inline(always)]
pub fn on_tick() {
    TICKS.fetch_add(1, Ordering::Relaxed);
}