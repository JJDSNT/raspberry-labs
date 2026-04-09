// src/emu/launcher.rs
//
// Tela de seleção interativa para o emulador Omega2 (bare-metal).
// Útil para hardware real onde não há um launcher externo (ex: Go TUI).
//
// Controles USB HID:
//   UP / DOWN  — muda o slot ativo (DF0 / DF1 / HD0)
//   LEFT / RIGHT — cicla os arquivos disponíveis (inclui [none])
//   ENTER — confirma e lança o emulador

use crate::drivers::framebuffer::Framebuffer;
use crate::gfx::renderer::Renderer;
use crate::kernel::scheduler;

// ---------------------------------------------------------------------------
// Constantes
// ---------------------------------------------------------------------------

const MAX_FILES: usize = 16;
const NAME_LEN:  usize = 64;

// USB HID keycodes (boot protocol)
const KEY_ENTER: u8 = 0x28;
const KEY_RIGHT: u8 = 0x4F;
const KEY_LEFT:  u8 = 0x50;
const KEY_DOWN:  u8 = 0x51;
const KEY_UP:    u8 = 0x52;

// Paleta (0x00RRGGBB)
const C_BG:       u32 = 0x000A0F1A;
const C_BOX:      u32 = 0x00101B2E;
const C_BORDER:   u32 = 0x00305070;
const C_TITLE_BG: u32 = 0x00183050;
const C_TITLE:    u32 = 0x00FFD060;
const C_LABEL:    u32 = 0x007AAEFF;
const C_FILE:     u32 = 0x00FFCCAA;
const C_NONE:     u32 = 0x00667788;
const C_SEL_BG:   u32 = 0x00183860;
const C_SEL_LBL:  u32 = 0x00FFFFFF;
const C_SEL_FILE: u32 = 0x00FFFF88;
const C_HINT:     u32 = 0x00445566;
const C_ARROW:    u32 = 0x00779944;

const GW: usize = 8;
const GH: usize = 8;

// ---------------------------------------------------------------------------
// Resultado da seleção
// ---------------------------------------------------------------------------

pub struct LaunchConfig {
    pub df0:     [u8; NAME_LEN],
    pub df0_len: usize,
    pub df1:     [u8; NAME_LEN],
    pub df1_len: usize,
    pub hd0:     [u8; NAME_LEN],
    pub hd0_len: usize,
}

impl LaunchConfig {
    fn empty() -> Self {
        Self {
            df0: [0; NAME_LEN], df0_len: 0,
            df1: [0; NAME_LEN], df1_len: 0,
            hd0: [0; NAME_LEN], hd0_len: 0,
        }
    }

    pub fn df0_str(&self) -> Option<&str> {
        str_opt(&self.df0, self.df0_len)
    }
    pub fn df1_str(&self) -> Option<&str> {
        str_opt(&self.df1, self.df1_len)
    }
    pub fn hd0_str(&self) -> Option<&str> {
        str_opt(&self.hd0, self.hd0_len)
    }
}

fn str_opt(buf: &[u8; NAME_LEN], len: usize) -> Option<&str> {
    if len == 0 { None } else { core::str::from_utf8(&buf[..len]).ok() }
}

// ---------------------------------------------------------------------------
// FileList — lista de arquivos por extensão
// ---------------------------------------------------------------------------

struct FileList {
    names: [[u8; NAME_LEN]; MAX_FILES],
    lens:  [usize; MAX_FILES],
    count: usize,
}

impl FileList {
    const fn empty() -> Self {
        Self {
            names: [[0; NAME_LEN]; MAX_FILES],
            lens:  [0; MAX_FILES],
            count: 0,
        }
    }

    fn load(&mut self, ext: &str) {
        self.count = crate::fs::fat32::scan_ext(ext, &mut self.names, &mut self.lens);
    }

    fn name_at(&self, idx: usize) -> &str {
        if idx == 0 || idx > self.count { return ""; }
        core::str::from_utf8(&self.names[idx - 1][..self.lens[idx - 1]]).unwrap_or("")
    }

