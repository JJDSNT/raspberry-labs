// src/kernel/time.rs
//
// Tempo monotônico do kernel.
//
// Responsabilidades:
// - manter contador de ticks (incrementado por IRQ de timer)
// - expor frequência do timer
// - fornecer helpers de conversão (ticks <-> tempo)
//
// NOTA: o log em on_tick() é temporário para diagnóstico.
// Remover após confirmar que o timer está disparando corretamente.
//

use core::sync::atomic::{AtomicU64, Ordering};

static TICKS: AtomicU64 = AtomicU64::new(0);
static TICKS_PER_SECOND: AtomicU64 = AtomicU64::new(0);

/// Inicializa o sistema de tempo.
/// Deve ser chamado durante o boot, após configurar o timer.
/// `ticks_per_second` normalmente vem de CNTFRQ_EL0 (ex: 62_500_000).
pub fn init(ticks_per_second: u64) {
    TICKS.store(0, Ordering::Relaxed);
    TICKS_PER_SECOND.store(ticks_per_second, Ordering::Relaxed);
}

/// Retorna o número de ticks desde o boot.
#[inline(always)]
pub fn ticks() -> u64 {
    TICKS.load(Ordering::Relaxed)
}

/// Frequência do timer (ticks por segundo).
#[inline(always)]
pub fn ticks_per_second() -> u64 {
    TICKS_PER_SECOND.load(Ordering::Relaxed)
}

/// Chamado pela IRQ de timer. Incrementa o contador monotônico.
#[inline(always)]
pub fn on_tick() {
    let t = TICKS.fetch_add(1, Ordering::Relaxed);

    // TODO: remover após confirmar que o timer está disparando.
    if t % 100 == 0 {
        crate::log!("TIME", "tick={}", t);
    }
}

/// Converte ticks para segundos (f32).
#[inline(always)]
pub fn ticks_to_secs_f32(ticks: u64) -> f32 {
    let freq = ticks_per_second();
    if freq == 0 {
        0.0
    } else {
        ticks as f32 / freq as f32
    }
}

/// Converte ticks para milissegundos (f32).
#[inline(always)]
pub fn ticks_to_millis_f32(ticks: u64) -> f32 {
    ticks_to_secs_f32(ticks) * 1_000.0
}

/// Converte segundos para ticks.
#[inline(always)]
pub fn secs_to_ticks(secs: f32) -> u64 {
    let freq = ticks_per_second();
    if secs <= 0.0 || freq == 0 {
        0
    } else {
        (secs * freq as f32) as u64
    }
}

/// Tempo total desde o boot em segundos (f32).
#[inline(always)]
pub fn time_secs() -> f32 {
    ticks_to_secs_f32(ticks())
}

/// Tempo total desde o boot em milissegundos (f32).
#[inline(always)]
pub fn time_millis() -> f32 {
    ticks_to_millis_f32(ticks())
}