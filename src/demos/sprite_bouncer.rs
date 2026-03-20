// src/demos/sprite_bouncer.rs

use libm::sinf;

use crate::demos::Demo;
use crate::gfx::renderer::Renderer;
use crate::gfx::sprite::{Sprite, SpriteFlags, SpriteInstance};
use crate::media::FrameContext;

const SPRITE_W: usize = 16;
const SPRITE_H: usize = 16;
const TWO_PI: f32 = 6.2831855;

// Sprite 16x16 simples em ARGB8888.
// 0x00000000 = transparente
static BALL_PIXELS: [u32; SPRITE_W * SPRITE_H] = [
    0,0,0,0,0,0xFFFFD54A,0xFFFFD54A,0xFFFFD54A,0xFFFFD54A,0xFFFFD54A,0,0,0,0,0,0,
    0,0,0,0xFFFFD54A,0xFFFFD54A,0xFFFFE082,0xFFFFE082,0xFFFFE082,0xFFFFE082,0xFFFFD54A,0xFFFFD54A,0xFFFFD54A,0,0,0,0,
    0,0,0xFFFFD54A,0xFFFFE082,0xFFFFF3C4,0xFFFFF3C4,0xFFFFF3C4,0xFFFFF3C4,0xFFFFF3C4,0xFFFFE082,0xFFFFD54A,0xFFFFD54A,0xFFFFD54A,0,0,0,
    0,0xFFFFD54A,0xFFFFE082,0xFFFFF3C4,0xFFFFF3C4,0xFFFFF3C4,0xFFFFF3C4,0xFFFFF3C4,0xFFFFF3C4,0xFFFFF3C4,0xFFFFE082,0xFFFFD54A,0xFFFFD54A,0xFFFFD54A,0,0,
    0,0xFFFFD54A,0xFFFFF3C4,0xFFFFF3C4,0xFFFFF3C4,0xFFFFF3C4,0xFFFFF3C4,0xFFFFF3C4,0xFFFFF3C4,0xFFFFF3C4,0xFFFFF3C4,0xFFFFE082,0xFFFFD54A,0xFFFFD54A,0,0,
    0xFFFFD54A,0xFFFFE082,0xFFFFF3C4,0xFFFFF3C4,0xFFFFF3C4,0xFFFFF3C4,0xFFFFFFFF,0xFFFFFFFF,0xFFFFF3C4,0xFFFFF3C4,0xFFFFF3C4,0xFFFFF3C4,0xFFFFE082,0xFFFFD54A,0xFFFFD54A,0,
    0xFFFFD54A,0xFFFFE082,0xFFFFF3C4,0xFFFFF3C4,0xFFFFF3C4,0xFFFFFFFF,0xFF000000,0xFF000000,0xFFFFFFFF,0xFFFFF3C4,0xFFFFF3C4,0xFFFFF3C4,0xFFFFE082,0xFFFFD54A,0xFFFFD54A,0,
    0xFFFFD54A,0xFFFFE082,0xFFFFF3C4,0xFFFFF3C4,0xFFFFF3C4,0xFFFFFFFF,0xFF000000,0xFF000000,0xFFFFFFFF,0xFFFFF3C4,0xFFFFF3C4,0xFFFFF3C4,0xFFFFE082,0xFFFFD54A,0xFFFFD54A,0,
    0xFFFFD54A,0xFFFFE082,0xFFFFF3C4,0xFFFFF3C4,0xFFFFF3C4,0xFFFFF3C4,0xFFFFFFFF,0xFFFFFFFF,0xFFFFF3C4,0xFFFFF3C4,0xFFFFF3C4,0xFFFFF3C4,0xFFFFE082,0xFFFFD54A,0xFFFFD54A,0,
    0,0xFFFFD54A,0xFFFFF3C4,0xFFFFF3C4,0xFFFFF3C4,0xFFFFF3C4,0xFFFFF3C4,0xFFFFF3C4,0xFFFFF3C4,0xFFFFF3C4,0xFFFFF3C4,0xFFFFE082,0xFFFFD54A,0xFFFFD54A,0,0,
    0,0xFFFFD54A,0xFFFFE082,0xFFFFF3C4,0xFFFFF3C4,0xFFFFF3C4,0xFFFFF3C4,0xFFFFF3C4,0xFFFFF3C4,0xFFFFF3C4,0xFFFFE082,0xFFFFD54A,0xFFFFD54A,0xFFFFD54A,0,0,
    0,0,0xFFFFD54A,0xFFFFE082,0xFFFFF3C4,0xFFFFF3C4,0xFFFFF3C4,0xFFFFF3C4,0xFFFFF3C4,0xFFFFE082,0xFFFFD54A,0xFFFFD54A,0xFFFFD54A,0,0,0,
    0,0,0,0xFFFFD54A,0xFFFFD54A,0xFFFFE082,0xFFFFE082,0xFFFFE082,0xFFFFE082,0xFFFFD54A,0xFFFFD54A,0xFFFFD54A,0,0,0,0,
    0,0,0,0,0xFFFFD54A,0xFFFFD54A,0xFFFFD54A,0xFFFFD54A,0xFFFFD54A,0xFFFFD54A,0xFFFFD54A,0,0,0,0,0,
    0,0,0,0,0,0xFFFFD54A,0xFFFFD54A,0xFFFFD54A,0xFFFFD54A,0xFFFFD54A,0,0,0,0,0,0,
    0,0,0,0,0,0,0,0xFFFFD54A,0xFFFFD54A,0,0,0,0,0,0,0,
];

