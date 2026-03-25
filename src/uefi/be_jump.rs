// src/uefi/be_jump.rs
//
// Transição UEFI (LE) → kernel BE.
//
// Fluxo:
//   1. Flush D-cache do range [kernel_start, kernel_start+size)
//   2. Invalida I-cache (IC IALLU)
//   3. Desabilita MMU (SCTLR_EL1.M=0) — kernel BE usa identity map própria
//   4. Seta SCTLR_EL1.EE=1 (data big-endian)
//   5. Invalida TLB
//   6. Branch para entry point
//
// Nota: em AArch64, instruction fetch é sempre LE independente de EE.
// O bit EE afeta apenas loads/stores de dados. O compilador aarch64_be
// gera código AArch64 normal (instrução LE) com dados em BE — portanto
// setar EE=1 é suficiente para o kernel BE rodar corretamente.
//
// Nota sobre EL: o UEFI já nos colocou em EL1. O kernel BE também deve
// ser preparado para começar em EL1 (diferente do boot bare-metal que
// começa em EL2). O entry point do kernel precisa lidar com isso.

/// Flush D-cache + I-cache, switch para BE, jump para entry point.
/// Nunca retorna.
///
/// # Safety
/// `entry` deve ser um endereço válido de código AArch64 compilado para
/// aarch64_be (LE instruction encoding, BE data). `kernel_start` e
/// `kernel_size` definem o range para flush de cache.
pub unsafe fn switch_be_and_jump(
    entry:        usize,
    kernel_start: usize,
    kernel_size:  usize,
) -> ! {
    core::arch::asm!(
        // ── D-cache flush: DC CIVAC para cada cache line do range ────────────
        "cbz x2, 2f",           // sem tamanho → skip flush
        "add x9, x1, x2",       // x9  = end address
        "mov x10, x1",          // x10 = current address
        "1:",
        "dc  civac, x10",       // clean + invalidate cache line by VA
        "add x10, x10, #64",    // próxima cache line (64 bytes no Cortex-A53)
        "cmp x10, x9",
        "blo 1b",
        "dsb ish",              // aguarda conclusão das operações de cache

        // ── I-cache invalidate all ───────────────────────────────────────────
        "ic  iallu",            // invalidate I-cache (all, inner shareable)
        "dsb ish",
        "isb",

        // ── MMU off + EE=1 ───────────────────────────────────────────────────
        "2:",
        "mrs x9, sctlr_el1",
        "bic x9, x9, #1",           // M=0: desabilita MMU
        "orr x9, x9, #0x2000000",   // EE=1: data big-endian (bit 25)
        "msr sctlr_el1, x9",
        "isb",

        // ── TLB flush ────────────────────────────────────────────────────────
        "tlbi vmalle1",
        "dsb ish",
        "isb",

        // ── Jump para BE kernel ───────────────────────────────────────────────
        "br  x0",

        in("x0") entry,
        in("x1") kernel_start,
        in("x2") kernel_size,
        options(noreturn),
    );
}
