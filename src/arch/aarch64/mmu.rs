// src/arch/aarch64/mmu.rs
//
// MMU para o Raspberry Pi 3 (AArch64, EL1).
//
// Mapeamento flat (VA = PA) com blocos de 2MB (L1 → L2):
//
//   0x0000_0000 - 0x3BFF_FFFF  Normal WB cacheable     (RAM + kernel)
//   0x3C00_0000 - 0x3EFF_FFFF  Normal Non-Cacheable    (GPU framebuffer)
//   0x3F00_0000 - 0x3FFF_FFFF  Device-nGnRnE           (BCM MMIO)
//   0x4000_0000 - 0x401F_FFFF  Device-nGnRnE           (periféricos locais)
//
// As tabelas ficam na seção .mmu (alinhada a 4KB no linker),
// separadas do BSS para garantir flush de cache correto antes
// de habilitar a MMU.
//

use crate::arch::aarch64::regs::{
    cache, Daif, Mair, Sctlr, Tcr, Ttbr0,
};

// ---------------------------------------------------------------------------
// MAIR_EL1 — três atributos de memória
// ---------------------------------------------------------------------------
const MAIR_VALUE: u64 =
      Mair::NORMAL                       // índice 0 — Normal WB
    | (Mair::DEVICE    << 8)             // índice 1 — Device
    | (Mair::NORMAL_NC << 16);           // índice 2 — Normal NC

// ---------------------------------------------------------------------------
// TCR_EL1
//
// T0SZ=32  → VA space = 4GB
// TG0=4KB  → granule 4KB
// SH0=ISH  → Inner Shareable
// ORGN0=01 → Outer WB RA WA
// IRGN0=01 → Inner WB RA WA
// EPD1=1   → desabilita TTBR1
// IPS=001  → 36-bit PA
// ---------------------------------------------------------------------------
const TCR_VALUE: u64 =
      32u64                 // T0SZ
    | (0b00 << 14)          // TG0 = 4KB
    | (0b11 << 12)          // SH0 = Inner Shareable
    | (0b01 << 10)          // ORGN0
    | (0b01 << 8)           // IRGN0
    | (1    << 23)          // EPD1
    | (0b001 << 32);        // IPS = 36 bits

// ---------------------------------------------------------------------------
// Bits de descritor de bloco/tabela
// ---------------------------------------------------------------------------
const DESC_VALID:     u64 = 1 << 0;
const DESC_BLOCK:     u64 = 0 << 1; // bloco (L1=1GB ou L2=2MB)
const DESC_TABLE:     u64 = 1 << 1; // tabela (aponta para próximo nível)
const DESC_AF:        u64 = 1 << 10; // Access Flag
const DESC_SH_INNER:  u64 = 0b11 << 8; // Inner Shareable
const DESC_AP_RW_EL1: u64 = 0b00 << 6; // R/W EL1
const DESC_UXN:       u64 = 1 << 54;   // Unprivileged Execute Never
const DESC_PXN:       u64 = 1 << 53;   // Privileged Execute Never

const fn attr_idx(idx: u64) -> u64 { idx << 2 }

const fn block_normal(pa: u64) -> u64 {
    pa | DESC_VALID | DESC_BLOCK | DESC_AF
       | DESC_SH_INNER | DESC_AP_RW_EL1
       | attr_idx(Mair::IDX_NORMAL)
}

const fn block_normal_nc(pa: u64) -> u64 {
    pa | DESC_VALID | DESC_BLOCK | DESC_AF
       | DESC_SH_INNER | DESC_AP_RW_EL1
       | DESC_UXN | DESC_PXN
       | attr_idx(Mair::IDX_NORMAL_NC)
}

const fn block_device(pa: u64) -> u64 {
    pa | DESC_VALID | DESC_BLOCK | DESC_AF
       | DESC_AP_RW_EL1
       | DESC_UXN | DESC_PXN
       | attr_idx(Mair::IDX_DEVICE)
}

const fn table_desc(pa: u64) -> u64 {
    pa | DESC_VALID | DESC_TABLE
}

// ---------------------------------------------------------------------------
// Tabelas de página — seção .mmu (alinhada a 4KB pelo linker)
// ---------------------------------------------------------------------------
#[repr(C, align(4096))]
struct PageTable([u64; 512]);

#[link_section = ".mmu"]
static mut L1: PageTable = PageTable([0u64; 512]);

