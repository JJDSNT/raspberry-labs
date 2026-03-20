// src/gfx/font.rs
//
// Rasterizador de fonte bitmap 8×8 para bare metal.
//
// A fonte cobre ASCII 0x20 (espaço) até 0x7F (bloco) — 96 glifos.
// Cada glifo é 8 bytes, 1 bit por pixel, MSB = pixel mais à esquerda.
//
// Os dados da fonte estão em font_data.rs como array estático.
// Este módulo só rasteriza — não mantém estado de cursor.
// Para cursor e scroll, veja platform/console.rs.
//
// API:
//   draw_char(buf, w, h, x, y, ch, fg, bg)
//   draw_str(buf, w, h, x, y, s, fg, bg)
//   draw_str_transparent(buf, w, h, x, y, s, fg)
//
// Constantes:
//   GLYPH_W = 8
//   GLYPH_H = 8

use crate::gfx::font_data::FONT_8X8;

pub const GLYPH_W: usize = 8;
pub const GLYPH_H: usize = 8;

// Primeiro caractere coberto pela fonte
const FONT_FIRST: u8 = 0x20; // espaço
const FONT_LAST:  u8 = 0x7F; // bloco

// -----------------------------------------------------------------------
// API pública
// -----------------------------------------------------------------------

/// Desenha um único caractere em (x, y) com cor de frente e fundo.
///
/// Pixels "ligados" recebem `fg`, pixels "desligados" recebem `bg`.
/// Use `draw_char_transparent` se não quiser pintar o fundo.
pub fn draw_char(
    buf: &mut [u32], w: usize, h: usize,
    x: usize, y: usize,
    ch: char,
    fg: u32, bg: u32,
) {
    let glyph = glyph_for(ch);
    render_glyph(buf, w, h, x, y, glyph, fg, Some(bg));
}

/// Desenha um único caractere sem pintar os pixels de fundo (transparente).
pub fn draw_char_transparent(
    buf: &mut [u32], w: usize, h: usize,
    x: usize, y: usize,
    ch: char,
    fg: u32,
) {
    let glyph = glyph_for(ch);
    render_glyph(buf, w, h, x, y, glyph, fg, None);
}

/// Desenha uma string em (x, y) avançando horizontalmente.
/// Não faz wrap — caracteres que ultrapassem `w` são clipados.
pub fn draw_str(
    buf: &mut [u32], w: usize, h: usize,
    x: usize, y: usize,
    s: &str,
    fg: u32, bg: u32,
) {
    let mut cx = x;
    for ch in s.chars() {
        if cx + GLYPH_W > w { break; }
        draw_char(buf, w, h, cx, y, ch, fg, bg);
        cx += GLYPH_W;
    }
}

/// Desenha uma string sem fundo (transparente).
pub fn draw_str_transparent(
    buf: &mut [u32], w: usize, h: usize,
    x: usize, y: usize,
    s: &str,
    fg: u32,
) {
    let mut cx = x;
    for ch in s.chars() {
        if cx + GLYPH_W > w { break; }
        draw_char_transparent(buf, w, h, cx, y, ch, fg);
        cx += GLYPH_W;
    }
}

/// Largura em pixels de uma string (sem wrap).
#[inline]
pub fn str_width(s: &str) -> usize {
    s.chars().count() * GLYPH_W
}

// -----------------------------------------------------------------------
// Helpers internos
// -----------------------------------------------------------------------

/// Retorna os 8 bytes do glifo para o caractere dado.
/// Caracteres fora do range retornam o glifo do espaço.
#[inline]
fn glyph_for(ch: char) -> &'static [u8] {
    let code = ch as u32;
    let idx = if code >= FONT_FIRST as u32 && code <= FONT_LAST as u32 {
        (code - FONT_FIRST as u32) as usize
    } else {
        0 // espaço como fallback
    };
    &FONT_8X8[idx * GLYPH_H..(idx + 1) * GLYPH_H]
}

/// Rasteriza um glifo no buffer.
/// `bg = None` → transparente (não escreve pixels apagados).
#[inline]
fn render_glyph(
    buf: &mut [u32], w: usize, h: usize,
    x: usize, y: usize,
    glyph: &[u8],
    fg: u32,
    bg: Option<u32>,
) {
    for (row, &byte) in glyph.iter().enumerate() {
        let py = y + row;
        if py >= h { break; }

        for col in 0..GLYPH_W {
            let px = x + col;
            if px >= w { break; }

            // MSB = pixel mais à esquerda
            let bit = (byte >> (7 - col)) & 1;
            let color = if bit != 0 {
                fg
            } else {
                match bg {
                    Some(c) => c,
                    None => continue,
                }
            };

            buf[py * w + px] = color;
        }
    }
}