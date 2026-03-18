use crate::drivers::framebuffer::Framebuffer;

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
        // t = 0..255
        let r = self.r as u32 + (((other.r as i32 - self.r as i32) * t as i32) / 255) as u32;
        let g = self.g as u32 + (((other.g as i32 - self.g as i32) * t as i32) / 255) as u32;
        let b = self.b as u32 + (((other.b as i32 - self.b as i32) * t as i32) / 255) as u32;

        Self {
            r: r as u8,
            g: g as u8,
            b: b as u8,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum CopperOp {
    /// Preenche a tela toda com uma cor
    Clear {
        color: Rgb,
    },

    /// Faixa vertical com cor sólida
    SolidBar {
        y: u32,
        height: u32,
        color: Rgb,
    },

    /// Gradiente vertical entre top e bottom
    GradientBar {
        y: u32,
        height: u32,
        top: Rgb,
        bottom: Rgb,
    },

    /// Barra "macia" estilo raster bar:
    /// sobe até center e desce depois
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
}

pub struct Copper {
    frame: u32,
}

impl Copper {
    pub const fn new() -> Self {
        Self { frame: 0 }
    }

    pub fn frame(&self) -> u32 {
        self.frame
    }

    pub fn next_frame(&mut self) {
        self.frame = self.frame.wrapping_add(1);
    }

    pub fn execute<const N: usize>(&mut self, fb: &mut Framebuffer, list: &CopperList<N>) {
        for op in list.iter() {
            match op {
                CopperOp::Clear { color } => {
                    fb.clear(fb.color_rgb(color.r, color.g, color.b));
                }

                CopperOp::SolidBar { y, height, color } => {
                    let c = fb.color_rgb(color.r, color.g, color.b);
                    self.fill_scanlines(fb, y, height, c);
                }

                CopperOp::GradientBar {
                    y,
                    height,
                    top,
                    bottom,
                } => {
                    self.gradient_bar(fb, y, height, top, bottom);
                }

                CopperOp::RasterBar {
                    y,
                    height,
                    color,
                    glow,
                } => {
                    self.raster_bar(fb, y, height, color, glow);
                }
            }
        }

        self.next_frame();
    }

    fn fill_scanlines(&self, fb: &mut Framebuffer, y: u32, height: u32, color: u32) {
        let y0 = y.min(fb.height);
        let y1 = y.saturating_add(height).min(fb.height);

        for yy in y0..y1 {
            fb.fill_rect(0, yy, fb.width, 1, color);
        }
    }

    fn gradient_bar(
        &self,
        fb: &mut Framebuffer,
        y: u32,
        height: u32,
        top: Rgb,
        bottom: Rgb,
    ) {
        if height == 0 {
            return;
        }

        let y0 = y.min(fb.height);
        let y1 = y.saturating_add(height).min(fb.height);
        let span = y1.saturating_sub(y0);

        if span == 0 {
            return;
        }

        let denom = span.saturating_sub(1).max(1);

        for i in 0..span {
            let t = ((i * 255) / denom) as u8;
            let c = top.lerp(bottom, t);
            let color = fb.color_rgb(c.r, c.g, c.b);
            fb.fill_rect(0, y0 + i, fb.width, 1, color);
        }
    }

    fn raster_bar(
        &self,
        fb: &mut Framebuffer,
        y: i32,
        height: u32,
        base: Rgb,
        glow: u8,
    ) {
        if height == 0 {
            return;
        }

        let half = (height as i32) / 2;
        let screen_h = fb.height as i32;

        for i in 0..height as i32 {
            let yy = y + i;
            if yy < 0 || yy >= screen_h {
                continue;
            }

            let dist = (i - half).unsigned_abs() as u32;
            let maxd = half.max(1) as u32;

            // intensidade 255 no centro, caindo para as bordas
            let falloff = 255u32.saturating_sub((dist * 255) / maxd) as u8;

            let boost = ((falloff as u32 * glow as u32) / 255) as u8;

            let r = base.r.saturating_add(boost);
            let g = base.g.saturating_add(boost);
            let b = base.b.saturating_add(boost);

            let color = fb.color_rgb(r, g, b);
            fb.fill_rect(0, yy as u32, fb.width, 1, color);
        }
    }
}