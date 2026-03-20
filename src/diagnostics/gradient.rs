// src/diagnostics/gradient.rs

use crate::demos::Demo;
use crate::gfx::renderer::Renderer;
use crate::media::FrameContext;

pub struct GradientDiag;

impl GradientDiag {
    pub fn new() -> Self {
        Self
    }
}

impl Demo for GradientDiag {
    fn render(&mut self, renderer: &mut Renderer, _frame: &FrameContext) {
        let w = renderer.width();
        let h = renderer.height();
        let width_max  = (w - 1).max(1);
        let height_max = (h - 1).max(1);
        let buf = renderer.back_buffer();

        for y in 0..h {
            for x in 0..w {
                let r = ((x * 255) / width_max)  as u8;
                let g = ((y * 255) / height_max) as u8;
                let b = 128u8;
                buf[y * w + x] = 0xFF00_0000
                    | ((r as u32) << 16)
                    | ((g as u32) << 8)
                    | (b as u32);
            }
        }
    }
}