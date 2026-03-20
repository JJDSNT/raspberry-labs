// src/demos/plasma.rs
//
// Plasma effect clássico para o demo engine bare-metal.
// Renderiza usando aritmética inteira e lookup table de seno.

use crate::gfx::renderer::Renderer;
use crate::media::FrameContext;

const TABLE_SIZE: usize = 256;

/// Demo de plasma.
pub struct Plasma {
    time: u16,
    sin_table: [i16; TABLE_SIZE],
    palette: [u32; 256],
}

impl Plasma {
    pub fn new() -> Self {
        let mut plasma = Self {
            time: 0,
            sin_table: [0; TABLE_SIZE],
            palette: [0; 256],
        };

        plasma.build_sin_table();
        plasma.build_palette();
        plasma
    }

    /// Renderiza um frame.
    pub fn render(&mut self, renderer: &mut Renderer, _frame: &FrameContext) {
        let width = renderer.width();
        let height = renderer.height();
        let t = self.time as usize;
        let buf = renderer.back_buffer();

        for y in 0..height {
            let sy1 = self.sin((y * 4 + t) & 0xFF);
            let sy2 = self.sin((y * 2 + (t * 3)) & 0xFF);

            for x in 0..width {
                let sx1 = self.sin((x * 3 + t) & 0xFF);
                let sx2 = self.sin(((x + y) * 2 + (t * 2)) & 0xFF);
                let sx3 = self.sin(self.radial_index(x, y, t, width, height));

                let v = sx1 as i32
                    + sy1 as i32
                    + sx2 as i32
                    + sy2 as i32
                    + sx3 as i32;

                // Faixa aproximada: 5 ondas em [-128, 127] => ~[-640, 635]
                let color_index = (((v + 640) * 255) / 1280) as usize & 0xFF;

                buf[y * width + x] = self.palette[color_index];
            }
        }

        self.time = self.time.wrapping_add(1);
    }

    #[inline]
    fn sin(&self, idx: usize) -> i16 {
        self.sin_table[idx & 0xFF]
    }

    #[inline]
    fn radial_index(&self, x: usize, y: usize, t: usize, width: usize, height: usize) -> usize {
        let cx = (width / 2) as isize;
        let cy = (height / 2) as isize;

        let dx = x as isize - cx;
        let dy = y as isize - cy;

        // Aproximação barata de distância:
        // dist ≈ max(dx,dy) + min(dx,dy)/2
        let ax = abs_i(dx) as usize;
        let ay = abs_i(dy) as usize;

        let dist = if ax > ay {
            ax + (ay >> 1)
        } else {
            ay + (ax >> 1)
        };

        (dist * 8 + t * 4) & 0xFF
    }

    fn build_sin_table(&mut self) {
        for i in 0..TABLE_SIZE {
            self.sin_table[i] = fast_sin_256(i as u8);
        }
    }

    fn build_palette(&mut self) {
        for i in 0..256 {
            let c = i as u8;

            let r = wave8(c);
            let g = wave8(c.wrapping_add(85));
            let b = wave8(c.wrapping_add(170));

            self.palette[i] = argb(0xFF, r, g, b);
        }
    }
}

#[inline]
fn argb(a: u8, r: u8, g: u8, b: u8) -> u32 {
    ((a as u32) << 24)
        | ((r as u32) << 16)
        | ((g as u32) << 8)
        | (b as u32)
}

#[inline]
fn abs_i(v: isize) -> isize {
    if v < 0 { -v } else { v }
}

/// Onda de 0..255 baseada em uma senoide aproximada.
#[inline]
fn wave8(x: u8) -> u8 {
    (fast_sin_256(x) + 128) as u8
}

/// Seno aproximado em 256 passos, saída em [-128, 127].
///
/// Implementação inteira, sem float.
/// Baseada em uma onda triangular com suavização quadrática leve.
fn fast_sin_256(x: u8) -> i16 {
    let p = x as i16;

    let tri = if p < 64 {
        p * 2
    } else if p < 128 {
        127 - (p - 64) * 2
    } else if p < 192 {
        -((p - 128) * 2)
    } else {
        -127 + (p - 192) * 2
    };

    let a = if tri < 0 { -tri } else { tri };
    (tri * (192 - a)) / 128
}

impl crate::demos::Demo for Plasma {
    fn render(&mut self, renderer: &mut crate::gfx::renderer::Renderer, _frame: &crate::demos::FrameContext) {
        Plasma::render(self, renderer, _frame);
    }
}