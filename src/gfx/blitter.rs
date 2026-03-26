// src/gfx/blitter.rs
//
// Blitter básico para o demo engine bare-metal.
// Opera sobre buffers lineares em ARGB8888.
//
// Layout esperado:
// - framebuffer/buffer linear
// - largura = screen_width
// - índice = y * width + x

#![allow(dead_code)]

pub struct Blitter {
    width: usize,
    height: usize,
}

impl Blitter {
    #[inline]
    pub const fn new(width: usize, height: usize) -> Self {
        Self { width, height }
    }

    #[inline]
    pub fn width(&self) -> usize {
        self.width
    }

    #[inline]
    pub fn height(&self) -> usize {
        self.height
    }

    // -----------------------------------------------------------------------
    // Pixel
    // -----------------------------------------------------------------------

    #[inline]
    pub fn put_pixel(&self, dst: &mut [u32], x: usize, y: usize, color: u32) {
        if x < self.width && y < self.height {
            let idx = y * self.width + x;
            if idx < dst.len() {
                dst[idx] = color;
            }
        }
    }

    #[inline]
    pub fn get_pixel(&self, src: &[u32], x: usize, y: usize) -> u32 {
        if x < self.width && y < self.height {
            let idx = y * self.width + x;
            if idx < src.len() {
                return src[idx];
            }
        }
        0
    }

    // -----------------------------------------------------------------------
    // Linhas
    // -----------------------------------------------------------------------

    pub fn hline(&self, dst: &mut [u32], y: usize, x0: usize, x1: usize, color: u32) {
        if y >= self.height || self.width == 0 {
            return;
        }

        let xa = x0.min(x1);
        let mut xb = x0.max(x1);

        if xa >= self.width {
            return;
        }

        if xb >= self.width {
            xb = self.width - 1;
        }

        let row = y * self.width;
        if row >= dst.len() {
            return;
        }

        let max_x_by_dst = dst.len().saturating_sub(row + 1);
        let xb = xb.min(max_x_by_dst);

        for x in xa..=xb {
            dst[row + x] = color;
        }
    }

    pub fn vline(&self, dst: &mut [u32], x: usize, y0: usize, y1: usize, color: u32) {
        if x >= self.width || self.height == 0 {
            return;
        }

        let ya = y0.min(y1);
        let mut yb = y0.max(y1);

        if ya >= self.height {
            return;
        }

        if yb >= self.height {
            yb = self.height - 1;
        }

        for y in ya..=yb {
            let idx = y * self.width + x;
            if idx < dst.len() {
                dst[idx] = color;
            } else {
                break;
            }
        }
    }

    // -----------------------------------------------------------------------
    // Retângulos
    // -----------------------------------------------------------------------

    pub fn fill_rect(
        &self,
        dst: &mut [u32],
        x: usize,
        y: usize,
        w: usize,
        h: usize,
        color: u32,
    ) {
        if w == 0 || h == 0 {
            return;
        }

        if x >= self.width || y >= self.height {
            return;
        }

        let x_end = x.saturating_add(w).min(self.width);
        let y_end = y.saturating_add(h).min(self.height);

        for yy in y..y_end {
            let row = yy * self.width;
            if row >= dst.len() {
                break;
            }

            let max_x_exclusive = dst.len().saturating_sub(row).min(self.width);
            let safe_x_end = x_end.min(max_x_exclusive);

            for xx in x..safe_x_end {
                dst[row + xx] = color;
            }
        }
    }

    // -----------------------------------------------------------------------
    // Blit sem alpha
    // -----------------------------------------------------------------------

    /// Copia um sprite linear `src` de dimensão `sw x sh` para `dst`,
    /// com clipping contra a tela.
    pub fn blit(
        &self,
        dst: &mut [u32],
        src: &[u32],
        sw: usize,
        sh: usize,
        dx: usize,
        dy: usize,
        screen_w: usize,
        screen_h: usize,
    ) {
        if sw == 0 || sh == 0 || screen_w == 0 || screen_h == 0 {
            return;
        }

        if dx >= screen_w || dy >= screen_h {
            return;
        }

        let copy_w = sw.min(screen_w.saturating_sub(dx));
        let copy_h = sh.min(screen_h.saturating_sub(dy));

        for sy in 0..copy_h {
            let y = dy + sy;
            if y >= screen_h {
                break;
            }

            let dst_row = y * screen_w;
            if dst_row >= dst.len() {
                break;
            }

            let src_row = sy * sw;
            if src_row >= src.len() {
                break;
            }

            let max_sx_by_dst = dst.len().saturating_sub(dst_row + dx);
            let max_sx_by_src = src.len().saturating_sub(src_row);
            let safe_copy_w = copy_w.min(max_sx_by_dst).min(max_sx_by_src);

            for sx in 0..safe_copy_w {
                let dst_idx = dst_row + dx + sx;
                let src_idx = src_row + sx;
                dst[dst_idx] = src[src_idx];
            }
        }
    }

