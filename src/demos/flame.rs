// src/demos/flame.rs

use crate::demos::Demo;
use crate::gfx::renderer::{MAX_HEIGHT, MAX_WIDTH, Renderer};
use crate::media::FrameContext;

// 8 linhas extras afastam a fornalha da área visível — o calor decai
// naturalmente antes de chegar à última linha renderizada.
const EXTRA_ROWS: usize = 8;
const MAX_HEAT: usize = MAX_WIDTH * (MAX_HEIGHT + 8);
const FIRE_LEVELS: usize = 256;

// Calibração: heat real na tela fica em ~0..220
const HEAT_MAX: u32 = 220;

// Fornalha
const FURNACE_BASE_MIN: u32 = 100;
const FURNACE_BASE_RND: u32 = 80;
const FURNACE_SPARK: u8 = 220;

// Buffer grande fora da stack.
static mut HEAT: [u8; MAX_HEAT] = [0; MAX_HEAT];

#[inline]
fn heat_slice_mut() -> &'static mut [u8; MAX_HEAT] {
    unsafe { &mut HEAT }
}

#[inline]
fn heat_slice() -> &'static [u8; MAX_HEAT] {
    unsafe { &HEAT }
}

struct Lcg(u32);

impl Lcg {
    #[inline]
    fn next(&mut self) -> u32 {
        self.0 = self.0.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        self.0
    }

    #[inline]
    fn next_mod(&mut self, n: u32) -> u32 {
        if n == 0 {
            return 0;
        }
        self.next() % n
    }
}

pub struct FlameDemo {
    palette: [u32; FIRE_LEVELS],
    rng: Lcg,
    seeded: bool,
}

impl FlameDemo {
    pub fn new() -> Self {
        let mut demo = Self {
            palette: [0; FIRE_LEVELS],
            rng: Lcg(0xCAFE_BABE),
            seeded: false,
        };
        demo.build_palette();
        demo
    }

    pub fn render(&mut self, renderer: &mut Renderer, _frame: &FrameContext) {
        let w = renderer.width().min(MAX_WIDTH);
        let h = renderer.height().min(MAX_HEIGHT);

        if w == 0 || h == 0 {
            return;
        }

        if !self.seeded {
            let heat = heat_slice_mut();
            heat[..w * (h + EXTRA_ROWS)].fill(0);
            self.seeded = true;
        }

        self.feed_furnace(w, h);
        self.propagate(w, h);
        self.draw(renderer, w, h);
    }

    fn feed_furnace(&mut self, w: usize, h: usize) {
        let base = h * w;
        let heat = heat_slice_mut();

        for x in 0..w {
            let v = FURNACE_BASE_MIN + self.rng.next_mod(FURNACE_BASE_RND);
            heat[base + x] = v.min(255) as u8;
        }

        let num_sparks = 30 + self.rng.next_mod(40);
        for _ in 0..num_sparks {
            if w < 3 {
                break;
            }

            let pos = self.rng.next_mod((w - 3) as u32) as usize;
            for dy in 0..3usize {
                let row = (h + 1 + dy) * w;
                for dx in 0..3usize {
                    let idx = row + pos + dx;
                    if idx < MAX_HEAT {
                        heat[idx] = FURNACE_SPARK;
                    }
                }
            }
        }
    }

    fn propagate(&mut self, w: usize, h: usize) {
        let count = w * (h + 2) - 2;
        let heat = heat_slice_mut();

        for i in 0..count {
            let s = 2 * w + i;
            let d = w + i;

            if s + 2 * w + 1 >= MAX_HEAT {
                break;
            }

            let avg = (heat[s + w] as u32
                + heat[s + 2 * w - 1] as u32
                + heat[s + 2 * w] as u32
                + heat[s + 2 * w + 1] as u32)
                >> 2;

            heat[d] = if avg > 0 { (avg - 1) as u8 } else { 0 };
        }
    }

    fn draw(&self, renderer: &mut Renderer, w: usize, h: usize) {
        let buf = renderer.back_buffer();
        let heat = heat_slice();

        for i in 0..(w * h).min(buf.len()) {
            buf[i] = self.palette[heat[i] as usize];
        }
    }

    // Paleta fiel ao Amiga original (init_colormap).
    // O azul aparece em duas regiões:
    //   i < 8   → azul crescente (índices muito baixos, bordas frias)
    //   i > 57  → azul crescente de novo (depois do amarelo → branco azulado)
    // Isso produz o efeito da foto 3: bordas das chamas com toque azul.
    //
    // Remapeamos i → t = i * 255 / HEAT_MAX para cobrir a faixa real de calor.
    fn build_palette(&mut self) {
        for i in 0..FIRE_LEVELS {
            let v = ((i as u32) * 255 / HEAT_MAX).min(255);

            let r = if v > 7 && v < 32 {
                10 * (v - 7)
            } else if v >= 32 {
                255
            } else {
                0
            };

            let g = if v > 32 && v < 57 {
                10 * (v - 32)
            } else if v >= 57 {
                255
            } else {
                0
            };

            let b = if v < 8 {
                8 * v
            } else if v >= 8 && v < 17 {
                8 * (16 - v)
            } else if v > 57 && v < 82 {
                10 * (v - 57)
            } else if v >= 82 {
                255
            } else {
                0
            };

            self.palette[i] = argb(
                0xFF,
                r.min(255) as u8,
                g.min(255) as u8,
                b.min(255) as u8,
            );
        }

        self.palette[0] = argb(0xFF, 0, 0, 0);
    }
}

impl Demo for FlameDemo {
    fn render(&mut self, renderer: &mut Renderer, frame: &FrameContext) {
        FlameDemo::render(self, renderer, frame);
    }
}

#[inline]
fn argb(a: u8, r: u8, g: u8, b: u8) -> u32 {
    ((a as u32) << 24) | ((r as u32) << 16) | ((g as u32) << 8) | (b as u32)
}