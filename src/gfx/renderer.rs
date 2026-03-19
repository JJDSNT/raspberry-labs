// src/gfx/renderer.rs
//
// Renderer — orquestrador de frames para o demo engine bare-metal (aarch64 / RPi)
// Coordena framebuffer, blitter e copper para produzir cada frame.

use core::sync::atomic::{AtomicBool, Ordering};

use crate::drivers::framebuffer::Framebuffer;
use crate::gfx::blitter::Blitter;
use crate::gfx::copper::CopperList;

// ---------------------------------------------------------------------------
// Limites máximos do renderer
// ---------------------------------------------------------------------------
//
// Como estamos em bare metal e não queremos depender de allocator,
// reservamos buffers estáticos grandes o bastante e usamos apenas a
// região necessária para a resolução atual.

pub const MAX_WIDTH: usize = 1024;
pub const MAX_HEIGHT: usize = 768;
pub const BYTES_PER_PIXEL: usize = 4; // ARGB8888
pub const MAX_PIXELS: usize = MAX_WIDTH * MAX_HEIGHT;
pub const FRAMEBUFFER_SIZE: usize = MAX_PIXELS * BYTES_PER_PIXEL;

const COPPER_CAPACITY: usize = 256;

// ---------------------------------------------------------------------------
// Buffers estáticos (.bss)
// ---------------------------------------------------------------------------

static mut RENDER_BUFFERS: [[u32; MAX_PIXELS]; 2] = [[0; MAX_PIXELS]; 2];
static RENDERER_TAKEN: AtomicBool = AtomicBool::new(false);

// ---------------------------------------------------------------------------
// Double-buffer
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
// Renderer
// ---------------------------------------------------------------------------

pub struct Renderer {
    fb: Framebuffer,
    blitter: Blitter,
    copper: CopperList<COPPER_CAPACITY>,

    /// Buffers máximos estáticos; usamos apenas `pixels` elementos.
    buffers: &'static mut [[u32; MAX_PIXELS]; 2],

    width: usize,
    height: usize,
    pixels: usize,

    /// Buffer onde o próximo frame está sendo desenhado.
    draw: BufferIndex,

    /// Contador de frames desde o boot.
    frame_count: u64,
}

impl Renderer {
    /// Inicializa o renderer.
    ///
    /// Assume instância única no sistema.
    /// Uma segunda inicialização gera panic.
    pub fn new(fb: Framebuffer) -> Self {
        let already_taken = RENDERER_TAKEN.swap(true, Ordering::AcqRel);
        assert!(!already_taken, "Renderer::new() called more than once");

        let width = fb.width as usize;
        let height = fb.height as usize;
        let pixels = width.saturating_mul(height);

        assert!(width > 0, "Renderer width must be > 0");
        assert!(height > 0, "Renderer height must be > 0");
        assert!(
            width <= MAX_WIDTH && height <= MAX_HEIGHT,
            "Renderer resolution exceeds static buffer capacity"
        );

        Self {
            fb,
            blitter: Blitter::new(width, height),
            copper: CopperList::new(),
            // SAFETY:
            // - `RENDERER_TAKEN` garante instância única.
            // - Logo só existe um `&'static mut` ativo para os buffers.
            buffers: unsafe { &mut RENDER_BUFFERS },
            width,
            height,
            pixels,
            draw: BufferIndex::Back,
            frame_count: 0,
        }
    }

    // -----------------------------------------------------------------------
    // Helpers de índices
    // -----------------------------------------------------------------------

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

    #[inline]
    fn front_buffer_full(&self) -> &[u32; MAX_PIXELS] {
        &self.buffers[self.front_index()]
    }

    // -----------------------------------------------------------------------
    // API de frame
    // -----------------------------------------------------------------------

    /// Retorna o back-buffer atual, limitado à resolução ativa.
    #[inline]
    pub fn back_buffer(&mut self) -> &mut [u32] {
        let pixels = self.pixels;
        &mut self.back_buffer_full()[..pixels]
    }

    /// Retorna o front-buffer atual (somente leitura), limitado à resolução ativa.
    #[inline]
    pub fn front_buffer(&self) -> &[u32] {
        &self.front_buffer_full()[..self.pixels]
    }

    /// Limpa o back-buffer com uma cor ARGB.
    #[inline]
    pub fn clear(&mut self, color: u32) {
        self.back_buffer().fill(color);
    }

    /// Limpa o back-buffer com preto opaco.
    #[inline]
    pub fn clear_black(&mut self) {
        self.clear(0xFF00_0000);
    }

    /// Executa a copper list sobre o back-buffer.
    pub fn run_copper(&mut self) {
        let idx = self.back_index();
        self.copper
            .execute(&mut self.buffers[idx][..self.pixels], self.width, self.height);
    }

    /// Apresenta o back-buffer no framebuffer físico e faz flip lógico.
    pub fn present(&mut self) {
        let idx = self.back_index();
        self.fb.blit_argb(&self.buffers[idx][..self.pixels]);

        self.draw = self.draw.flip();
        self.frame_count = self.frame_count.wrapping_add(1);
    }