    // -----------------------------------------------------------------------
    // Blit com alpha
    // -----------------------------------------------------------------------

    /// Copia sprite ARGB8888 com alpha sobre o destino.
    ///
    /// Regras:
    /// - alpha=0   -> não altera destino
    /// - alpha=255 -> substitui destino
    /// - senão     -> blend normal
    pub fn blit_alpha(
        &self,
        dst: &mut [u32],
        src: &[u32],
        sw: usize,
        sh: usize,
        dx: usize,
        dy: usize,
        screen_w: usize,
        screen_h: usize,
    ) {
        if sw == 0 || sh == 0 || screen_w == 0 || screen_h == 0 {
            return;
        }

        if dx >= screen_w || dy >= screen_h {
            return;
        }

        let copy_w = sw.min(screen_w.saturating_sub(dx));
        let copy_h = sh.min(screen_h.saturating_sub(dy));

        for sy in 0..copy_h {
            let y = dy + sy;
            if y >= screen_h {
                break;
            }

            let dst_row = y * screen_w;
            if dst_row >= dst.len() {
                break;
            }

            let src_row = sy * sw;
            if src_row >= src.len() {
                break;
            }

            let max_sx_by_dst = dst.len().saturating_sub(dst_row + dx);
            let max_sx_by_src = src.len().saturating_sub(src_row);
            let safe_copy_w = copy_w.min(max_sx_by_dst).min(max_sx_by_src);

            for sx in 0..safe_copy_w {
                let src_px = src[src_row + sx];
                let dst_idx = dst_row + dx + sx;

                let a = ((src_px >> 24) & 0xFF) as u8;

                if a == 0 {
                    continue;
                } else if a == 255 {
                    dst[dst_idx] = src_px;
                } else {
                    dst[dst_idx] = blend_argb(dst[dst_idx], src_px);
                }
            }
        }
    }

    // -----------------------------------------------------------------------
    // Bresenham
    // -----------------------------------------------------------------------

    pub fn line(
        &self,
        dst: &mut [u32],
        mut x0: isize,
        mut y0: isize,
        x1: isize,
        y1: isize,
        color: u32,
    ) {
        let dx = abs_i(x1 - x0);
        let sx = if x0 < x1 { 1 } else { -1 };
        let dy = -abs_i(y1 - y0);
        let sy = if y0 < y1 { 1 } else { -1 };
        let mut err = dx + dy;

        loop {
            if x0 >= 0 && y0 >= 0 {
                let xu = x0 as usize;
                let yu = y0 as usize;
                if xu < self.width && yu < self.height {
                    let idx = yu * self.width + xu;
                    if idx < dst.len() {
                        dst[idx] = color;
                    }
                }
            }

            if x0 == x1 && y0 == y1 {
                break;
            }

            let e2 = err * 2;
            if e2 >= dy {
                err += dy;
                x0 += sx;
            }
            if e2 <= dx {
                err += dx;
                y0 += sy;
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Helpers de cor
// ---------------------------------------------------------------------------

#[inline]
pub fn argb(a: u8, r: u8, g: u8, b: u8) -> u32 {
    ((a as u32) << 24)
        | ((r as u32) << 16)
        | ((g as u32) << 8)
        | (b as u32)
}

#[inline]
pub fn blend_argb(dst: u32, src: u32) -> u32 {
    let sa = (src >> 24) & 0xFF;

    if sa == 0 {
        return dst;
    }
    if sa == 255 {
        return src;
    }

    let inv = 255 - sa;

    let sr = (src >> 16) & 0xFF;
    let sg = (src >> 8) & 0xFF;
    let sb = src & 0xFF;

    let dr = (dst >> 16) & 0xFF;
    let dg = (dst >> 8) & 0xFF;
    let db = dst & 0xFF;

    let r = (sr * sa + dr * inv) / 255;
    let g = (sg * sa + dg * inv) / 255;
    let b = (sb * sa + db * inv) / 255;

    0xFF00_0000 | (r << 16) | (g << 8) | b
}

#[inline]
fn abs_i(v: isize) -> isize {
    if v < 0 { -v } else { v }
}
