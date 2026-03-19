// src/demos/rasterbars.rs
//
// Demo de raster bars estilo Amiga, usando CopperList sobre o back-buffer.

use crate::gfx::copper::{CopperOp, Rgb};
use crate::gfx::renderer::Renderer;

const TAU: usize = 256;

// LUT seno 0..255 -> aproximadamente -127..127
const SIN_LUT: [i8; TAU] = [
      0,   3,   6,   9,  12,  16,  19,  22,  25,  28,  31,  34,  37,  40,  43,  46,
     49,  52,  55,  58,  60,  63,  66,  68,  71,  74,  76,  78,  81,  83,  85,  87,
     89,  91,  93,  95,  96,  98, 100, 101, 103, 104, 105, 107, 108, 109, 110, 111,
    112, 112, 113, 114, 114, 115, 115, 116, 116, 116, 116, 116, 116, 116, 116, 115,
    115, 114, 114, 113, 112, 112, 111, 110, 109, 108, 107, 105, 104, 103, 101, 100,
     98,  96,  95,  93,  91,  89,  87,  85,  83,  81,  78,  76,  74,  71,  68,  66,
     63,  60,  58,  55,  52,  49,  46,  43,  40,  37,  34,  31,  28,  25,  22,  19,
     16,  12,   9,   6,   3,   0,  -3,  -6,  -9, -12, -16, -19, -22, -25, -28, -31,
    -34, -37, -40, -43, -46, -49, -52, -55, -58, -60, -63, -66, -68, -71, -74, -76,
    -78, -81, -83, -85, -87, -89, -91, -93, -95, -96, -98,-100,-101,-103,-104,-105,
   -107,-108,-109,-110,-111,-112,-112,-113,-114,-114,-115,-115,-116,-116,-116,-116,
   -116,-116,-116,-116,-115,-115,-114,-114,-113,-112,-112,-111,-110,-109,-108,-107,
   -105,-104,-103,-101,-100, -98, -96, -95, -93, -91, -89, -87, -85, -83, -81, -78,
    -76, -74, -71, -68, -66, -63, -60, -58, -55, -52, -49, -46, -43, -40, -37, -34,
    -31, -28, -25, -22, -19, -16, -12,  -9,  -6,  -3,   0,   3,   6,   9,  12,  16,
     19,  22,  25,  28,  31,  34,  37,  40,  43,  46,  49,  52,  55,  58,  60,  63,
];

pub struct RasterBarsDemo {
    frame: u32,
}

impl RasterBarsDemo {
    pub const fn new() -> Self {
        Self { frame: 0 }
    }

    pub fn render(&mut self, renderer: &mut Renderer) {
        let screen_h = renderer.height() as u32;

        let copper = renderer.copper_mut();
        copper.clear();

        let phase = (self.frame & 0xFF) as usize;

        // Fundo em gradiente "céu noturno Amiga"
        let _ = copper.push(CopperOp::GradientBar {
            y: 0,
            height: screen_h,
            top: Rgb::new(0, 8, 32),
            bottom: Rgb::new(0, 0, 0),
        });

        // Faixa sutil no topo para dar mais cara de raster display
        let _ = copper.push(CopperOp::GradientBar {
            y: 0,
            height: screen_h / 4,
            top: Rgb::new(10, 20, 60),
            bottom: Rgb::new(0, 8, 32),
        });

        let center = (screen_h / 2) as i32;
        let amp = ((screen_h / 3) as i32).max(16);

        // Barras principais
        self.push_bar(copper, center, amp, phase,   0, 34, Rgb::new(255,  32,  96), 140);
        self.push_bar(copper, center, amp, phase,  32, 30, Rgb::new(255, 128,   0), 120);
        self.push_bar(copper, center, amp, phase,  64, 28, Rgb::new(255, 255,  32), 100);
        self.push_bar(copper, center, amp, phase,  96, 30, Rgb::new( 32, 255, 128), 110);
        self.push_bar(copper, center, amp, phase, 128, 32, Rgb::new( 32, 160, 255), 120);
        self.push_bar(copper, center, amp, phase, 160, 36, Rgb::new(180,  64, 255), 140);

        // Barras secundárias mais finas
        self.push_bar(
            copper,
            center,
            amp / 2,
            phase.wrapping_mul(2) & 0xFF,
            48,
            14,
            Rgb::new(255, 255, 255),
            60,
        );

        self.push_bar(
            copper,
            center,
            amp / 2,
            phase.wrapping_mul(2) & 0xFF,
            176,
            12,
            Rgb::new(64, 220, 255),
            50,
        );

        renderer.run_copper();

        self.frame = self.frame.wrapping_add(1);
    }

    fn push_bar<const N: usize>(
        &self,
        list: &mut crate::gfx::copper::CopperList<N>,
        center: i32,
        amplitude: i32,
        phase: usize,
        phase_offset: usize,
        height: u32,
        color: Rgb,
        glow: u8,
    ) {
        let s = sin8(phase.wrapping_add(phase_offset));
        let y = center + ((s as i32 * amplitude) / 127) - (height as i32 / 2);

        let _ = list.push(CopperOp::RasterBar {
            y,
            height,
            color,
            glow,
        });
    }
}

#[inline(always)]
fn sin8(idx: usize) -> i8 {
    SIN_LUT[idx & 0xFF]
}

impl crate::demos::Demo for RasterBarsDemo {
    fn render(&mut self, renderer: &mut crate::gfx::renderer::Renderer) {
        RasterBarsDemo::render(self, renderer);
    }
}