    // -----------------------------------------------------------------------
    // Primitivas de desenho
    // -----------------------------------------------------------------------

    #[inline]
    pub fn put_pixel(&mut self, x: usize, y: usize, color: u32) {
        if x < self.width && y < self.height {
            let idx = y * self.width + x;
            self.buffers[self.back_index()][idx] = color;
        }
    }

    #[inline]
    pub fn get_pixel(&self, x: usize, y: usize) -> u32 {
        if x < self.width && y < self.height {
            self.buffers[self.back_index()][y * self.width + x]
        } else {
            0
        }
    }

    pub fn hline(&mut self, y: usize, x0: usize, x1: usize, color: u32) {
        let idx = self.back_index();
        self.blitter
            .hline(&mut self.buffers[idx][..self.pixels], y, x0, x1, color);
    }

    pub fn vline(&mut self, x: usize, y0: usize, y1: usize, color: u32) {
        let idx = self.back_index();
        self.blitter
            .vline(&mut self.buffers[idx][..self.pixels], x, y0, y1, color);
    }

    pub fn fill_rect(&mut self, x: usize, y: usize, w: usize, h: usize, color: u32) {
        let idx = self.back_index();
        self.blitter
            .fill_rect(&mut self.buffers[idx][..self.pixels], x, y, w, h, color);
    }

    pub fn blit_sprite(
        &mut self,
        sprite: &[u32],
        sw: usize,
        sh: usize,
        dx: usize,
        dy: usize,
    ) {
        let idx = self.back_index();
        self.blitter.blit_alpha(
            &mut self.buffers[idx][..self.pixels],
            sprite,
            sw,
            sh,
            dx,
            dy,
            self.width,
            self.height,
        );
    }

    // -----------------------------------------------------------------------
    // Efeitos fullscreen
    // -----------------------------------------------------------------------

    /// Escurece o back-buffer em direção ao preto.
    ///
    /// amount=0   -> imagem original
    /// amount=255 -> preto total
    pub fn fade_to_black(&mut self, amount: u8) {
        let factor = 255u32 - amount as u32;
        let idx = self.back_index();

        for px in self.buffers[idx][..self.pixels].iter_mut() {
            let r = ((*px >> 16) & 0xFF) * factor / 255;
            let g = ((*px >> 8) & 0xFF) * factor / 255;
            let b = (*px & 0xFF) * factor / 255;

            *px = 0xFF00_0000 | (r << 16) | (g << 8) | b;
        }
    }

    /// Clareia a partir do preto.
    ///
    /// amount=0   -> preto total
    /// amount=255 -> imagem original
    #[inline]
    pub fn fade_from_black(&mut self, amount: u8) {
        self.fade_to_black(255 - amount);
    }

    /// Mistura o frame atual com o anterior para produzir ghosting/trail.
    ///
    /// `blend` é o peso do frame novo:
    /// - 255 => quase só frame atual
    /// - 0   => quase só frame anterior
    pub fn motion_blur(&mut self, blend: u8) {
        let keep_new = blend as u32;
        let keep_old = 255u32 - keep_new;

        let front_idx = self.front_index();
        let back_idx = self.back_index();
        let pixels = self.pixels;

        let (front, back): (&[u32; MAX_PIXELS], &mut [u32; MAX_PIXELS]) = if front_idx < back_idx {
            let (lo, hi) = self.buffers.split_at_mut(back_idx);
            (&lo[front_idx], &mut hi[0])
        } else {
            let (lo, hi) = self.buffers.split_at_mut(front_idx);
            (&hi[0], &mut lo[back_idx])
        };

        for (dst, src) in back[..pixels].iter_mut().zip(front[..pixels].iter()) {
            let r = (((*dst >> 16) & 0xFF) * keep_new + ((src >> 16) & 0xFF) * keep_old) / 255;
            let g = (((*dst >> 8) & 0xFF) * keep_new + ((src >> 8) & 0xFF) * keep_old) / 255;
            let b = ((*dst & 0xFF) * keep_new + (src & 0xFF) * keep_old) / 255;

            *dst = 0xFF00_0000 | (r << 16) | (g << 8) | b;
        }
    }

    // -----------------------------------------------------------------------
    // Copper
    // -----------------------------------------------------------------------

    #[inline]
    pub fn copper_mut(&mut self) -> &mut CopperList<COPPER_CAPACITY> {
        &mut self.copper
    }

    // -----------------------------------------------------------------------
    // Informação
    // -----------------------------------------------------------------------

    #[inline]
    pub fn width(&self) -> usize {
        self.width
    }

    #[inline]
    pub fn height(&self) -> usize {
        self.height
    }

    #[inline]
    pub fn pixels(&self) -> usize {
        self.pixels
    }

    #[inline]
    pub fn frame(&self) -> u64 {
        self.frame_count
    }
}