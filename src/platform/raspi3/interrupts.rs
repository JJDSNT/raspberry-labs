// src/platform/raspi3/interrupts.rs

use crate::platform::raspi3::mmio;

// BCM2836/BCM2837 ARM local peripherals base.
// No Pi 3, o bloco local de interrupções/timers por-core fica em 0x4000_0000.
const LOCAL_PERIPH_BASE: usize = 0x4000_0000;

// Core timer interrupt control registers:
// 0x4000_0040 + 4 * core
const LOCAL_TIMER_INT_CONTROL0: usize = LOCAL_PERIPH_BASE + 0x40;

// Core IRQ pending registers:
// 0x4000_0060 + 4 * core
const LOCAL_IRQ_PENDING0: usize = LOCAL_PERIPH_BASE + 0x60;

// Bits do "Core N Timers interrupt control".
// O que queremos para CNTP_* em EL1 non-secure é o nCNTPNSIRQ (bit 1).
pub const CNTPNSIRQ_BIT: u32 = 1 << 1;

// Outros bits úteis, caso vocês queiram experimentar depois.
pub const CNTPSIRQ_BIT: u32 = 1 << 0;
pub const CNTVIRQ_BIT: u32 = 1 << 3;
pub const CNTHPIRQ_BIT: u32 = 1 << 2;

/// Habilita a entrega da IRQ do timer físico non-secure (CNTPNSIRQ)
/// para o core 0.
///
/// Esse é o caminho esperado para usar CNTP_TVAL_EL0 / CNTP_CTL_EL0 em EL1.
pub fn enable_core0_cntpnsirq() {
    let reg = LOCAL_TIMER_INT_CONTROL0;
    let val = mmio::read(reg);
    mmio::write(reg, val | CNTPNSIRQ_BIT);
}

/// Desabilita a entrega da IRQ do timer físico non-secure (CNTPNSIRQ)
/// para o core 0.
pub fn disable_core0_cntpnsirq() {
    let reg = LOCAL_TIMER_INT_CONTROL0;
    let val = mmio::read(reg);
    mmio::write(reg, val & !CNTPNSIRQ_BIT);
}

/// Retorna o valor bruto do registrador de controle de timer local do core 0.
#[inline(always)]
pub fn core0_timer_int_control() -> u32 {
    mmio::read(LOCAL_TIMER_INT_CONTROL0)
}

/// Retorna o valor bruto do pending local de IRQ do core 0.
///
/// Os bits baixos representam fontes locais pendentes por-core.
/// Útil para debug de bring-up.
#[inline(always)]
pub fn core0_irq_pending() -> u32 {
    mmio::read(LOCAL_IRQ_PENDING0)
}

/// Heurística mínima para o primeiro bring-up:
/// verifica se existe alguma IRQ local pendente no core 0.
///
/// Para diagnóstico inicial isso já ajuda bastante, mesmo antes de
/// mapear todos os bits individualmente.
#[inline(always)]
pub fn core0_has_local_irq_pending() -> bool {
    (core0_irq_pending() & 0x7ff) != 0
}

/// Inicialização mínima do caminho local de IRQ para o timer do core 0.
pub fn init_core0_timer_irq() {
    enable_core0_cntpnsirq();
}