    fn copy_name(&self, sel: usize, dst: &mut [u8; NAME_LEN]) -> usize {
        let s = self.name_at(sel);
        let l = s.len().min(NAME_LEN);
        dst[..l].copy_from_slice(&s.as_bytes()[..l]);
        l
    }
}

// ---------------------------------------------------------------------------
// Ponto de entrada
// ---------------------------------------------------------------------------

/// Mostra o launcher interativo e devolve a configuração escolhida.
/// `fb` é consumido para criar o Renderer; o emulador usa os valores
/// já gravados em `host::set_framebuffer` antes desta chamada.
pub fn run(fb: Framebuffer) -> LaunchConfig {
    let mut r = Renderer::new(fb);

    let mut adf = FileList::empty();
    let mut hdf = FileList::empty();
    adf.load("adf");
    hdf.load("hdf");

    // Seleções independentes (0 = none, 1..=count = arquivo)
    let mut df0_sel: usize = if adf.count > 0 { 1 } else { 0 };
    let mut df1_sel: usize = 0;
    let mut hd0_sel: usize = if hdf.count > 0 { 1 } else { 0 };
    let mut cursor:  usize = 0; // 0=DF0, 1=DF1, 2=HD0

    loop {
        draw(&mut r, &adf, &hdf, df0_sel, df1_sel, hd0_sel, cursor);
        r.present();

        // Aguarda tecla pressionada (descarta key-up events)
        let sc = loop {
            if let Some((sc, pressed)) = crate::emu::host::poll_key() {
                if pressed { break sc; }
            }
            scheduler::yield_now();
        };

        match sc {
            KEY_UP   => { cursor = if cursor == 0 { 2 } else { cursor - 1 }; }
            KEY_DOWN => { cursor = if cursor == 2 { 0 } else { cursor + 1 }; }

            KEY_RIGHT => match cursor {
                0 => df0_sel = cycle_fwd(df0_sel, adf.count),
                1 => df1_sel = cycle_fwd(df1_sel, adf.count),
                2 => hd0_sel = cycle_fwd(hd0_sel, hdf.count),
                _ => {}
            },
            KEY_LEFT => match cursor {
                0 => df0_sel = cycle_bwd(df0_sel, adf.count),
                1 => df1_sel = cycle_bwd(df1_sel, adf.count),
                2 => hd0_sel = cycle_bwd(hd0_sel, hdf.count),
                _ => {}
            },

            KEY_ENTER => break,
            _ => {}
        }
    }

    let mut cfg = LaunchConfig::empty();
    cfg.df0_len = adf.copy_name(df0_sel, &mut cfg.df0);
    cfg.df1_len = adf.copy_name(df1_sel, &mut cfg.df1);
    cfg.hd0_len = hdf.copy_name(hd0_sel, &mut cfg.hd0);
    cfg
}

#[inline] fn cycle_fwd(sel: usize, max: usize) -> usize { if sel < max { sel + 1 } else { 0 } }
#[inline] fn cycle_bwd(sel: usize, max: usize) -> usize { if sel > 0  { sel - 1 } else { max } }

// ---------------------------------------------------------------------------
// Renderização
// ---------------------------------------------------------------------------

