// src/demos/flame.rs
//
// Efeito de fogo / chama em software, estilo clássico.
// Usa um buffer de intensidades e uma paleta ARGB.
//
// Compatível com o renderer dinâmico e com o trait `Demo`.

use crate::demos::Demo;
use crate::gfx::renderer::{MAX_HEIGHT, MAX_WIDTH, Renderer};

const MAX_PIXELS: usize = MAX_WIDTH * MAX_HEIGHT;
const FIRE_LEVELS: usize = 256;

/// Demo de fogo.
pub struct FlameDemo {
    heat: [u8; MAX_PIXELS],
    palette: [u32; FIRE_LEVELS],
    tick: u32,
    seeded: bool,
}

impl FlameDemo {
    pub fn new() -> Self {
        let mut demo = Self {
            heat: [0; MAX_PIXELS],
            palette: [0; FIRE_LEVELS],
            tick: 0,
            seeded: false,
        };

        demo.build_palette();
        demo
    }

    pub fn render(&mut self, renderer: &mut Renderer) {
        let width = renderer.width();
        let height = renderer.height();
        let pixels = width * height;

        if width == 0 || height == 0 {
            return;
        }

        if !self.seeded {
            self.clear_heat(pixels);
            self.seeded = true;
        }

        self.inject_base(width, height);
        self.propagate(width, height);

        let buf = renderer.back_buffer();
        for i in 0..pixels {
            buf[i] = self.palette[self.heat[i] as usize];
        }

        self.tick = self.tick.wrapping_add(1);
    }

    fn clear_heat(&mut self, pixels: usize) {
        self.heat[..pixels].fill(0);
    }

    fn inject_base(&mut self, width: usize, height: usize) {
        let last_row = height - 1;
        let row_start = last_row * width;

        for x in 0..width {
            let idx = row_start + x;

            // Base do fogo com pequenas variações pseudo-periódicas.
            let phase = ((x as u32)
                .wrapping_mul(13)
                .wrapping_add(self.tick.wrapping_mul(7)))
                & 31;

            let value = if phase < 24 { 255 } else { 200 };
            self.heat[idx] = value as u8;
        }
    }

    fn propagate(&mut self, width: usize, height: usize) {
        if height < 2 || width < 2 {
            return;
        }

        // Atualiza de baixo para cima, ignorando a última linha
        // (que é a fonte do fogo).
        for y in 1..height {
            let row = y * width;
            let dst_row = (y - 1) * width;

            for x in 0..width {
                let src_idx = row + x;
                let src = self.heat[src_idx] as u16;

                // Pequena variação horizontal determinística.
                let noise = ((x as u32)
                    .wrapping_mul(17)
                    .wrapping_add(y as u32 * 31)
                    .wrapping_add(self.tick * 13))
                    & 3;

                let decay = 1 + (noise as u16);
                let cooled = src.saturating_sub(decay as u16) as u8;

                let dst_x = x.saturating_sub(noise as usize >> 1);
                let dst_idx = dst_row + dst_x.min(width - 1);

                self.heat[dst_idx] = cooled;
            }
        }
    }

    fn build_palette(&mut self) {
        for i in 0..FIRE_LEVELS {
            let c = i as u8;

            let color = if c < 64 {
                // preto -> vermelho
                argb(0xFF, c.saturating_mul(4), 0, 0)
            } else if c < 128 {
                // vermelho -> laranja
                let t = c - 64;
                argb(0xFF, 255, t.saturating_mul(4), 0)
            } else if c < 192 {
                // laranja -> amarelo
                let t = c - 128;
                argb(0xFF, 255, 255, t.saturating_mul(4))
            } else {
                // amarelo -> branco
                let t = c - 192;
                let boost = t.saturating_mul(4);
                argb(0xFF, 255, 255, 255u8.saturating_sub(63 - boost.min(63)))
            };

            self.palette[i] = color;
        }

        self.palette[0] = argb(0xFF, 0, 0, 0);
    }
}

impl Demo for FlameDemo {
    fn render(&mut self, renderer: &mut Renderer) {
        FlameDemo::render(self, renderer);
    }
}

#[inline]
fn argb(a: u8, r: u8, g: u8, b: u8) -> u32 {
    ((a as u32) << 24)
        | ((r as u32) << 16)
        | ((g as u32) << 8)
        | (b as u32)
}