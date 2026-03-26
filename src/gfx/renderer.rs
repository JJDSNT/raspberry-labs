// src/gfx/renderer.rs
//
// Renderer — orquestrador de frames para o demo engine bare-metal (aarch64 / RPi)
// Coordena framebuffer, blitter, copper, sprites e primitives para produzir cada frame.

use core::sync::atomic::{AtomicBool, Ordering};

use crate::drivers::framebuffer::Framebuffer;
use crate::gfx::blitter::Blitter;
use crate::gfx::copper::CopperList;
use crate::gfx::font;
use crate::gfx::primitives;
use crate::gfx::sprite::{sprite_pixel, SpriteBatch, SpriteInstance};

// ---------------------------------------------------------------------------
// Limites máximos do renderer
// ---------------------------------------------------------------------------

pub const MAX_WIDTH: usize = 1024;
pub const MAX_HEIGHT: usize = 768;
pub const BYTES_PER_PIXEL: usize = 4; // ARGB8888
pub const MAX_PIXELS: usize = MAX_WIDTH * MAX_HEIGHT;
pub const FRAMEBUFFER_SIZE: usize = MAX_PIXELS * BYTES_PER_PIXEL;

const COPPER_CAPACITY: usize = 256;

// ---------------------------------------------------------------------------
// Buffers estáticos (.bss) com Proteção de Memória
// ---------------------------------------------------------------------------

#[repr(align(16))]
struct SafeBuffer {
    data: [[u32; MAX_PIXELS]; 2],
    // Padding de segurança para evitar que overflow de pixels
    // atropele outras estruturas globais no kernel.
    _padding: [u8; 1024],
}

static mut RENDER_BUFFERS: SafeBuffer = SafeBuffer {
    data: [[0; MAX_PIXELS]; 2],
    _padding: [0; 1024],
};

static RENDERER_TAKEN: AtomicBool = AtomicBool::new(false);

// ---------------------------------------------------------------------------
// Double-buffer Enum
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq, Eq)]
enum BufferIndex {
    Front = 0,
    Back = 1,
}

impl BufferIndex {
    #[inline]
    fn flip(self) -> Self {
        match self {
            BufferIndex::Front => BufferIndex::Back,
            BufferIndex::Back => BufferIndex::Front,
        }
    }

    #[inline]
    fn as_usize(self) -> usize {
        self as usize
    }
}

// ---------------------------------------------------------------------------
// Renderer Struct
// ---------------------------------------------------------------------------

pub struct Renderer {
    // Campos de controle primeiro (mais seguros contra corrupção de ponteiros)
    width: usize,
    height: usize,
    pixels: usize,
    draw: BufferIndex,
    frame_count: u64,

    // Componentes de hardware/software
    fb: Framebuffer,
    blitter: Blitter,
    copper: CopperList<COPPER_CAPACITY>,

    // Buffers por último
    buffers: &'static mut [[u32; MAX_PIXELS]; 2],
}

impl Renderer {
    pub fn new(fb: Framebuffer) -> Self {
        let already_taken = RENDERER_TAKEN.swap(true, Ordering::AcqRel);
        assert!(!already_taken, "Renderer::new() called more than once");

        let width = fb.width as usize;
        let height = fb.height as usize;
        let pixels = width.saturating_mul(height);

        assert!(
            width <= MAX_WIDTH && height <= MAX_HEIGHT,
            "Resolution exceeds static buffer"
        );
        assert!(pixels <= MAX_PIXELS, "Pixel count exceeds static buffer");

        Self {
            width,
            height,
            pixels,
            draw: BufferIndex::Back,
            frame_count: 0,
            fb,
            blitter: Blitter::new(width, height),
            copper: CopperList::new(),
            buffers: unsafe { &mut RENDER_BUFFERS.data },
        }
    }

    // --- Helpers de Buffer ---

    #[inline]
    fn back_index(&self) -> usize {
        self.draw.as_usize()
    }

    #[inline]
    fn front_index(&self) -> usize {
        self.draw.flip().as_usize()
    }

    #[inline]
    fn back_buffer_full(&mut self) -> &mut [u32; MAX_PIXELS] {
        &mut self.buffers[self.back_index()]
    }

    // --- API de Frame ---