fn draw(
    r:       &mut Renderer,
    adf:     &FileList,
    hdf:     &FileList,
    df0_sel: usize,
    df1_sel: usize,
    hd0_sel: usize,
    cursor:  usize,
) {
    let sw = r.width();
    let sh = r.height();

    // Caixa: 52 chars × 10 linhas
    let bw = 52 * GW;
    let bh = 10 * GH;
    let bx = (sw - bw) / 2;
    let by = (sh - bh) / 2;

    r.clear(C_BG);

    // Borda exterior
    r.fill_rect(bx.saturating_sub(2), by.saturating_sub(2), bw + 4, bh + 4, C_BORDER);

    // Corpo
    r.fill_rect(bx, by, bw, bh, C_BOX);

    // Barra de título
    r.fill_rect(bx, by, bw, GH, C_TITLE_BG);
    draw_centered(&mut *r, bx, by, bw, "*** Omega2 Launcher ***", C_TITLE, C_TITLE_BG);

    // Slots
    draw_slot(r, bx, by + 2 * GH, "DF0", cursor == 0, adf, df0_sel);
    draw_slot(r, bx, by + 3 * GH, "DF1", cursor == 1, adf, df1_sel);
    draw_slot(r, bx, by + 4 * GH, "HD0", cursor == 2, hdf, hd0_sel);

    // Linha de ajuda
    let hy = by + 7 * GH;
    r.fill_rect(bx, hy, bw, GH, C_BOX);
    r.draw_str(bx + GW, hy, "UP/DN:slot  LT/RT:file  ENTER:boot", C_HINT, C_BOX);
}

/// Desenha um slot de seleção de arquivo.
fn draw_slot(
    r:      &mut Renderer,
    bx:     usize,
    ry:     usize,
    label:  &str,
    active: bool,
    list:   &FileList,
    sel:    usize,
) {
    const BOX_W: usize = 52 * GW;
    const MAX_NAME: usize = 38; // chars disponíveis para o nome

    let bg      = if active { C_SEL_BG  } else { C_BOX };
    let lbl_fg  = if active { C_SEL_LBL } else { C_LABEL };
    let file_fg = if active { C_SEL_FILE } else { C_FILE };

    r.fill_rect(bx, ry, BOX_W, GH, bg);

    // Rótulo + setas
    r.draw_str(bx + GW, ry, label, lbl_fg, bg);
    r.draw_str(bx + 5 * GW, ry, "<", C_ARROW, bg);
    r.draw_str(bx + (BOX_W / GW - 2) * GW, ry, ">", C_ARROW, bg);

    // Nome do arquivo
    let nx = bx + 7 * GW;
    if sel == 0 || list.count == 0 {
        r.draw_str(nx, ry, "[none]", C_NONE, bg);
    } else {
        let name  = list.name_at(sel);
        let trunc = trunc_str(name, MAX_NAME);
        r.draw_str(nx, ry, trunc, file_fg, bg);
    }

    // Contador sel/total
    let mut cbuf = [0u8; 8];
    let cstr = fmt_ratio(&mut cbuf, sel, list.count);
    let cx = bx + (BOX_W / GW - 9) * GW;
    r.draw_str(cx, ry, cstr, C_HINT, bg);
}

fn draw_centered(r: &mut Renderer, bx: usize, by: usize, bw: usize, s: &str, fg: u32, bg: u32) {
    let tw = s.len() * GW;
    let tx = bx + (bw.saturating_sub(tw)) / 2;
    r.draw_str(tx, by, s, fg, bg);
}

fn trunc_str(s: &str, max_chars: usize) -> &str {
    let b = s.as_bytes();
    if b.len() <= max_chars { s }
    else { core::str::from_utf8(&b[..max_chars]).unwrap_or(s) }
}

/// Formata "sel/total" em `buf`, retorna a fatia usada.
fn fmt_ratio<'a>(buf: &'a mut [u8; 8], sel: usize, total: usize) -> &'a str {
    let mut pos = 0usize;

    fn push_u(buf: &mut [u8; 8], pos: &mut usize, mut n: usize) {
        if n == 0 { if *pos < 8 { buf[*pos] = b'0'; *pos += 1; } return; }
        let start = *pos;
        while n > 0 && *pos < 8 { buf[*pos] = b'0' + (n % 10) as u8; n /= 10; *pos += 1; }
        buf[start..*pos].reverse();
    }

    push_u(buf, &mut pos, sel);
    if pos < 8 { buf[pos] = b'/'; pos += 1; }
    push_u(buf, &mut pos, total);

    core::str::from_utf8(&buf[..pos]).unwrap_or("?")
}
