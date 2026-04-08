// src/demos/mod.rs

pub mod flame;
pub mod starfield;
pub mod plasma;
pub mod rasterbars;
pub mod scroller;
pub mod tunnel;
pub mod parallax;
pub mod juggler;
pub mod sprite_bouncer;
pub mod gfx3d_triangle;

use crate::drivers::framebuffer::Framebuffer;
use crate::gfx::renderer::Renderer;
use crate::kernel::time;
use crate::media::clock::{FrameContext, MediaClock};

pub trait Demo {
    fn render(&mut self, renderer: &mut Renderer, frame: &FrameContext);
}

#[derive(Clone, Copy, Debug)]
pub enum DemoKind {
    RasterBars,
    Plasma,
    Flame,
    Starfield,
    Tunnel,
    Parallax,
    Juggler,
    SpriteBouncer,
    Gfx3dTriangle, // 🔥 novo
    Omega,
}

impl DemoKind {
    pub const fn as_str(&self) -> &'static str {
        match self {
            DemoKind::RasterBars    => "RasterBars",
            DemoKind::Plasma        => "Plasma",
            DemoKind::Flame         => "Flame",
            DemoKind::Starfield     => "Starfield",
            DemoKind::Tunnel        => "Tunnel",
            DemoKind::Parallax      => "Parallax",
            DemoKind::Juggler       => "Juggler",
            DemoKind::SpriteBouncer => "SpriteBouncer",
            DemoKind::Gfx3dTriangle => "Gfx3dTriangle", // 🔥 novo
            DemoKind::Omega         => "Omega",
        }
    }
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

pub fn run_demo(kind: DemoKind, fb: Framebuffer) -> ! {
    crate::log!("DEMO", "run_demo kind={}", kind.as_str());

    match kind {
        DemoKind::RasterBars    => run_renderer_demo(fb, rasterbars::RasterBarsDemo::new()),
        DemoKind::Plasma        => run_renderer_demo(fb, plasma::Plasma::new()),
        DemoKind::Flame         => run_renderer_demo(fb, flame::FlameDemo::new()),
        DemoKind::Starfield     => run_renderer_demo(fb, starfield::StarfieldDemo::new()),
        DemoKind::Tunnel        => run_renderer_demo(fb, tunnel::TunnelDemo::new()),
        DemoKind::Parallax      => run_renderer_demo(fb, parallax::ParallaxDemo::new()),
        DemoKind::Juggler       => run_renderer_demo(fb, juggler::JugglerDemo::new()),
        DemoKind::SpriteBouncer => run_renderer_demo(fb, sprite_bouncer::SpriteBouncerDemo::new()),
        DemoKind::Gfx3dTriangle => run_renderer_demo(fb, gfx3d_triangle::Gfx3dTriangleDemo::new()),

        #[cfg(not(target_os = "uefi"))]
        DemoKind::Omega         => crate::emu::run(fb),
        #[cfg(target_os = "uefi")]
        DemoKind::Omega         => loop { core::hint::spin_loop(); },
    }
}

// ---------------------------------------------------------------------------
// Main loop com debug pesado
// ---------------------------------------------------------------------------

fn run_renderer_demo<D: Demo>(fb: Framebuffer, mut demo: D) -> ! {
    let mut renderer = Renderer::new(fb);
    crate::log!("DEMO", "renderer ok pixels={} {}x{}", renderer.pixels(), renderer.width(), renderer.height());
    let ticks_per_second = time::ticks_per_second();
    assert!(ticks_per_second != 0, "kernel time not initialized");

    let mut clock = MediaClock::new(ticks_per_second, 60);
    let mut frame_counter: u64 = 0;

    loop {
        frame_counter = frame_counter.wrapping_add(1);

        let frame = clock.begin_frame();
        demo.render(&mut renderer, &frame);
        renderer.present();

        // Cede para a idle (ou qualquer outra task futura)
        crate::kernel::scheduler::yield_now();
    }
}