    #[inline]
    pub fn back_buffer(&mut self) -> &mut [u32] {
        let p = self.pixels;
        &mut self.back_buffer_full()[..p]
    }

    #[inline]
    pub fn front_buffer(&self) -> &[u32] {
        &self.buffers[self.front_index()][..self.pixels]
    }

    #[inline]
    pub fn clear(&mut self, color: u32) {
        self.back_buffer().fill(color);
    }

    #[inline]
    pub fn clear_black(&mut self) {
        self.clear(0xFF00_0000);
    }

    pub fn present(&mut self) {
        let idx = self.back_index();
        let limit = self.pixels.min(MAX_PIXELS);
        self.fb.blit_argb(&self.buffers[idx][..limit]);
        self.draw = self.draw.flip();
        self.frame_count = self.frame_count.wrapping_add(1);
    }

    // --- Primitivas de Pixel e Blitter ---

    #[inline]
    pub fn put_pixel(&mut self, x: usize, y: usize, color: u32) {
        if x < self.width && y < self.height {
            let idx = y * self.width + x;
            if idx < self.pixels {
                self.buffers[self.back_index()][idx] = color;
            }
        }
    }

    #[inline]
    pub fn get_pixel(&self, x: usize, y: usize) -> u32 {
        if x < self.width && y < self.height {
            let idx = y * self.width + x;
            if idx < self.pixels {
                self.buffers[self.back_index()][idx]
            } else {
                0
            }
        } else {
            0
        }
    }

    pub fn hline(&mut self, y: usize, x0: usize, x1: usize, color: u32) {
        let idx = self.back_index();
        let p = self.pixels;
        self.blitter
            .hline(&mut self.buffers[idx][..p], y, x0, x1, color);
    }

    pub fn vline(&mut self, x: usize, y0: usize, y1: usize, color: u32) {
        let idx = self.back_index();
        let p = self.pixels;
        self.blitter
            .vline(&mut self.buffers[idx][..p], x, y0, y1, color);
    }

    pub fn fill_rect(&mut self, x: usize, y: usize, w: usize, h: usize, color: u32) {
        let idx = self.back_index();
        let p = self.pixels;
        self.blitter
            .fill_rect(&mut self.buffers[idx][..p], x, y, w, h, color);
    }

    // --- Primitivas Geométricas (primitives.rs) ---

    pub fn draw_line(&mut self, x0: i32, y0: i32, x1: i32, y1: i32, color: u32) {
        let idx = self.back_index();
        let p = self.pixels;
        primitives::line(
            &mut self.buffers[idx][..p],
            self.width,
            self.height,
            x0,
            y0,
            x1,
            y1,
            color,
        );
    }

    pub fn draw_circle(&mut self, cx: i32, cy: i32, r: i32, color: u32) {
        let idx = self.back_index();
        let p = self.pixels;
        primitives::circle(
            &mut self.buffers[idx][..p],
            self.width,
            self.height,
            cx,
            cy,
            r,
            color,
        );
    }

    pub fn fill_circle(&mut self, cx: i32, cy: i32, r: i32, color: u32) {
        let idx = self.back_index();
        let p = self.pixels;
        primitives::fill_circle(
            &mut self.buffers[idx][..p],
            self.width,
            self.height,
            cx,
            cy,
            r,
            color,
        );
    }

    pub fn draw_ellipse(&mut self, cx: i32, cy: i32, rx: i32, ry: i32, color: u32) {
        let idx = self.back_index();
        let p = self.pixels;
        primitives::ellipse(
            &mut self.buffers[idx][..p],
            self.width,
            self.height,
            cx,
            cy,
            rx,
            ry,
            color,
        );
    }

    pub fn fill_ellipse(&mut self, cx: i32, cy: i32, rx: i32, ry: i32, color: u32) {
        let idx = self.back_index();
        let p = self.pixels;
        primitives::fill_ellipse(
            &mut self.buffers[idx][..p],
            self.width,
            self.height,
            cx,
            cy,
            rx,
            ry,
            color,
        );
    }

    // --- Texto (font.rs) ---

    pub fn draw_str(&mut self, x: usize, y: usize, s: &str, fg: u32, bg: u32) {
        let idx = self.back_index();
        let p = self.pixels;
        font::draw_str(&mut self.buffers[idx][..p], self.width, self.height, x, y, s, fg, bg);
    }

