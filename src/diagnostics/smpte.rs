// src/diagnostics/smpte.rs
//
// Padrão de Teste Picasso RTG — baseado em análise pixel-a-pixel da imagem.
//
// Coordenadas originais em 640x480, escaladas proporcionalmente.
//
// Layout:
//   Grade branca 40x40 sobre fundo preto
//   5 elipses brancas (1 central + 4 cantos)
//   8 barras EBU 75%   — x=114..474, y=70..175
//   Escala de cinza    — x=114..474, y=175..240
//   Tarja + resolução  — y=244..275
//   Padrão de burst    — y=280..315
//   Linha vertical central — y=315..390
//   Faixa inferior     — y=390..430 (vermelho | preto | verde | preto | azul)
//   Wedges             — y=430..470

use crate::demos::Demo;
use crate::gfx::font;
use crate::gfx::font_data::FONT_8X8;
use crate::gfx::renderer::Renderer;
use crate::media::FrameContext;

pub struct SmpteDiag;
impl SmpteDiag { pub fn new() -> Self { Self } }

const BLACK: u32 = 0xFF_00_00_00;
const WHITE: u32 = 0xFF_FF_FF_FF;

// 8 barras EBU 75% (0xBF = 191)
const BARS: [u32; 8] = [
    0xFF_BF_BF_BF, // cinza claro
    0xFF_BF_BF_00, // amarelo
    0xFF_00_BF_BF, // ciano
    0xFF_00_BF_00, // verde
    0xFF_BF_00_BF, // magenta
    0xFF_BF_00_00, // vermelho
    0xFF_00_00_BF, // azul
    0xFF_40_40_40, // cinza escuro
];

