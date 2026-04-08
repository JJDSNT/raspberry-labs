// src/fs/fat32.rs
// Leitor FAT32 read-only.
// Encontra um arquivo por nome no diretório raiz e lê seu conteúdo
// em um buffer fornecido pelo chamador.
//
// Suporta nomes curtos (8.3) e longos (LFN, comparação case-insensitive ASCII).

use crate::drivers::sdcard as emmc;

const BLOCK: usize = 512;

// ---------------------------------------------------------------------------
// Estruturas on-disk (lidas via offsets, sem repr(C) para evitar padding)
// ---------------------------------------------------------------------------

fn u16le(buf: &[u8], off: usize) -> u16 {
    u16::from_le_bytes([buf[off], buf[off + 1]])
}

fn u32le(buf: &[u8], off: usize) -> u32 {
    u32::from_le_bytes([buf[off], buf[off + 1], buf[off + 2], buf[off + 3]])
}

// ---------------------------------------------------------------------------
// Estado do volume
// ---------------------------------------------------------------------------

struct Fat32 {
    part_lba:   u32, // LBA da partição FAT32
    fat_lba:    u32, // LBA da FAT
    data_lba:   u32, // LBA do início da área de dados (cluster 2)
    spc:        u32, // sectors per cluster
    root_clus:  u32, // cluster do diretório raiz
}

impl Fat32 {
    fn mount() -> Option<Self> {
        // --- MBR ---
        let mut mbr = [0u8; BLOCK];
        if !emmc::read_blocks(0, &mut mbr) { return None; }

        // Assinatura 0x55AA
        if mbr[510] != 0x55 || mbr[511] != 0xAA { return None; }

        // Primeira partição: offset 0x1BE, tipo em +4, LBA start em +8
        let p = 0x1BE;
        let part_type = mbr[p + 4];
        // 0x0B/0x0C = FAT32, 0x0E = FAT16 LBA (aceito para flexibilidade)
        if part_type != 0x0B && part_type != 0x0C && part_type != 0x0E {
            crate::log!("FAT32", "partition type {:#x} não suportado", part_type);
            return None;
        }
        let part_lba = u32le(&mbr, p + 8);

        // --- BPB (Volume Boot Record) ---
        let mut vbr = [0u8; BLOCK];
        if !emmc::read_blocks(part_lba, &mut vbr) { return None; }

        let bytes_per_sec = u16le(&vbr, 0x0B) as u32;
        if bytes_per_sec != 512 {
            crate::log!("FAT32", "bytes_per_sector={} (esperado 512)", bytes_per_sec);
            return None;
        }

        let spc          = vbr[0x0D] as u32;       // sectors per cluster
        let reserved     = u16le(&vbr, 0x0E) as u32; // reserved sectors
        let num_fats     = vbr[0x10] as u32;
        let fat_size     = u32le(&vbr, 0x24);       // sectors per FAT (FAT32)
        let root_clus    = u32le(&vbr, 0x2C);

        let fat_lba  = part_lba + reserved;
        let data_lba = fat_lba + num_fats * fat_size;

        crate::log!("FAT32", "part={} spc={} fat={} data={} root_clus={}",
            part_lba, spc, fat_lba, data_lba, root_clus);

        Some(Fat32 { part_lba, fat_lba, data_lba, spc, root_clus })
    }

    // LBA do primeiro setor do cluster N
    fn cluster_lba(&self, cluster: u32) -> u32 {
        self.data_lba + (cluster - 2) * self.spc
    }

    // Próximo cluster na FAT
    fn fat_next(&self, cluster: u32) -> u32 {
        let fat_off = cluster * 4;
        let fat_blk = self.fat_lba + fat_off / 512;
        let fat_idx = (fat_off % 512) as usize;
        let mut buf = [0u8; BLOCK];
        if !emmc::read_blocks(fat_blk, &mut buf) { return 0x0FFF_FFFF; }
        u32le(&buf, fat_idx) & 0x0FFF_FFFF
    }

    fn is_eof(cluster: u32) -> bool {
        cluster >= 0x0FFF_FFF8
    }