    pub fn draw_str_transparent(&mut self, x: usize, y: usize, s: &str, fg: u32) {
        let idx = self.back_index();
        let p = self.pixels;
        font::draw_str_transparent(
            &mut self.buffers[idx][..p],
            self.width,
            self.height,
            x,
            y,
            s,
            fg,
        );
    }

    // --- Sprites ---

    pub fn draw_sprite_instance(&mut self, instance: &SpriteInstance<'_>) {
        if !instance.visible || !instance.is_valid() {
            return;
        }

        let width = self.width;
        let height = self.height;
        let pixels = self.pixels;
        let back_idx = self.back_index();
        let buffer = &mut self.buffers[back_idx];

        // Trabalha tudo em i32 até a última conversão.
        let sprite_w = instance.src.w as i32;
        let sprite_h = instance.src.h as i32;

        // Usa saturating_add para evitar overflow de i32 em posições extremas.
        let raw_x1 = instance.x.saturating_add(sprite_w);
        let raw_y1 = instance.y.saturating_add(sprite_h);

        let x0 = instance.x.max(0);
        let y0 = instance.y.max(0);
        let x1 = raw_x1.min(width as i32);
        let y1 = raw_y1.min(height as i32);

        // Totalmente fora da tela ou retângulo inválido.
        if x0 >= x1 || y0 >= y1 {
            return;
        }

        for sy in y0..y1 {
            let syu = sy as usize;
            let row_base = syu * width;

            // Como sy está dentro da interseção clipada, ly fica seguro.
            let ly_i32 = sy - instance.y;
            if ly_i32 < 0 {
                continue;
            }
            let ly = ly_i32 as usize;

            for sx in x0..x1 {
                let sxu = sx as usize;

                let lx_i32 = sx - instance.x;
                if lx_i32 < 0 {
                    continue;
                }
                let lx = lx_i32 as usize;

                if let Some(src_px) = sprite_pixel(instance, lx, ly) {
                    if (src_px >> 24) & 0xFF != 0 {
                        let dst_idx = row_base + sxu;
                        if dst_idx < pixels {
                            // sem alpha blend por performance
                            buffer[dst_idx] = src_px;
                        }
                    }
                }
            }
        }
    }

    pub fn draw_sprite_batch<const N: usize>(&mut self, batch: &SpriteBatch<'_, N>) {
        batch.for_each_sorted(|instance| self.draw_sprite_instance(instance));
    }

    // --- Efeitos e Copper ---

    pub fn copper_mut(&mut self) -> &mut CopperList<COPPER_CAPACITY> {
        &mut self.copper
    }

    pub fn run_copper(&mut self) {
        let idx = self.back_index();
        let p = self.pixels;
        self.copper
            .execute(&mut self.buffers[idx][..p], self.width, self.height);
    }

    pub fn motion_blur(&mut self, blend: u8) {
        if blend == 255 {
            return;
        }

        let keep_new = blend as u32;
        let keep_old = 255 - keep_new;
        let pixels_limit = self.pixels.min(MAX_PIXELS);

        let (slice_0, slice_1) = self.buffers.split_at_mut(1);
        let (front_buf, back_buf) = if self.draw == BufferIndex::Back {
            (slice_0[0].as_slice(), &mut slice_1[0])
        } else {
            (slice_1[0].as_slice(), &mut slice_0[0])
        };

        for i in 0..pixels_limit {
            let src = front_buf[i];
            let dst = &mut back_buf[i];

            let r = (((*dst >> 16) & 0xFF) * keep_new + ((src >> 16) & 0xFF) * keep_old) / 255;
            let g = (((*dst >> 8) & 0xFF) * keep_new + ((src >> 8) & 0xFF) * keep_old) / 255;
            let b = ((*dst & 0xFF) * keep_new + (src & 0xFF) * keep_old) / 255;

            *dst = 0xFF00_0000 | (r << 16) | (g << 8) | b;
        }
    }

    // --- Getters ---

    #[inline]
    pub fn width(&self) -> usize {
        self.width
    }

    #[inline]
    pub fn height(&self) -> usize {
        self.height
    }

    #[inline]
    pub fn frame(&self) -> u64 {
        self.frame_count
    }
}