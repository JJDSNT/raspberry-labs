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
// Fat32File — handle de arquivo aberto para leitura posicional (streaming)
// ---------------------------------------------------------------------------

/// Arquivo aberto para leitura. Mantém estado de seek na chain FAT para
/// evitar re-traversal a partir do início em cada leitura sequencial.
pub struct Fat32File {
    fs:            Fat32,
    first_cluster: u32,
    pub file_size: u32,
    cur_cluster:   u32,
    cur_byte_off:  u64,  // byte offset do início de cur_cluster no arquivo
}

impl Fat32File {
    fn new(fs: Fat32, first_cluster: u32, file_size: u32) -> Self {
        Self { cur_cluster: first_cluster, cur_byte_off: 0, fs, first_cluster, file_size }
    }

    /// Lê `buf.len()` bytes a partir do byte `offset` do arquivo.
    /// Retorna quantos bytes foram efetivamente lidos.
    pub fn read_at(&mut self, offset: u64, buf: &mut [u8]) -> usize {
        if offset >= self.file_size as u64 || buf.is_empty() { return 0; }

        let to_read      = buf.len().min((self.file_size as u64 - offset) as usize);
        let cluster_bytes = self.fs.spc as u64 * 512;

        // Seek para trás: recomeça da cadeia
        if offset < self.cur_byte_off {
            self.cur_cluster  = self.first_cluster;
            self.cur_byte_off = 0;
        }

        // Avança clusters até chegar no que contém `offset`
        while self.cur_byte_off + cluster_bytes <= offset {
            if Fat32::is_eof(self.cur_cluster) { return 0; }
            self.cur_cluster   = self.fs.fat_next(self.cur_cluster);
            self.cur_byte_off += cluster_bytes;
        }

        let mut dst = 0usize;
        let mut pos = offset;

        while dst < to_read && !Fat32::is_eof(self.cur_cluster) {
            let within = (pos - self.cur_byte_off) as usize;
            let lba    = self.fs.cluster_lba(self.cur_cluster);
            let sec0   = within / 512;
            let boff0  = within % 512;

            for s in sec0..self.fs.spc as usize {
                if dst >= to_read { break; }
                let mut tmp = [0u8; 512];
                if !emmc::read_blocks(lba + s as u32, &mut tmp) { return dst; }
                let start = if s == sec0 { boff0 } else { 0 };
                let copy  = (512 - start).min(to_read - dst);
                buf[dst..dst + copy].copy_from_slice(&tmp[start..start + copy]);
                dst += copy;
                pos += copy as u64;
            }

            if dst < to_read {
                self.cur_cluster   = self.fs.fat_next(self.cur_cluster);
                self.cur_byte_off += cluster_bytes;
            }
        }

        dst
    }
}

// ---------------------------------------------------------------------------
// API pública
// ---------------------------------------------------------------------------

/// Abre `name` no diretório raiz do SD e retorna um handle para leitura posicional.
/// Retorna None se o arquivo não for encontrado ou o SD não estiver disponível.
pub fn open_file(name: &str) -> Option<Fat32File> {
    let fat = Fat32::mount()?;
    let (first_cluster, file_size) = fat.find_in_root(name)?;
    crate::log!("FAT32", "open '{}' cluster={} size={}", name, first_cluster, file_size);
    Some(Fat32File::new(fat, first_cluster, file_size))
}

/// Escaneia o diretório raiz e devolve o nome do primeiro arquivo com
/// extensão `.hdf` encontrado em `out`. Retorna o comprimento do nome,
/// ou 0 se nenhum arquivo for encontrado.
pub fn find_first_hdf(out: &mut [u8]) -> usize {
    let fat = match Fat32::mount() {
        Some(f) => f,
        None => return 0,
    };

    let mut cluster  = fat.root_clus;
    let mut lfn_buf  = [0u8; 256];
    let mut lfn_len  = 0usize;

    while !Fat32::is_eof(cluster) {
        let lba = fat.cluster_lba(cluster);
        for sec in 0..fat.spc {
            let mut buf = [0u8; BLOCK];
            if !emmc::read_blocks(lba + sec, &mut buf) { return 0; }

            for e in 0..16usize {
                let off   = e * 32;
                let first = buf[off];
                if first == 0x00 { return 0; }
                if first == 0xE5 { lfn_len = 0; continue; }

                let attr = buf[off + 11];
                if attr == 0x0F {
                    let order = buf[off] & 0x1F;
                    let chars = lfn_chars(&buf[off..off + 32]);
                    let start = ((order as usize) - 1) * 13;
                    if start + chars.len() <= lfn_buf.len() {
                        for (i, &c) in chars.iter().enumerate() {
                            lfn_buf[start + i] = c;
                        }
                        lfn_len = lfn_len.max(start + chars.len());
                    }
                    continue;
                }

                if attr & 0x08 != 0 || attr & 0x10 != 0 { lfn_len = 0; continue; } // label/dir

                // Verifica extensão: LFN acumulado ou extensão do SFN
                let is_hdf = if lfn_len > 0 {
                    has_ext(&lfn_buf, lfn_len, b"hdf")
                } else {
                    // Extensão SFN está em bytes 8-10
                    let ext = &buf[off + 8..off + 11];
                    let etrim = ext[..ext.iter().position(|&b| b == b' ').unwrap_or(3)].iter();
                    let hdf   = b"HDF".iter();
                    etrim.zip(hdf).all(|(&a, &b)| a.to_ascii_uppercase() == b)
                        && ext.iter().position(|&b| b == b' ').unwrap_or(3) == 3
                };

                if is_hdf {
                    // Copia nome efetivo para out
                    let name_len = if lfn_len > 0 {
                        let copy = lfn_len.min(out.len());
                        out[..copy].copy_from_slice(&lfn_buf[..copy]);
                        copy
                    } else {
                        let sfn = sfn_to_str(&buf[off..off + 11]);
                        let slen = sfn.iter().position(|&b| b == 0).unwrap_or(13);
                        let copy = slen.min(out.len());
                        out[..copy].copy_from_slice(&sfn[..copy]);
                        copy
                    };
                    lfn_len = 0;
                    return name_len;
                }

                lfn_len = 0;
            }
        }
        cluster = fat.fat_next(cluster);
    }

    0
}

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

