// src/arch/aarch64/regs/cache.rs
//
// Barreiras de memória, TLB e operações de cache para AArch64.
//
// Guia de uso:
//
//   dsb_sy()  — antes de habilitar MMU, após escrever tabelas de página,
//               operações críticas de sistema. Mais lento.
//   dsb_ish() — sincronização entre cores (Inner Shareable).
//               Suficiente para a maioria dos casos SMP.
//   dmb_sy()  — ordenação de acesso a MMIO e DMA.
//   dmb_ish() — ordenação entre cores sem envolver dispositivos.
//   isb()     — flush de pipeline após mudar registradores do sistema
//               (SCTLR, TCR, VBAR, etc).
//

use core::arch::asm;

// ---------------------------------------------------------------------------
// DSB — Data Synchronization Barrier
// Garante que todas as operações de memória anteriores completaram.
// ---------------------------------------------------------------------------

/// DSB sistema inteiro — necessário para operações de MMU e init crítico.
#[inline(always)]
pub fn dsb_sy() {
    unsafe {
        asm!("dsb sy", options(nostack, preserves_flags));
    }
}

/// DSB Inner Shareable — sincronização entre cores.
/// Suficiente para sincronização de tasks e spinlocks em SMP.
#[inline(always)]
pub fn dsb_ish() {
    unsafe {
        asm!("dsb ish", options(nostack, preserves_flags));
    }
}

// ---------------------------------------------------------------------------
// DMB — Data Memory Barrier
// Garante ordenação de acessos à memória sem esperar pela conclusão.
// ---------------------------------------------------------------------------

/// DMB sistema inteiro — use para ordenar acessos a MMIO e DMA.
#[inline(always)]
pub fn dmb_sy() {
    unsafe {
        asm!("dmb sy", options(nostack, preserves_flags));
    }
}

/// DMB Inner Shareable — ordenação entre cores sem envolver dispositivos.
/// Use para sincronização de dados entre tasks no mesmo cluster.
#[inline(always)]
pub fn dmb_ish() {
    unsafe {
        asm!("dmb ish", options(nostack, preserves_flags));
    }
}

// ---------------------------------------------------------------------------
// ISB — Instruction Synchronization Barrier
// Flush de pipeline — necessário após mudar registradores do sistema.
// ---------------------------------------------------------------------------

#[inline(always)]
pub fn isb() {
    unsafe {
        asm!("isb", options(nostack, preserves_flags));
    }
}

// ---------------------------------------------------------------------------
// TLB
// ---------------------------------------------------------------------------

/// Invalida todos os TLBs de EL1 (Inner Shareable).
/// Usar após modificar tabelas de página.
#[inline(always)]
pub fn tlbi_vmalle1() {
    unsafe {
        asm!(
            "tlbi vmalle1",
            "dsb sy",
            "isb",
            options(nostack, preserves_flags)
        );
    }
}

/// Invalida todo o I-cache (PoU) no domínio Inner Shareable.
/// Necessário em hardware real antes de habilitar o I-cache na MMU,
/// para descartar entradas deixadas pelo firmware do Pi.
#[inline(always)]
pub fn ic_iallu() {
    unsafe {
        asm!(
            "ic iallu",
            "dsb ish",
            "isb",
            options(nostack, preserves_flags)
        );
    }
}

// ---------------------------------------------------------------------------
// Cache — flush de linhas (clean + invalidate)
// ---------------------------------------------------------------------------

/// Clean + Invalidate de uma linha de cache pelo endereço virtual.
/// Necessário para garantir que dados escritos pela CPU são visíveis
/// a dispositivos de DMA ou ao page walker da MMU.
#[inline(always)]
pub fn dc_civac(addr: usize) {
    unsafe {
        asm!(
            "dc civac, {addr}",
            addr = in(reg) addr,
            options(nostack, preserves_flags)
        );
    }
}

/// Flush de um range de memória (clean + invalidate por linha de cache).
///
/// Use antes de habilitar a MMU para garantir que as tabelas de página
/// escritas pela CPU estão na memória física.
pub fn flush_range(start: usize, end: usize) {
    const CACHE_LINE: usize = 64;
    let mut addr = start & !(CACHE_LINE - 1);
    while addr < end {
        dc_civac(addr);
        addr += CACHE_LINE;
    }
    dsb_sy();
    isb();
}

/// Invalida o cache de dados em um intervalo de endereços.
/// Força a CPU a ler os dados diretamente da RAM física na próxima tentativa.
pub fn invalidate_range(start: usize, end: usize) {
    let mut curr = start & !(64 - 1); // Alinha ao tamanho da linha de cache (64 bytes no Pi 3)
    
    while curr < end {
        unsafe {
            // dc ivac: Data Cache Invalidate by Virtual Address to Point of Coherency
            core::arch::asm!("dc ivac, {}", in(reg) curr, options(nostack));
        }
        curr += 64;
    }
    
    // Barreiras para garantir que a invalidação terminou antes de prosseguirmos
    unsafe {
        core::arch::asm!("dsb sy", options(nostack));
        core::arch::asm!("isb", options(nostack));
    }
}