// src/gfx/copper.rs
// Simula um "copper" estilo Amiga para efeitos raster em buffer de software.

#[derive(Clone, Copy, Debug)]
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Rgb {
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    #[inline(always)]
    pub fn lerp(self, other: Self, t: u8) -> Self {
        let r = self.r as u32 + (((other.r as i32 - self.r as i32) * t as i32) / 255) as u32;
        let g = self.g as u32 + (((other.g as i32 - self.g as i32) * t as i32) / 255) as u32;
        let b = self.b as u32 + (((other.b as i32 - self.b as i32) * t as i32) / 255) as u32;

        Self {
            r: r as u8,
            g: g as u8,
            b: b as u8,
        }
    }

    #[inline(always)]
    pub fn to_argb(self) -> u32 {
        0xFF00_0000 | ((self.r as u32) << 16) | ((self.g as u32) << 8) | (self.b as u32)
    }
}

#[derive(Clone, Copy, Debug)]
pub enum CopperOp {
    /// Preenche a tela toda com uma cor.
    Clear {
        color: Rgb,
    },

    /// Faixa horizontal sólida.
    SolidBar {
        y: u32,
        height: u32,
        color: Rgb,
    },

    /// Gradiente vertical entre `top` e `bottom`.
    GradientBar {
        y: u32,
        height: u32,
        top: Rgb,
        bottom: Rgb,
    },

    /// Barra "macia" estilo raster bar.
    RasterBar {
        y: i32,
        height: u32,
        color: Rgb,
        glow: u8,
    },
}

pub struct CopperList<const N: usize> {
    ops: [Option<CopperOp>; N],
    len: usize,
}

impl<const N: usize> CopperList<N> {
    pub const fn new() -> Self {
        Self {
            ops: [None; N],
            len: 0,
        }
    }

    pub fn clear(&mut self) {
        self.len = 0;
    }

    pub fn push(&mut self, op: CopperOp) -> bool {
        if self.len >= N {
            return false;
        }

        self.ops[self.len] = Some(op);
        self.len += 1;
        true
    }

    pub fn iter(&self) -> impl Iterator<Item = CopperOp> + '_ {
        self.ops[..self.len].iter().filter_map(|x| *x)
    }

    pub fn execute(&self, buf: &mut [u32], width: usize, height: usize) {
        for op in self.iter() {
            match op {
                CopperOp::Clear { color } => {
                    clear(buf, color.to_argb());
                }

                CopperOp::SolidBar { y, height: h, color } => {
                    fill_scanlines(buf, width, height, y as usize, h as usize, color.to_argb());
                }

                CopperOp::GradientBar {
                    y,
                    height: h,
                    top,
                    bottom,
                } => {
                    gradient_bar(buf, width, height, y as usize, h as usize, top, bottom);
                }

                CopperOp::RasterBar {
                    y,
                    height: h,
                    color,
                    glow,
                } => {
                    raster_bar(buf, width, height, y, h as usize, color, glow);
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Helpers internos
// ---------------------------------------------------------------------------

#[inline]
fn clear(buf: &mut [u32], color: u32) {
    buf.fill(color);
}

fn fill_scanlines(
    buf: &mut [u32],
    width: usize,
    height: usize,
    y: usize,
    bar_height: usize,
    color: u32,
) {
    if width == 0 || height == 0 || bar_height == 0 {
        return;
    }

    let y0 = y.min(height);
    let y1 = y.saturating_add(bar_height).min(height);

    for yy in y0..y1 {
        let row = yy * width;
        let row_slice = &mut buf[row..row + width];
        row_slice.fill(color);
    }
}

fn gradient_bar(
    buf: &mut [u32],
    width: usize,
    height: usize,
    y: usize,
    bar_height: usize,
    top: Rgb,
    bottom: Rgb,
) {
    if width == 0 || height == 0 || bar_height == 0 {
        return;
    }

    let y0 = y.min(height);
    let y1 = y.saturating_add(bar_height).min(height);
    let span = y1.saturating_sub(y0);

    if span == 0 {
        return;
    }

    let denom = span.saturating_sub(1).max(1);

    for i in 0..span {
        let t = ((i * 255) / denom) as u8;
        let c = top.lerp(bottom, t).to_argb();
        let row = (y0 + i) * width;
        let row_slice = &mut buf[row..row + width];
        row_slice.fill(c);
    }
}

fn raster_bar(
    buf: &mut [u32],
    width: usize,
    height: usize,
    y: i32,
    bar_height: usize,
    base: Rgb,
    glow: u8,
) {
    if width == 0 || height == 0 || bar_height == 0 {
        return;
    }

    let half = (bar_height as i32) / 2;
    let screen_h = height as i32;

    for i in 0..bar_height as i32 {
        let yy = y + i;
        if yy < 0 || yy >= screen_h {
            continue;
        }

        let dist = (i - half).unsigned_abs() as u32;
        let maxd = half.max(1) as u32;

        let falloff = 255u32.saturating_sub((dist * 255) / maxd) as u8;
        let boost = ((falloff as u32 * glow as u32) / 255) as u8;

        let r = base.r.saturating_add(boost);
        let g = base.g.saturating_add(boost);
        let b = base.b.saturating_add(boost);

        let color = 0xFF00_0000 | ((r as u32) << 16) | ((g as u32) << 8) | (b as u32);

        let row = yy as usize * width;
        let row_slice = &mut buf[row..row + width];
        row_slice.fill(color);
    }
}