#[link_section = ".mmu"]
static mut L2_0: PageTable = PageTable([0u64; 512]); // 0x0000_0000..0x3FFF_FFFF

#[link_section = ".mmu"]
static mut L2_1: PageTable = PageTable([0u64; 512]); // 0x4000_0000..0x7FFF_FFFF

// ---------------------------------------------------------------------------
// Símbolos do linker para flush da seção .mmu
// ---------------------------------------------------------------------------
unsafe extern "C" {
    static __mmu_start: u8;
    static __mmu_end: u8;
}

// ---------------------------------------------------------------------------
// init()
// ---------------------------------------------------------------------------
pub fn init() {
    unsafe {
        fill_tables();

        // Flush da seção .mmu para garantir que as tabelas estão
        // na memória física antes de habilitar a MMU.
        let start = core::ptr::addr_of!(__mmu_start) as usize;
        let end   = core::ptr::addr_of!(__mmu_end)   as usize;
        cache::flush_range(start, end);

        let ttbr0 = core::ptr::addr_of!(L1) as u64;

        Mair::write(MAIR_VALUE);
        Tcr::write(TCR_VALUE);
        Ttbr0::write(ttbr0);

        cache::dsb_sy();
        cache::isb();
        cache::tlbi_vmalle1();

        // Invalida I-cache antes de habilitá-la — em hardware real o firmware
        // pode ter deixado entradas que corrompem a execução após enable.
        cache::ic_iallu();

        // Habilita MMU + D-cache + I-cache
        // Desabilita stack alignment check (SA, SA0) durante desenvolvimento
        // Em builds BE (target_endian = "big"), seta EE para data accesses em BE
        let mut sctlr = Sctlr::read();
        sctlr &= !(Sctlr::SA | Sctlr::SA0);
        sctlr |= Sctlr::M | Sctlr::C | Sctlr::I;
        #[cfg(target_endian = "big")]
        { sctlr |= Sctlr::EE; }
        Sctlr::write(sctlr);
    }

    crate::log!("MMU", "enabled — D-cache and I-cache active");
}

/// Mapeia um endereço físico adicional como Device (MMIO).
/// Útil para adicionar periféricos sem rebuildar as tabelas completas.
pub fn map_device(pa: u64, size_mb: usize) {
    // Implementação futura — por enquanto o mapeamento estático cobre
    // todos os periféricos do Pi 3.
    let _ = (pa, size_mb);
}

unsafe fn fill_tables() {
    let l2_0_pa = core::ptr::addr_of!(L2_0) as u64;
    let l2_1_pa = core::ptr::addr_of!(L2_1) as u64;

    // L1[0] → L2_0 (primeiro GB: 0x0000_0000..0x3FFF_FFFF)
    L1.0[0] = table_desc(l2_0_pa);

    // L1[1] → L2_1 (segundo GB: 0x4000_0000..0x7FFF_FFFF)
    L1.0[1] = table_desc(l2_1_pa);

    // L1[2..511] — não mapeados
    for i in 2..512 {
        L1.0[i] = 0;
    }

    // -----------------------------------------------------------------------
    // L2_0: 0x0000_0000..0x3FFF_FFFF em blocos de 2MB
    //
    // Entradas 0..479   → 0x0000_0000..0x3BFF_FFFF  Normal cacheable
    // Entradas 480..503 → 0x3C00_0000..0x3EFF_FFFF  Normal NC (framebuffer)
    // Entradas 504..511 → 0x3F00_0000..0x3FFF_FFFF  Device (BCM MMIO)
    // -----------------------------------------------------------------------
    for i in 0usize..480 {
        L2_0.0[i] = block_normal((i as u64) * 0x0020_0000);
    }
    for i in 480usize..504 {
        L2_0.0[i] = block_normal_nc((i as u64) * 0x0020_0000);
    }
    for i in 504usize..512 {
        L2_0.0[i] = block_device((i as u64) * 0x0020_0000);
    }

    // -----------------------------------------------------------------------
    // L2_1: 0x4000_0000..0x7FFF_FFFF em blocos de 2MB
    //
    // Entrada 0 → 0x4000_0000..0x401F_FFFF  Device (periféricos locais)
    // Resto     → não mapeado
    // -----------------------------------------------------------------------
    L2_1.0[0] = block_device(0x4000_0000);
    for i in 1usize..512 {
        L2_1.0[i] = 0;
    }
}