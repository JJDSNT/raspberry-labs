// src/demos/starfield.rs
//
// Starfield clássico em software.
// Compatível com o renderer dinâmico e com o trait `Demo`.

use crate::demos::Demo;
use crate::gfx::renderer::Renderer;
use crate::media::FrameContext;

const STAR_COUNT: usize = 192;
const STARFIELD_DEPTH: i32 = 1024;
const STAR_MIN_Z: i32 = 8;
const STAR_SPEED: i32 = 8;
const PROJECTION_SCALE: i32 = 256;

#[derive(Clone, Copy)]
struct Star {
    x: i32,
    y: i32,
    z: i32,
}

pub struct StarfieldDemo {
    stars: [Star; STAR_COUNT],
    seed: u32,
    initialized: bool,
}

impl StarfieldDemo {
    pub fn new() -> Self {
        Self {
            stars: [Star { x: 0, y: 0, z: 1 }; STAR_COUNT],
            seed: 0x1234_5678,
            initialized: false,
        }
    }

    pub fn render(&mut self, renderer: &mut Renderer, _frame: &FrameContext) {
        let width = renderer.width();
        let height = renderer.height();

        if width < 2 || height < 2 {
            return;
        }

        if !self.initialized {
            self.init_stars(width, height);
            self.initialized = true;
        }

        renderer.clear_black();

        let cx = (width / 2) as i32;
        let cy = (height / 2) as i32;
        let spread_x = cx.max(1);
        let spread_y = cy.max(1);

        for i in 0..STAR_COUNT {
            let mut star = self.stars[i];

            star.z -= STAR_SPEED;

            if star.z <= STAR_MIN_Z {
                self.respawn_star(&mut star, spread_x, spread_y);
            }

            let sx = cx + (star.x * PROJECTION_SCALE) / star.z;
            let sy = cy + (star.y * PROJECTION_SCALE) / star.z;

            if sx < 0 || sy < 0 || sx >= width as i32 || sy >= height as i32 {
                self.respawn_star(&mut star, spread_x, spread_y);
            } else {
                let brightness = self.star_brightness(star.z);
                let color = gray(brightness);

                renderer.put_pixel(sx as usize, sy as usize, color);

                // Pequeno rastro para estrelas mais próximas.
                if brightness > 180 {
                    let tail_x = cx + (star.x * PROJECTION_SCALE) / (star.z + STAR_SPEED);
                    let tail_y = cy + (star.y * PROJECTION_SCALE) / (star.z + STAR_SPEED);

                    if tail_x >= 0
                        && tail_y >= 0
                        && tail_x < width as i32
                        && tail_y < height as i32
                    {
                        renderer.put_pixel(
                            tail_x as usize,
                            tail_y as usize,
                            gray(brightness.saturating_sub(80)),
                        );
                    }
                }
            }

            self.stars[i] = star;
        }
    }

    fn init_stars(&mut self, width: usize, height: usize) {
        let spread_x = (width as i32 / 2).max(1);
        let spread_y = (height as i32 / 2).max(1);

        for i in 0..STAR_COUNT {
            let mut star = Star { x: 0, y: 0, z: 1 };
            self.respawn_star_full(&mut star, spread_x, spread_y);
            // Distribui melhor em profundidade no frame inicial.
            star.z = STAR_MIN_Z + ((i as i32 * (STARFIELD_DEPTH - STAR_MIN_Z)) / STAR_COUNT as i32);
            self.stars[i] = star;
        }
    }

    fn respawn_star(&mut self, star: &mut Star, spread_x: i32, spread_y: i32) {
        star.x = self.rand_range(-spread_x, spread_x);
        star.y = self.rand_range(-spread_y, spread_y);
        star.z = STARFIELD_DEPTH;
    }

    fn respawn_star_full(&mut self, star: &mut Star, spread_x: i32, spread_y: i32) {
        star.x = self.rand_range(-spread_x, spread_x);
        star.y = self.rand_range(-spread_y, spread_y);
        star.z = self.rand_range(STAR_MIN_Z, STARFIELD_DEPTH);
    }

    #[inline]
    fn star_brightness(&self, z: i32) -> u8 {
        let depth_span = (STARFIELD_DEPTH - STAR_MIN_Z).max(1);
        let near_factor = STARFIELD_DEPTH - z;
        let v = 80 + ((near_factor * 175) / depth_span);
        v.clamp(0, 255) as u8
    }

    #[inline]
    fn rand_u32(&mut self) -> u32 {
        // LCG simples, suficiente para demo.
        self.seed = self
            .seed
            .wrapping_mul(1664525)
            .wrapping_add(1013904223);
        self.seed
    }

    #[inline]
    fn rand_range(&mut self, min_v: i32, max_v: i32) -> i32 {
        let span = (max_v - min_v).max(1) as u32;
        min_v + (self.rand_u32() % span) as i32
    }
}

impl Demo for StarfieldDemo {
    fn render(&mut self, renderer: &mut Renderer, _frame: &crate::demos::FrameContext) {
        StarfieldDemo::render(self, renderer, _frame);
    }
}

#[inline]
fn gray(v: u8) -> u32 {
    0xFF00_0000 | ((v as u32) << 16) | ((v as u32) << 8) | (v as u32)
}