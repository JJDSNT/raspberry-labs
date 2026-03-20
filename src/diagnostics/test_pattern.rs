// src/diagnostics/test_pattern.rs

use crate::demos::Demo;
use crate::gfx::renderer::Renderer;

pub struct TestPatternDiag;

impl TestPatternDiag {
    pub fn new() -> Self {
        Self
    }
}

impl Demo for TestPatternDiag {
    fn render(&mut self, renderer: &mut Renderer) {
        let w = renderer.width();
        let h = renderer.height();
        let buf = renderer.back_buffer();

        let black = argb(0,   0,   0  );
        let red   = argb(255, 0,   0  );
        let green = argb(0,   255, 0  );
        let blue  = argb(0,   0,   255);
        let white = argb(255, 255, 255);

        buf[..w * h].fill(black);

        let third = w / 3;
        fill(buf, w, h, 0,         0, third,                       h, red);
        fill(buf, w, h, third,     0, third,                       h, green);
        fill(buf, w, h, third * 2, 0, w.saturating_sub(third * 2), h, blue);

        let cx = w / 2;
        let cy = h / 2;
        fill(buf, w, h, cx.saturating_sub(16), cy.saturating_sub(16), 32, 32, white);
    }
}

#[inline]
fn fill(buf: &mut [u32], w: usize, h: usize,
        x: usize, y: usize, rw: usize, rh: usize, color: u32) {
    let x1 = (x + rw).min(w);
    let y1 = (y + rh).min(h);
    for row in y..y1 {
        buf[row * w + x..row * w + x1].fill(color);
    }
}

#[inline]
fn argb(r: u8, g: u8, b: u8) -> u32 {
    0xFF00_0000 | ((r as u32) << 16) | ((g as u32) << 8) | (b as u32)
}