/// Retorna true se `name[..len]` termina com ".<ext>" (case-insensitive ASCII).
fn has_ext(name: &[u8], len: usize, ext: &[u8]) -> bool {
    let dot_ext_len = ext.len() + 1;
    if len < dot_ext_len + 1 { return false; }
    let dot_pos = len - dot_ext_len;
    if name[dot_pos] != b'.' { return false; }
    name[dot_pos + 1..dot_pos + 1 + ext.len()]
        .iter()
        .zip(ext.iter())
        .all(|(&a, &b)| a.to_ascii_uppercase() == b.to_ascii_uppercase())
}

/// Retorna true se a extensão SFN (raw[8..11]) bate com `ext` (ex: "adf").
fn sfn_has_ext(raw: &[u8], ext: &str) -> bool {
    let ext_bytes = ext.as_bytes();
    if ext_bytes.len() > 3 { return false; }
    let sfn_ext     = &raw[8..11];
    let sfn_ext_len = sfn_ext.iter().position(|&b| b == b' ').unwrap_or(3);
    if sfn_ext_len != ext_bytes.len() { return false; }
    sfn_ext[..sfn_ext_len].iter().zip(ext_bytes.iter())
        .all(|(&a, &b)| a.to_ascii_uppercase() == b.to_ascii_uppercase())
}

/// Escaneia o diretório raiz do SD e coleta todos os arquivos com extensão `ext`
/// (sem o ponto, ex: "adf" ou "hdf"). Preenche `name_buf[i][..name_len[i]]`
/// para cada arquivo encontrado. Retorna quantos foram encontrados (≤ name_buf.len()).
pub fn scan_ext(
    ext:      &str,
    name_buf: &mut [[u8; 64]],
    name_len: &mut [usize],
) -> usize {
    let max = name_buf.len().min(name_len.len());
    if max == 0 { return 0; }

    let fat = match Fat32::mount() {
        Some(f) => f,
        None    => return 0,
    };

    let mut found    = 0usize;
    let mut cluster  = fat.root_clus;
    let mut lfn_buf  = [0u8; 256];
    let mut lfn_len  = 0usize;

    'scan: while !Fat32::is_eof(cluster) {
        let lba = fat.cluster_lba(cluster);
        for sec in 0..fat.spc {
            let mut buf = [0u8; BLOCK];
            if !emmc::read_blocks(lba + sec, &mut buf) { break 'scan; }

            for e in 0..16usize {
                let off   = e * 32;
                let first = buf[off];
                if first == 0x00 { break 'scan; } // fim do diretório
                if first == 0xE5 { lfn_len = 0; continue; } // deletado

                let attr = buf[off + 11];

                // Entrada LFN
                if attr == 0x0F {
                    let order = buf[off] & 0x1F;
                    let chars = lfn_chars(&buf[off..off + 32]);
                    let start = ((order as usize) - 1) * 13;
                    if start + chars.len() <= lfn_buf.len() {
                        for (i, &c) in chars.iter().enumerate() {
                            lfn_buf[start + i] = c;
                        }
                        lfn_len = lfn_len.max(start + chars.len());
                    }
                    continue;
                }

                if attr & 0x08 != 0 || attr & 0x10 != 0 { lfn_len = 0; continue; }

                // Verifica extensão
                let is_match = if lfn_len > 0 {
                    has_ext(&lfn_buf, lfn_len, ext.as_bytes())
                } else {
                    sfn_has_ext(&buf[off..off + 11], ext)
                };

                if is_match {
                    if found < max {
                        if lfn_len > 0 {
                            let l = lfn_len.min(64);
                            name_buf[found][..l].copy_from_slice(&lfn_buf[..l]);
                            name_len[found] = l;
                        } else {
                            let sfn  = sfn_to_str(&buf[off..off + 11]);
                            let l    = sfn.iter().position(|&b| b == 0).unwrap_or(13).min(64);
                            name_buf[found][..l].copy_from_slice(&sfn[..l]);
                            name_len[found] = l;
                        }
                        found += 1;
                        if found == max { break 'scan; }
                    }
                }

                lfn_len = 0;
            }
        }
        cluster = fat.fat_next(cluster);
    }

    found
}
