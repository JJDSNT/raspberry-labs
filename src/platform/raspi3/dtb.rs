// src/platform/dtb.rs
//
// Parser mínimo de Device Tree Blob (FDT spec v17).
// Extrai apenas o nó /chosen → propriedade "bootargs".

use core::str;

const FDT_MAGIC: u32 = 0xD00D_FEED;

const FDT_BEGIN_NODE: u32 = 0x1;
const FDT_END_NODE:   u32 = 0x2;
const FDT_PROP:       u32 = 0x3;
const FDT_NOP:        u32 = 0x4;
const FDT_END:        u32 = 0x9;

#[repr(C)]
struct FdtHeader {
    magic:             u32,
    totalsize:         u32,
    off_dt_struct:     u32,
    off_dt_strings:    u32,
    off_mem_rsvmap:    u32,
    version:           u32,
    last_comp_version: u32,
    boot_cpuid_phys:   u32,
    size_dt_strings:   u32,
    size_dt_struct:    u32,
}

#[inline(always)]
fn be32(x: u32) -> u32 {
    u32::from_be(x)
}

#[inline(always)]
fn align4(x: usize) -> usize {
    (x + 3) & !3
}

/// Lê um u32 big-endian sem garantia de alinhamento.
#[inline(always)]
unsafe fn read_be32(ptr: *const u8) -> u32 {
    let raw = core::ptr::read_unaligned(ptr as *const u32);
    u32::from_be(raw)
}

/// Lê uma string C (terminada em '\0') a partir de `ptr`.
/// Retorna `""` se não for UTF-8 válido.
unsafe fn read_cstr<'a>(ptr: *const u8) -> &'a str {
    let mut len = 0usize;
    loop {
        if core::ptr::read(ptr.add(len)) == 0 {
            break;
        }
        len += 1;
    }
    let bytes = core::slice::from_raw_parts(ptr, len);
    str::from_utf8(bytes).unwrap_or("")
}

// ---------------------------------------------------------------------------

pub struct Fdt {
    struct_block:  *const u8,
    strings_block: *const u8,
    struct_size:   usize,
}

impl Fdt {
    /// Cria uma view do FDT a partir do ponteiro recebido em x0 (dtb_ptr).
    ///
    /// # Safety
    /// `dtb_ptr` deve apontar para um FDT válido em memória acessível e
    /// permanecer mapeado durante toda a vida do `Fdt` retornado.
    pub unsafe fn from_ptr(dtb_ptr: usize) -> Option<Self> {
        if dtb_ptr == 0 {
            return None;
        }

        let base = dtb_ptr as *const u8;
        let header = &*(base as *const FdtHeader);

        if be32(header.magic) != FDT_MAGIC {
            return None;
        }

        let off_struct  = be32(header.off_dt_struct)  as usize;
        let off_strings = be32(header.off_dt_strings) as usize;
        let size_struct = be32(header.size_dt_struct) as usize;

        Some(Self {
            struct_block:  base.add(off_struct),
            strings_block: base.add(off_strings),
            struct_size:   size_struct,
        })
    }

    /// Retorna `/chosen/bootargs` como `&str`, se existir e for UTF-8 válido.
    /// O `\0` terminal da propriedade é removido automaticamente.
    ///
    /// # Safety
    /// Usa ponteiros crus vindos do DTB; o blob deve continuar mapeado
    /// enquanto o `&str` retornado estiver em uso.
    pub unsafe fn bootargs<'a>(&self) -> Option<&'a str> {
        let mut p  = self.struct_block;
        let end    = self.struct_block.add(self.struct_size);

        let mut depth     = 0usize;
        let mut in_chosen = false;

        while p < end {
            let token = read_be32(p);
            p = p.add(4);

            match token {
                FDT_BEGIN_NODE => {
                    let name     = read_cstr(p);
                    let name_len = name.len() + 1; // +1 pelo NUL
                    p = p.add(align4(name_len));

                    depth += 1;

                    // depth 1 = root "/" (nome vazio)
                    // depth 2 = filhos diretos do root
                    if depth == 2 && name == "chosen" {
                        in_chosen = true;
                    }
                }

                FDT_END_NODE => {
                    if in_chosen && depth == 2 {
                        // Saiu de /chosen sem encontrar bootargs
                        return None;
                    }

                    depth = depth.saturating_sub(1);
                }

                FDT_PROP => {
                    let prop_len = read_be32(p) as usize;
                    p = p.add(4);

                    let name_off = read_be32(p) as usize;
                    p = p.add(4);

                    let value_ptr = p;
                    p = p.add(align4(prop_len));

                    if !in_chosen {
                        continue;
                    }

                    let prop_name = read_cstr(self.strings_block.add(name_off));

                    if prop_name == "bootargs" {
                        if prop_len == 0 {
                            return None;
                        }

                        let raw = core::slice::from_raw_parts(value_ptr, prop_len);

                        // Remove o NUL terminal, se presente
                        let trimmed = match raw.last() {
                            Some(&0) => &raw[..prop_len - 1],
                            _        => raw,
                        };

                        return str::from_utf8(trimmed).ok();
                    }
                }

                FDT_NOP => {
                    // ignorado pela spec
                }

                FDT_END => break,

                _ => {
                    // Token desconhecido — blob corrompido ou versão futura
                    return None;
                }
            }
        }

        None
    }
}