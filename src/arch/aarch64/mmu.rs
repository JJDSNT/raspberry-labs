// src/arch/aarch64/mmu.rs
// 
// MMU Consolidada para Raspberry Pi 3B (AArch64)
// Foco: Estabilidade de Mailbox e Framebuffer

use crate::arch::aarch64::regs::{cache, Mair, Sctlr, Tcr, Ttbr0};

// ---------------------------------------------------------------------------
// MAIR_EL1 - Atributos de Memória
// ---------------------------------------------------------------------------
// Índice 0: Normal WB RA WA (Memória RAM comum)
// Índice 1: Device nGnRE (0x04) - O "ponto doce" para periféricos do Pi 3
// Índice 2: Normal Non-Cacheable (0x44) - Para o buffer de pixels
const MAIR_VALUE: u64 = 0xFF | (0x04 << 8) | (0x44 << 16);

// ---------------------------------------------------------------------------
// TCR_EL1 - Controle de Tradução
// ---------------------------------------------------------------------------
const TCR_VALUE: u64 =
      32u64                 // T0SZ: 4GB de espaço virtual (2^(64-32))
    | (0b00 << 14)          // TG0: Grânulo de 4KB
    | (0b11 << 12)          // SH0: Inner Shareable
    | (0b01 << 10)          // ORGN0: Outer WB RA WA
    | (0b01 << 8)           // IRGN0: Inner WB RA WA
    | (1    << 23)          // EPD1: Desabilita a tabela superior (TTBR1)
    | (0b000 << 32);        // IPS: 32-bit Physical Address (Limite do Pi 3)

// ---------------------------------------------------------------------------
// Flags de Descritores
// ---------------------------------------------------------------------------
const DESC_VALID:     u64 = 1 << 0;
const DESC_BLOCK:     u64 = 0 << 1; 
const DESC_TABLE:     u64 = 1 << 1; 
const DESC_AF:        u64 = 1 << 10;
const DESC_SH_INNER:  u64 = 0b11 << 8;
const DESC_AP_RW_EL1: u64 = 0b00 << 6;
const DESC_UXN:       u64 = 1 << 54;
const DESC_PXN:       u64 = 1 << 53;

const fn attr_idx(idx: u64) -> u64 { idx << 2 }

const fn block_normal(pa: u64) -> u64 {
    pa | DESC_VALID | DESC_BLOCK | DESC_AF | DESC_SH_INNER | DESC_AP_RW_EL1 | attr_idx(0)
}

const fn block_normal_nc(pa: u64) -> u64 {
    pa | DESC_VALID | DESC_BLOCK | DESC_AF | DESC_SH_INNER | DESC_AP_RW_EL1 | DESC_UXN | DESC_PXN | attr_idx(2)
}

const fn block_device(pa: u64) -> u64 {
    pa | DESC_VALID | DESC_BLOCK | DESC_AF | DESC_AP_RW_EL1 | DESC_UXN | DESC_PXN | attr_idx(1)
}

const fn table_desc(pa: u64) -> u64 {
    pa | DESC_VALID | DESC_TABLE
}

// ---------------------------------------------------------------------------
// Estruturas de Tabelas (Alinhadas a 4KB)
// ---------------------------------------------------------------------------
#[repr(C, align(4096))]
struct PageTable([u64; 512]);

#[link_section = ".mmu"]
static mut L1: PageTable = PageTable([0u64; 512]);

#[link_section = ".mmu"]
static mut L2_0: PageTable = PageTable([0u64; 512]);

#[link_section = ".mmu"]
static mut L2_1: PageTable = PageTable([0u64; 512]);

unsafe extern "C" {
    static __mmu_start: u8;
    static __mmu_end: u8;
}

// ---------------------------------------------------------------------------
// Inicialização
// ---------------------------------------------------------------------------
pub fn init() {
    unsafe {
        fill_tables();

        // Sincronização pré-ativação: Garante que as tabelas cheguem na RAM
        let start = core::ptr::addr_of!(__mmu_start) as usize;
        let end   = core::ptr::addr_of!(__mmu_end)   as usize;
        cache::flush_range(start, end);

        Mair::write(MAIR_VALUE);
        Tcr::write(TCR_VALUE);
        Ttbr0::write(core::ptr::addr_of!(L1) as u64);

        // Sequência de barreira exigida pela arquitetura ARMv8
        cache::isb();               // Sincroniza escrita dos registros acima
        cache::tlbi_vmalle1();      // Invalida TLB (essencial se o firmware usou MMU)
        cache::dsb_sy();            // Aguarda conclusão da invalidação
        cache::isb();               // Sincroniza contexto

        cache::ic_iallu();          // Invalida cache de instruções
        cache::dsb_sy();
        cache::isb();

        // Ativação Final
        let mut sctlr = Sctlr::read();
        sctlr &= !(Sctlr::SA | Sctlr::SA0); // Desabilita checagem de alinhamento de stack
        sctlr |= Sctlr::M | Sctlr::C | Sctlr::I; // MMU + D-Cache + I-Cache
        
        Sctlr::write(sctlr);
        cache::isb(); // Garante que a próxima instrução já rode com MMU ativa
    }

    crate::log!("MMU", "Enabled - Flat Map - Device nGnRE active");
}

unsafe fn fill_tables() {
    let l2_0_pa = core::ptr::addr_of!(L2_0) as u64;
    let l2_1_pa = core::ptr::addr_of!(L2_1) as u64;

    L1.0[0] = table_desc(l2_0_pa);
    L1.0[1] = table_desc(l2_1_pa);

    // Mapeamento L2_0 (0MB - 1GB)
    for i in 0usize..480 {
        L2_0.0[i] = block_normal((i as u64) * 0x0020_0000);
    }
    for i in 480usize..504 { // Framebuffer area (0x3C00_0000)
        L2_0.0[i] = block_normal_nc((i as u64) * 0x0020_0000);
    }
    for i in 504usize..512 { // BCM Peripherals (0x3F00_0000)
        L2_0.0[i] = block_device((i as u64) * 0x0020_0000);
    }

    // Mapeamento L2_1 (1GB - 2GB) - Local Peripherals (0x4000_0000)
    L2_1.0[0] = block_device(0x4000_0000);
}