impl Demo for SmpteDiag {
    fn render(&mut self, renderer: &mut Renderer, _frame: &FrameContext) {
        let w = renderer.width();
        let h = renderer.height();
        if w == 0 || h == 0 { return; }

        let sx = |x: usize| x * w / 640;
        let sy = |y: usize| y * h / 480;

        // ---------------------------------------------------------------
        // 1. Fundo preto
        // ---------------------------------------------------------------
        renderer.clear(BLACK);

        // ---------------------------------------------------------------
        // 2. Grade branca 40x40
        // ---------------------------------------------------------------
        let gx = (40 * w / 640).max(1);
        let gy = (40 * h / 480).max(1);
        let mut x = 0usize;
        while x <= w { renderer.vline(x.min(w-1), 0, h-1, WHITE); x += gx; }
        let mut y = 0usize;
        while y <= h { renderer.hline(y.min(h-1), 0, w-1, WHITE); y += gy; }

        // ---------------------------------------------------------------
        // 3. 5 elipses brancas
        // ---------------------------------------------------------------
        // Central — quase toda a tela
        renderer.draw_ellipse((w/2) as i32, (h/2) as i32,
            sx(310) as i32, sy(225) as i32, WHITE);
        // 4 cantos — elipses achatadas que cortam a borda
        let erx = sx(75) as i32;
        let ery = sy(60) as i32;
        renderer.draw_ellipse(sx(80)  as i32, sy(75)  as i32, erx, ery, WHITE);
        renderer.draw_ellipse(sx(560) as i32, sy(75)  as i32, erx, ery, WHITE);
        renderer.draw_ellipse(sx(80)  as i32, sy(405) as i32, erx, ery, WHITE);
        renderer.draw_ellipse(sx(560) as i32, sy(405) as i32, erx, ery, WHITE);

        // ---------------------------------------------------------------
        // 4. 8 barras de cor EBU — x=114..474, y=70..175
        // ---------------------------------------------------------------
        let bar_x0 = sx(114);
        let bar_x1 = sx(474);
        let bar_y0 = sy(70);
        let bar_h  = sy(175).saturating_sub(bar_y0).max(1);
        let bar_w  = (bar_x1 - bar_x0) / BARS.len();

        for (i, &c) in BARS.iter().enumerate() {
            let bx = bar_x0 + i * bar_w;
            let bw = if i == BARS.len()-1 { bar_x1 - bx } else { bar_w };
            renderer.fill_rect(bx, bar_y0, bw, bar_h, c);
        }

        // ---------------------------------------------------------------
        // 5. Escala de cinza — x=114..474, y=175..240
        // ---------------------------------------------------------------
        let gray_y0 = sy(175);
        let gray_h  = sy(240).saturating_sub(gray_y0).max(1);
        // 8 passos: 0, 32, 64, 96, 128, 160, 192, 224
        for i in 0..8usize {
            let v   = (i * 32) as u8;
            let gx0 = bar_x0 + i * bar_w;
            let gw  = if i == 7 { bar_x1 - gx0 } else { bar_w };
            renderer.fill_rect(gx0, gray_y0, gw, gray_h, gray(v));
        }

        // ---------------------------------------------------------------
        // 6. Área branca de teste — x=114..474, y=240..390
        // ---------------------------------------------------------------
        let test_y0 = sy(240);
        let test_h  = sy(390).saturating_sub(test_y0).max(1);
        renderer.fill_rect(bar_x0, test_y0, bar_x1 - bar_x0, test_h, WHITE);

        // ---------------------------------------------------------------
        // 7. Tarja preta + texto resolução — y=244..276
        // ---------------------------------------------------------------
        let tx  = sx(195);
        let ty  = sy(244);
        let tw  = sx(250);
        let th  = sy(276).saturating_sub(ty).max(1);
        renderer.fill_rect(tx, ty, tw, th, BLACK);

        let res    = build_res_str(w, h);
        let rw_px  = res.len() * font::GLYPH_W * 2;
        let rx     = tx + tw.saturating_sub(rw_px) / 2;
        let ry     = ty + th.saturating_sub(font::GLYPH_H * 2) / 2;
        draw_str_x2(renderer, rx, ry, res, WHITE);

        // ---------------------------------------------------------------
        // 8. Padrão de burst — y=280..315
        // ---------------------------------------------------------------
        let burst_y0 = sy(280);
        let burst_h  = sy(315).saturating_sub(burst_y0).max(1);
        let burst_x0 = bar_x0;
        let burst_w  = bar_x1 - bar_x0;
        let half     = burst_w / 2;

        // Metade esquerda: linhas verticais finas (brancas e pretas alternadas)
        let mut bx = burst_x0;
        let mut on = true;
        while bx < burst_x0 + half {
            let c = if on { BLACK } else { WHITE };
            renderer.vline(bx, burst_y0, burst_y0 + burst_h - 1, c);
            bx += 1;
            on = !on;
        }

        // Metade direita: linhas horizontais
        let step = (sy(4)).max(2);
        let mut by = burst_y0;
        let mut on = true;
        while by < burst_y0 + burst_h {
            let yend = (by + step - 1).min(burst_y0 + burst_h - 1);
            let c = if on { BLACK } else { WHITE };
            for yy in by..=yend {
                renderer.hline(yy, burst_x0 + half, burst_x0 + burst_w - 1, c);
            }
            by += step;
            on = !on;
        }

        // Linha vertical central descendo do burst até a faixa inferior
        let center_x = w / 2;
        renderer.vline(center_x, sy(315), sy(390), BLACK);

        // ---------------------------------------------------------------
        // 9. Faixa de gradientes — y=390..430
        // vermelho (preto→vermelho) | verde (preto→verde) | azul (preto→azul)
        // ---------------------------------------------------------------
        let grad_y = sy(390);
        let grad_h = sy(430).saturating_sub(grad_y).max(1);

        // Limites de cada gradiente (medidos da imagem)
        let r_x0 = sx(112); let r_x1 = sx(250);
        let g_x0 = sx(262); let g_x1 = sx(387);
        let b_x0 = sx(400); let b_x1 = sx(524);

        // Vermelho: preto → vermelho
        for px in r_x0..r_x1 {
            let t = (px - r_x0) * 255 / (r_x1 - r_x0).max(1);
            renderer.fill_rect(px, grad_y, 1, grad_h, 0xFF_00_00_00 | ((t as u32) << 16));
        }
        // Verde: preto → verde
        for px in g_x0..g_x1 {
            let t = (px - g_x0) * 255 / (g_x1 - g_x0).max(1);
            renderer.fill_rect(px, grad_y, 1, grad_h, 0xFF_00_00_00 | ((t as u32) << 8));
        }
        // Azul: preto → azul
        for px in b_x0..b_x1 {
            let t = (px - b_x0) * 255 / (b_x1 - b_x0).max(1);
            renderer.fill_rect(px, grad_y, 1, grad_h, 0xFF_00_00_00 | (t as u32));
        }

        // ---------------------------------------------------------------
        // 10. Wedges de convergência — y=430..470
        // ---------------------------------------------------------------
        let wx  = sx(180);
        let wy  = sy(430);
        let ww  = sx(460) - wx;
        let wh  = sy(470).saturating_sub(wy).max(1);
        let mid_x = wx + ww / 2;
        let mid_y = (wy + wy + wh) / 2;

        renderer.fill_rect(wx, wy, ww, wh, BLACK);

        // Wedge esquerdo
        draw_line_r(renderer, wx + 4,      wy + 3,      mid_x, mid_y, WHITE);
        draw_line_r(renderer, wx + 4,      wy + wh - 4, mid_x, mid_y, WHITE);
        draw_line_r(renderer, wx + ww/5,   wy + 3,      mid_x, mid_y, WHITE);
        draw_line_r(renderer, wx + ww/5,   wy + wh - 4, mid_x, mid_y, WHITE);
        draw_line_r(renderer, wx + ww*2/5, wy + 3,      mid_x, mid_y, WHITE);
        draw_line_r(renderer, wx + ww*2/5, wy + wh - 4, mid_x, mid_y, WHITE);

        // Wedge direito (espelho)
        draw_line_r(renderer, wx + ww - 5,      wy + 3,      mid_x, mid_y, WHITE);
        draw_line_r(renderer, wx + ww - 5,      wy + wh - 4, mid_x, mid_y, WHITE);
        draw_line_r(renderer, wx + ww*4/5,      wy + 3,      mid_x, mid_y, WHITE);
        draw_line_r(renderer, wx + ww*4/5,      wy + wh - 4, mid_x, mid_y, WHITE);
        draw_line_r(renderer, wx + ww*3/5,      wy + 3,      mid_x, mid_y, WHITE);
        draw_line_r(renderer, wx + ww*3/5,      wy + wh - 4, mid_x, mid_y, WHITE);
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

#[inline] fn gray(v: u8) -> u32 { 0xFF_00_00_00 | ((v as u32)*0x10101) }

fn build_res_str(w: usize, h: usize) -> &'static str {
    static mut RES_BUF: [u8; 20] = [0u8; 20];
    unsafe {
        let buf = &mut RES_BUF;
        let mut pos = 0usize;
        pos += write_usize(buf, pos, w);
        buf[pos] = b'x'; pos += 1;
        pos += write_usize(buf, pos, h);
        buf[pos] = b'x'; pos += 1;
        buf[pos] = b'2'; pos += 1;
        buf[pos] = b'4'; pos += 1;
        core::str::from_utf8(&buf[..pos]).unwrap_or("?")
    }
}

fn write_usize(buf: &mut [u8], pos: usize, mut n: usize) -> usize {
    let mut tmp = [0u8; 10];
    let mut len = 0usize;
    if n == 0 { buf[pos] = b'0'; return 1; }
    while n > 0 { tmp[len] = b'0' + (n % 10) as u8; n /= 10; len += 1; }
    for i in 0..len { buf[pos + i] = tmp[len - 1 - i]; }
    len
}

fn draw_str_x2(renderer: &mut Renderer, x: usize, y: usize, s: &str, color: u32) {
    let mut cx = x;
    for ch in s.chars() {
        let code  = ch as u32;
        let idx   = if (0x20..=0x7F).contains(&code) { (code-0x20) as usize } else { 0 };
        let glyph = &FONT_8X8[idx * font::GLYPH_H..(idx+1) * font::GLYPH_H];
        for (row, &byte) in glyph.iter().enumerate() {
            for col in 0..font::GLYPH_W {
                if (byte >> (7-col)) & 1 != 0 {
                    renderer.fill_rect(cx + col*2, y + row*2, 2, 2, color);
                }
            }
        }
        cx += font::GLYPH_W * 2;
    }
}

fn draw_line_r(renderer: &mut Renderer, x0: usize, y0: usize, x1: usize, y1: usize, color: u32) {
    let mut x0 = x0 as isize;
    let mut y0 = y0 as isize;
    let x1 = x1 as isize;
    let y1 = y1 as isize;
    let dx = (x1-x0).abs(); let sx = if x0<x1 {1} else {-1};
    let dy = -(y1-y0).abs(); let sy = if y0<y1 {1} else {-1};
    let mut err = dx+dy;
    loop {
        if x0>=0 && y0>=0 { renderer.put_pixel(x0 as usize, y0 as usize, color); }
        if x0==x1 && y0==y1 { break; }
        let e2 = 2*err;
        if e2 >= dy { err += dy; x0 += sx; }
        if e2 <= dx { err += dx; y0 += sy; }
    }
}