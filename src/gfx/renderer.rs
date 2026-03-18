// src/gfx/renderer.rs
//
// Renderer — orquestrador de frames para o demo engine bare-metal (aarch64 / RPi)
// Coordena framebuffer, blitter e copper para produzir cada frame.

use crate::drivers::framebuffer::{Framebuffer, PixelFormat};
use crate::gfx::blitter::Blitter;
use crate::gfx::copper::CopperList;

// ---------------------------------------------------------------------------
// Constantes de display
// ---------------------------------------------------------------------------

pub const SCREEN_WIDTH:  usize = 320;
pub const SCREEN_HEIGHT: usize = 240;
pub const BYTES_PER_PIXEL: usize = 4; // ARGB8888
pub const FRAMEBUFFER_SIZE: usize = SCREEN_WIDTH * SCREEN_HEIGHT * BYTES_PER_PIXEL;

// ---------------------------------------------------------------------------
// Double-buffer
// ---------------------------------------------------------------------------

/// Qual dos dois buffers está sendo exibido agora.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ActiveBuffer {
    Front = 0,
    Back  = 1,
}

impl ActiveBuffer {
    #[inline]
    pub fn flip(self) -> Self {
        match self {
            ActiveBuffer::Front => ActiveBuffer::Back,
            ActiveBuffer::Back  => ActiveBuffer::Front,
        }
    }
}

// ---------------------------------------------------------------------------
// Renderer
// ---------------------------------------------------------------------------

pub struct Renderer {
    fb:          Framebuffer,
    blitter:     Blitter,
    copper:      CopperList,

    /// Dois buffers em memória; alternamos a cada frame.
    buffers:     [[u32; SCREEN_WIDTH * SCREEN_HEIGHT]; 2],
    active:      ActiveBuffer,

    /// Contador de frames desde o boot.
    pub frame:   u64,
}

impl Renderer {
    /// Inicializa o renderer e configura o framebuffer via mailbox.
    pub fn new(fb: Framebuffer) -> Self {
        Self {
            blitter:  Blitter::new(SCREEN_WIDTH, SCREEN_HEIGHT),
            copper:   CopperList::new(),
            buffers:  [[0u32; SCREEN_WIDTH * SCREEN_HEIGHT]; 2],
            active:   ActiveBuffer::Back,
            frame:    0,
            fb,
        }
    }

    // -----------------------------------------------------------------------
    // API de frame
    // -----------------------------------------------------------------------

    /// Retorna um slice mutável para o back-buffer atual.
    #[inline]
    pub fn back_buffer(&mut self) -> &mut [u32] {
        &mut self.buffers[self.active as usize]
    }

    /// Apaga o back-buffer com a cor fornecida (ARGB).
    #[inline]
    pub fn clear(&mut self, color: u32) {
        let buf = self.back_buffer();
        for px in buf.iter_mut() {
            *px = color;
        }
    }

    /// Apaga o back-buffer com preto.
    #[inline]
    pub fn clear_black(&mut self) {
        self.clear(0xFF000000);
    }

    /// Executa a copper list sobre o back-buffer (efeitos de raster line-by-line).
    pub fn run_copper(&mut self) {
        let buf = &mut self.buffers[self.active as usize];
        self.copper.execute(buf, SCREEN_WIDTH, SCREEN_HEIGHT);
    }

    /// Apresenta o back-buffer: copia para o framebuffer físico e faz flip.
    pub fn present(&mut self) {
        // Copia o back-buffer para o framebuffer de hardware.
        let src = &self.buffers[self.active as usize];
        self.fb.blit_argb(src);

        // Flip: o buffer que acabou de ser enviado torna-se o front.
        self.active = self.active.flip();
        self.frame  = self.frame.wrapping_add(1);
    }

    // -----------------------------------------------------------------------
    // Primitivas de desenho (delegam ao Blitter)
    // -----------------------------------------------------------------------

    /// Desenha um pixel no back-buffer.
    #[inline]
    pub fn put_pixel(&mut self, x: usize, y: usize, color: u32) {
        if x < SCREEN_WIDTH && y < SCREEN_HEIGHT {
            self.buffers[self.active as usize][y * SCREEN_WIDTH + x] = color;
        }
    }

