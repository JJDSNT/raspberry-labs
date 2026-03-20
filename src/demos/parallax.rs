// src/demos/parallax.rs
//
// Parallax — portado do original Amiga (AROS demos collection)
//
// Algoritmo por frame:
//   1. c -= 2  (scroll horizontal automático)
//   2. Copia p[] → buf[] com wrap em (y-100, x+c) — exibe textura com scroll
//   3. Reduz p[0..128][0..128]: downscale 2x + brighten (/2 + 40)
//   4. Espelha horizontalmente: metade direita = metade esquerda
//   5. Espelha verticalmente:   metade inferior = metade superior
//   6. Injeta padrão XOR de senos em p[] na posição (c, d)
//
// Paleta: escala de cinza em um canal — cicla entre azul → vermelho → verde.
// No original, o clique do mouse trocava a paleta; aqui troca automaticamente
// a cada ~300 frames.

use crate::demos::Demo;
use crate::gfx::renderer::Renderer;

// Dimensões internas fixas do algoritmo — idênticas ao original.
const P_SIZE: usize = 256;               // textura interna 256×256
const P_LEN:  usize = P_SIZE * P_SIZE;   // 65536 bytes

// Ciclo de troca de paleta (frames)
const PAL_CYCLE: u32 = 300;

pub struct ParallaxDemo {
    /// Textura interna 256×256 — se auto-modifica a cada frame.
    p: [u8; P_LEN],
    /// Tabela de senos pré-calculada: s[a] = 32 - sin(π·a/128)·31
    s: [u8; P_SIZE],
    /// Offset horizontal (scroll automático, decrementa 2/frame)
    c: u32,
    /// Offset vertical
    d: u32,
    /// Canal de cor ativo: 0=vermelho, 1=verde, 2=azul
    pal: u8,
    /// Contador de frames para ciclo de paleta
    frame: u32,
}

impl ParallaxDemo {
    pub fn new() -> Self {
        let mut s = [0u8; P_SIZE];
        for a in 0..P_SIZE {
            let v = 32.0 - libm::sinf(core::f32::consts::PI * a as f32 / 128.0) * 31.0;
            s[a] = v as u8;
        }

        Self {
            p:     [0u8; P_LEN],
            s,
            c:     0,
            d:     200,
            pal:   2, // azul, como no original
            frame: 0,
        }
    }

    fn render_frame(&mut self, renderer: &mut Renderer) {
        let w = renderer.width();
        let h = renderer.height();
        if w == 0 || h == 0 { return; }

        // Troca de paleta automática a cada PAL_CYCLE frames
        self.frame = self.frame.wrapping_add(1);
        if self.frame % PAL_CYCLE == 0 {
            self.pal = (self.pal + 1) % 3;
        }

        // 1. Scroll horizontal
        self.c = self.c.wrapping_sub(2);

        // 2. Exibe p[] no back-buffer com scroll e wrap
        {
            let buf = renderer.back_buffer();
            let c = self.c as u8;
            let mut idx = 0usize;
            for y in 0..h {
                for x in 0..w {
                    let py = ((y as isize - 100).rem_euclid(P_SIZE as isize)) as usize;
                    let px = ((x as u8).wrapping_add(c)) as usize;
                    let pal_idx = self.p[py * P_SIZE + px];
                    buf[idx] = self.pal_color(pal_idx);
                    idx += 1;
                }
            }
        }

        // 3. Downscale 2x + brighten: p[y*256+x] = p[y*512+x*2]/2 + 40
        //    Opera apenas no quadrante superior esquerdo 128×128
        for y in 0..128usize {
            for x in 0..128usize {
                let src = self.p[y * P_SIZE * 2 + x * 2];
                self.p[y * P_SIZE + x] = src.wrapping_div(2).wrapping_add(40);
            }
        }

        // 4. Espelha horizontalmente: p[y][128..256] = p[y][0..128]
        for y in 0..128usize {
            let row = y * P_SIZE;
            // memmove(p + y*256 + 128, p + y*256, 128)
            for x in 0..128usize {
                self.p[row + 128 + x] = self.p[row + x];
            }
        }

        // 5. Espelha verticalmente: p[128..256][] = p[0..128][]
        //    memmove(p + 128*256, p, 32768)
        self.p.copy_within(0..32768, 32768);

        // 6. Injeta padrão XOR de senos em p[] na posição (c, d)
        //    a = (s[x]^s[y]) & s[(x^y)&255]
        //    if a > 37: p[(d+y)&255 * 256 + (c+x)&255] = (a*8 - 30) as u8 (wrapping)
        let c8 = self.c as u8;
        let d8 = self.d as u8;
        for y in 0..P_SIZE {
            for x in 0..P_SIZE {
                let a = (self.s[x] ^ self.s[y]) & self.s[(x ^ y) & 255];
                if a > 37 {
                    let py = d8.wrapping_add(y as u8) as usize;
                    let px = c8.wrapping_add(x as u8) as usize;
                    // wrapping u8 — idêntico ao comportamento do C original
                    self.p[py * P_SIZE + px] = (a as u16 * 8).wrapping_sub(30) as u8;
                }
            }
        }
    }

    /// Converte um índice de paleta (0..255) para ARGB32 no canal ativo.
    #[inline]
    fn pal_color(&self, idx: u8) -> u32 {
        let v = idx as u32;
        match self.pal {
            0 => 0xFF00_0000 | (v << 16),           // vermelho
            1 => 0xFF00_0000 | (v << 8),            // verde
            _ => 0xFF00_0000 | v,                   // azul (default)
        }
    }
}

impl Demo for ParallaxDemo {
    fn render(&mut self, renderer: &mut Renderer) {
        self.render_frame(renderer);
    }
}