pub struct SpriteBouncerDemo {
    x: f32,
    y: f32,
    vx: f32,
    phase: f32,
    debug_counter: u32,
}

impl SpriteBouncerDemo {
    pub fn new() -> Self {
        crate::log!("SPRITE", "SpriteBouncerDemo::new");

        Self {
            x: 32.0,
            y: 80.0,
            vx: 120.0,
            phase: 0.0,
            debug_counter: 0,
        }
    }

    fn sprite() -> Sprite<'static> {
        Sprite::new(&BALL_PIXELS, SPRITE_W, SPRITE_H)
    }

    fn draw_debug_overlay(
        &self,
        renderer: &mut Renderer,
        frame: &FrameContext,
        dt: f32,
        y: f32,
        bounce: f32,
    ) {
        renderer.draw_str_transparent(12, 40, "debug sprite", 0xFFFFFF00);

        if dt <= 0.000001 {
            renderer.draw_str_transparent(12, 52, "dt=ZERO", 0xFFFF4040);
        } else {
            renderer.draw_str_transparent(12, 52, "dt=OK", 0xFF80FF80);
        }

        if self.vx < 0.0 {
            renderer.draw_str_transparent(12, 64, "dir=LEFT", 0xFFFFFFFF);
        } else {
            renderer.draw_str_transparent(12, 64, "dir=RIGHT", 0xFFFFFFFF);
        }

        if (self.x as i32) < 0 || (self.x as usize) > renderer.width() {
            renderer.draw_str_transparent(12, 76, "x=OUT", 0xFFFF4040);
        } else {
            renderer.draw_str_transparent(12, 76, "x=IN", 0xFF80FF80);
        }

        if (y as i32) < 0 || (y as usize) > renderer.height() {
            renderer.draw_str_transparent(12, 88, "y=OUT", 0xFFFF4040);
        } else {
            renderer.draw_str_transparent(12, 88, "y=IN", 0xFF80FF80);
        }

        if frame.frame_dt_ticks == 0 {
            renderer.draw_str_transparent(12, 100, "ticks/frame=0", 0xFFFF4040);
        } else {
            renderer.draw_str_transparent(12, 100, "ticks/frame>0", 0xFF80FF80);
        }

        let _ = bounce; // útil se você quiser colocar breakpoint/inspecionar depois
    }
}

impl Demo for SpriteBouncerDemo {
    fn render(&mut self, renderer: &mut Renderer, frame: &FrameContext) {
        let raw_dt = frame.frame_dt_secs;
        let dt = raw_dt.clamp(0.0, 0.05);

        self.x += self.vx * dt;
        self.phase += dt * 4.0;

        if self.phase >= TWO_PI {
            self.phase -= TWO_PI;
        }

        let min_x = 0.0;
        let max_x = (renderer.width().saturating_sub(SPRITE_W)) as f32;

        if self.x <= min_x {
            self.x = min_x;
            self.vx = self.vx.abs();
            crate::log!("SPRITE", "bounce left edge x={}", self.x as i32);
        } else if self.x >= max_x {
            self.x = max_x;
            self.vx = -self.vx.abs();
            crate::log!("SPRITE", "bounce right edge x={}", self.x as i32);
        }

        let bounce = sinf(self.phase) * 18.0;
        let y = self.y + bounce;

        self.debug_counter = self.debug_counter.wrapping_add(1);
        if self.debug_counter % 60 == 0 {
            crate::log!(
                "SPRITE",
                "frame={} dt_ticks={} dt_ms={} x={} y={} vx={} phase={}",
                frame.frame,
                frame.frame_dt_ticks,
                (dt * 1000.0) as i32,
                self.x as i32,
                y as i32,
                self.vx as i32,
                (self.phase * 1000.0) as i32,
            );
        }

        renderer.clear(0xFF101820);

        for stripe in 0..renderer.height() / 16 {
            let c = if stripe & 1 == 0 { 0xFF142030 } else { 0xFF18283C };
            renderer.fill_rect(0, stripe * 16, renderer.width(), 16, c);
        }

        renderer.draw_str_transparent(12, 12, "SPRITE BOUNCER", 0xFFFFFFFF);
        renderer.draw_str_transparent(12, 24, "usa Sprite + SpriteInstance", 0xFFB0C4DE);

        self.draw_debug_overlay(renderer, frame, dt, y, bounce);

        let sprite = Self::sprite();

        let mut inst = SpriteInstance::new(&sprite, self.x as i32, y as i32).with_priority(10);

        if self.vx < 0.0 {
            inst = inst.with_flags(SpriteFlags::FLIP_X);
        }

        renderer.draw_sprite_instance(&inst);

        renderer.draw_line(
            0,
            (self.y as i32) + (SPRITE_H as i32) + 8,
            renderer.width() as i32 - 1,
            (self.y as i32) + (SPRITE_H as i32) + 8,
            0xFF406080,
        );
    }
}