    // Busca arquivo por nome (case-insensitive) no diretório raiz.
    // Retorna (first_cluster, file_size).
    fn find_in_root(&self, name: &str) -> Option<(u32, u32)> {
        let name_upper = to_upper_ascii(name);

        let mut cluster = self.root_clus;
        let mut lfn_buf = [0u8; 256]; // UTF-8 do nome longo acumulado
        let mut lfn_len = 0usize;

        while !Self::is_eof(cluster) {
            let lba = self.cluster_lba(cluster);
            for sec in 0..self.spc {
                let mut buf = [0u8; BLOCK];
                if !emmc::read_blocks(lba + sec, &mut buf) { return None; }

                for e in 0..16usize { // 16 entradas de 32 bytes por setor
                    let off = e * 32;
                    let first = buf[off];
                    if first == 0x00 { return None; } // fim do diretório
                    if first == 0xE5 { lfn_len = 0; continue; } // deletado

                    let attr = buf[off + 11];

                    // Entrada LFN
                    if attr == 0x0F {
                        let order = buf[off] & 0x1F; // 1-based sequence
                        let chars = lfn_chars(&buf[off..off + 32]);
                        // Preenchemos de trás para frente (ordem LFN é invertida)
                        let start = ((order as usize) - 1) * 13;
                        if start + chars.len() <= lfn_buf.len() {
                            for (i, &c) in chars.iter().enumerate() {
                                lfn_buf[start + i] = c;
                            }
                            lfn_len = lfn_len.max(start + chars.len());
                        }
                        continue;
                    }

                    // Entrada normal (não Volume Label, não subdiretório neste passo)
                    if attr & 0x08 != 0 { lfn_len = 0; continue; } // volume label

                    // Tenta correspondência por LFN acumulado
                    let matched = if lfn_len > 0 {
                        let candidate = core::str::from_utf8(&lfn_buf[..lfn_len])
                            .unwrap_or("");
                        names_match(candidate, name)
                    } else {
                        // Compara pelo nome 8.3
                        let sfn = sfn_to_str(&buf[off..off + 11]);
                        let sfn_str = core::str::from_utf8(&sfn)
                            .unwrap_or("").trim_end_matches('\0');
                        names_match(sfn_str, name)
                    };
                    lfn_len = 0;

                    if matched {
                        let hi = u16le(&buf, off + 20) as u32;
                        let lo = u16le(&buf, off + 26) as u32;
                        let first_cluster = (hi << 16) | lo;
                        let size = u32le(&buf, off + 28);
                        return Some((first_cluster, size));
                    }
                }
            }
            cluster = self.fat_next(cluster);
        }
        None
    }

    // Lê o conteúdo completo de um arquivo no buffer `out`.
    // Retorna quantos bytes foram lidos.
    fn read_file(&self, first_cluster: u32, file_size: u32, out: &mut [u8]) -> usize {
        let to_read = (file_size as usize).min(out.len());
        let mut written = 0usize;
        let mut cluster = first_cluster;
        let cluster_bytes = (self.spc * 512) as usize;

        while !Self::is_eof(cluster) && written < to_read {
            let lba = self.cluster_lba(cluster);
            for sec in 0..self.spc {
                if written >= to_read { break; }
                let mut tmp = [0u8; BLOCK];
                if !emmc::read_blocks(lba + sec, &mut tmp) { return written; }
                let copy = BLOCK.min(to_read - written);
                out[written..written + copy].copy_from_slice(&tmp[..copy]);
                written += copy;
            }
            cluster = self.fat_next(cluster);
        }
        written
    }
}

// ---------------------------------------------------------------------------
// API pública
// ---------------------------------------------------------------------------

/// Abre o arquivo `name` na raiz do cartão SD e escreve o conteúdo em `out`.
/// Retorna quantos bytes foram escritos, ou 0 em caso de erro.
pub fn load(name: &str, out: &mut [u8]) -> usize {
    let fat = match Fat32::mount() {
        Some(f) => f,
        None => { crate::log!("FAT32", "mount failed"); return 0; }
    };
    let (cluster, size) = match fat.find_in_root(name) {
        Some(x) => x,
        None => { crate::log!("FAT32", "'{}' not found", name); return 0; }
    };
    crate::log!("FAT32", "'{}' cluster={} size={}", name, cluster, size);
    fat.read_file(cluster, size, out)
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Extrai os 13 caracteres UTF-16LE de uma entrada LFN, converte para UTF-8 básico.
/// Ignora code points > 0x7F (trata como '?').
fn lfn_chars(entry: &[u8]) -> [u8; 13] {
    let offsets: [usize; 13] = [1,3,5,7,9, 14,16,18,20,22,24, 28,30];
    let mut out = [0u8; 13];
    let mut len = 0;
    for &off in &offsets {
        let lo = entry[off];
        let hi = entry[off + 1];
        if lo == 0xFF && hi == 0xFF { break; } // padding
        if lo == 0x00 && hi == 0x00 { break; } // null terminator
        out[len] = if hi == 0 && lo < 0x80 { lo } else { b'?' };
        len += 1;
    }
    out
}

/// Converte nome 8.3 em string ("README  TXT" → "README.TXT")
fn sfn_to_str(raw: &[u8]) -> [u8; 13] {
    let mut out = [0u8; 13];
    let mut len = 0;
    for i in 0..8 {
        if raw[i] == b' ' { break; }
        out[len] = raw[i].to_ascii_uppercase();
        len += 1;
    }
    if raw[8] != b' ' {
        out[len] = b'.'; len += 1;
        for i in 8..11 {
            if raw[i] == b' ' { break; }
            out[len] = raw[i].to_ascii_uppercase();
            len += 1;
        }
    }
    out
}

fn to_upper_ascii(s: &str) -> [u8; 256] {
    let mut out = [0u8; 256];
    for (i, b) in s.bytes().take(255).enumerate() {
        out[i] = b.to_ascii_uppercase();
    }
    out
}

fn names_match(candidate: &str, target: &str) -> bool {
    if candidate.len() != target.len() { return false; }
    candidate.bytes().zip(target.bytes())
        .all(|(a, b)| a.to_ascii_uppercase() == b.to_ascii_uppercase())
}