    /// Lê um pixel do back-buffer.
    #[inline]
    pub fn get_pixel(&self, x: usize, y: usize) -> u32 {
        if x < SCREEN_WIDTH && y < SCREEN_HEIGHT {
            self.buffers[self.active as usize][y * SCREEN_WIDTH + x]
        } else {
            0
        }
    }

    /// Linha horizontal rápida via blitter.
    pub fn hline(&mut self, y: usize, x0: usize, x1: usize, color: u32) {
        let buf = &mut self.buffers[self.active as usize];
        self.blitter.hline(buf, y, x0, x1, color);
    }

    /// Linha vertical rápida via blitter.
    pub fn vline(&mut self, x: usize, y0: usize, y1: usize, color: u32) {
        let buf = &mut self.buffers[self.active as usize];
        self.blitter.vline(buf, x, y0, y1, color);
    }

    /// Retângulo preenchido.
    pub fn fill_rect(&mut self, x: usize, y: usize, w: usize, h: usize, color: u32) {
        let buf = &mut self.buffers[self.active as usize];
        self.blitter.fill_rect(buf, x, y, w, h, color);
    }

    /// Copia um sprite (ARGB, com alpha) para o back-buffer.
    pub fn blit_sprite(
        &mut self,
        sprite: &[u32],
        sw: usize, sh: usize,
        dx: usize, dy: usize,
    ) {
        let buf = &mut self.buffers[self.active as usize];
        self.blitter.blit_alpha(buf, sprite, sw, sh, dx, dy, SCREEN_WIDTH, SCREEN_HEIGHT);
    }

    // -----------------------------------------------------------------------
    // Efeitos de tela cheia
    // -----------------------------------------------------------------------

    /// Fade out: mistura o back-buffer com preto em `amount` (0–255).
    pub fn fade_to_black(&mut self, amount: u8) {
        let buf = &mut self.buffers[self.active as usize];
        let a = amount as u32;
        for px in buf.iter_mut() {
            let r = ((*px >> 16) & 0xFF) * (255 - a) / 255;
            let g = ((*px >>  8) & 0xFF) * (255 - a) / 255;
            let b = ( *px        & 0xFF) * (255 - a) / 255;
            *px = 0xFF000000 | (r << 16) | (g << 8) | b;
        }
    }

    /// Fade in: mistura o back-buffer com preto em `amount` (0–255).
    /// amount=0 → preto total, amount=255 → imagem original.
    #[inline]
    pub fn fade_from_black(&mut self, amount: u8) {
        self.fade_to_black(255 - amount);
    }

    /// Motion blur leve: mistura back com front (ghosting/trail).
    pub fn motion_blur(&mut self, blend: u8) {
        let front_idx = self.active.flip() as usize;
        let back_idx  = self.active       as usize;

        // SAFETY: acessamos índices distintos do array.
        let (front, back) = if front_idx < back_idx {
            let (a, b) = self.buffers.split_at_mut(back_idx);
            (&a[front_idx], &mut b[0])
        } else {
            let (a, b) = self.buffers.split_at_mut(front_idx);
            (&b[0], &mut a[back_idx])
        };

        let a = blend as u32;
        for (dst, &src) in back.iter_mut().zip(front.iter()) {
            let mix = |ch_dst: u32, ch_src: u32| -> u32 {
                (ch_dst * a + ch_src * (255 - a)) / 255
            };
            let r = mix((*dst >> 16) & 0xFF, (src >> 16) & 0xFF);
            let g = mix((*dst >>  8) & 0xFF, (src >>  8) & 0xFF);
            let b = mix( *dst        & 0xFF,  src        & 0xFF);
            *dst = 0xFF000000 | (r << 16) | (g << 8) | b;
        }
    }

    // -----------------------------------------------------------------------
    // Helpers de copper
    // -----------------------------------------------------------------------

    /// Expõe a copper list para que demos possam programar efeitos de raster.
    #[inline]
    pub fn copper_mut(&mut self) -> &mut CopperList {
        &mut self.copper
    }

    // -----------------------------------------------------------------------
    // Informação
    // -----------------------------------------------------------------------

    #[inline]
    pub fn width(&self)  -> usize { SCREEN_WIDTH  }
    #[inline]
    pub fn height(&self) -> usize { SCREEN_HEIGHT }
    #[inline]
    pub fn frame(&self)  -> u64   { self.frame    }
}