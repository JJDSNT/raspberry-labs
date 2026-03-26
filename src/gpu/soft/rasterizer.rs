#![allow(dead_code)]

use libm::{ceilf, floorf};

use crate::gfx3d::command::Vertex;

use super::framebuffer::SoftFramebuffer;

pub struct SoftwareRasterizer<'a> {
    fb: SoftFramebuffer<'a>,
}

impl<'a> SoftwareRasterizer<'a> {
    pub fn new(fb: SoftFramebuffer<'a>) -> Self {
        Self { fb }
    }

    pub fn framebuffer(&self) -> &SoftFramebuffer<'a> {
        &self.fb
    }

    pub fn framebuffer_mut(&mut self) -> &mut SoftFramebuffer<'a> {
        &mut self.fb
    }

    pub fn clear(&mut self, color: u32) {
        self.fb.clear(color);
    }

    pub fn draw_triangle(&mut self, v0: Vertex, v1: Vertex, v2: Vertex) {
        if self.fb.width() == 0 || self.fb.height() == 0 {
            return;
        }

        let min_x = floor3(v0.x, v1.x, v2.x).max(0.0) as i32;
        let min_y = floor3(v0.y, v1.y, v2.y).max(0.0) as i32;
        let max_x = ceil3(v0.x, v1.x, v2.x).min((self.fb.width() - 1) as f32) as i32;
        let max_y = ceil3(v0.y, v1.y, v2.y).min((self.fb.height() - 1) as f32) as i32;

        if min_x > max_x || min_y > max_y {
            return;
        }

        let area = edge_function(v0.x, v0.y, v1.x, v1.y, v2.x, v2.y);
        if area == 0.0 {
            return;
        }

        let inv_area = 1.0 / area;

        for y in min_y..=max_y {
            for x in min_x..=max_x {
                let px = x as f32 + 0.5;
                let py = y as f32 + 0.5;

                let w0 = edge_function(v1.x, v1.y, v2.x, v2.y, px, py);
                let w1 = edge_function(v2.x, v2.y, v0.x, v0.y, px, py);
                let w2 = edge_function(v0.x, v0.y, v1.x, v1.y, px, py);

                let inside = if area > 0.0 {
                    w0 >= 0.0 && w1 >= 0.0 && w2 >= 0.0
                } else {
                    w0 <= 0.0 && w1 <= 0.0 && w2 <= 0.0
                };

                if !inside {
                    continue;
                }

                let b0 = w0 * inv_area;
                let b1 = w1 * inv_area;
                let b2 = w2 * inv_area;

                let color = interpolate_color(v0.color, v1.color, v2.color, b0, b1, b2);
                self.fb.put_pixel(x, y, color);
            }
        }
    }
}

fn edge_function(ax: f32, ay: f32, bx: f32, by: f32, px: f32, py: f32) -> f32 {
    (px - ax) * (by - ay) - (py - ay) * (bx - ax)
}

fn floor3(a: f32, b: f32, c: f32) -> f32 {
    floorf(a.min(b).min(c))
}

fn ceil3(a: f32, b: f32, c: f32) -> f32 {
    ceilf(a.max(b).max(c))
}

fn interpolate_color(c0: u32, c1: u32, c2: u32, w0: f32, w1: f32, w2: f32) -> u32 {
    let (r0, g0, b0, a0) = unpack_rgba8(c0);
    let (r1, g1, b1, a1) = unpack_rgba8(c1);
    let (r2, g2, b2, a2) = unpack_rgba8(c2);

    let r = clamp_to_u8(r0 as f32 * w0 + r1 as f32 * w1 + r2 as f32 * w2);
    let g = clamp_to_u8(g0 as f32 * w0 + g1 as f32 * w1 + g2 as f32 * w2);
    let b = clamp_to_u8(b0 as f32 * w0 + b1 as f32 * w1 + b2 as f32 * w2);
    let a = clamp_to_u8(a0 as f32 * w0 + a1 as f32 * w1 + a2 as f32 * w2);

    pack_rgba8(r, g, b, a)
}

fn clamp_to_u8(x: f32) -> u8 {
    if x <= 0.0 {
        0
    } else if x >= 255.0 {
        255
    } else {
        x as u8
    }
}

fn unpack_rgba8(color: u32) -> (u8, u8, u8, u8) {
    let r = ((color >> 24) & 0xff) as u8;
    let g = ((color >> 16) & 0xff) as u8;
    let b = ((color >> 8) & 0xff) as u8;
    let a = (color & 0xff) as u8;
    (r, g, b, a)
}

fn pack_rgba8(r: u8, g: u8, b: u8, a: u8) -> u32 {
    ((r as u32) << 24) | ((g as u32) << 16) | ((b as u32) << 8) | (a as u32)
}