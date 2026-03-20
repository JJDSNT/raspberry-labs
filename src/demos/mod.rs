// src/demos/mod.rs

pub mod flame;
pub mod starfield;
pub mod plasma;
pub mod rasterbars;
pub mod scroller;
pub mod tunnel;
pub mod parallax;
pub mod juggler;

use crate::drivers::framebuffer::Framebuffer;
use crate::gfx::renderer::Renderer;

pub trait Demo {
    fn render(&mut self, renderer: &mut Renderer);
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
}

impl DemoKind {
    pub const fn as_str(&self) -> &'static str {
        match self {
            DemoKind::RasterBars => "RasterBars",
            DemoKind::Plasma     => "Plasma",
            DemoKind::Flame      => "Flame",
            DemoKind::Starfield  => "Starfield",
            DemoKind::Tunnel     => "Tunnel",
            DemoKind::Parallax   => "Parallax",
            DemoKind::Juggler    => "Juggler",
        }
    }
}

pub fn run_demo(kind: DemoKind, fb: Framebuffer) -> ! {
    match kind {
        DemoKind::RasterBars => run_renderer_demo(fb, rasterbars::RasterBarsDemo::new()),
        DemoKind::Plasma     => run_renderer_demo(fb, plasma::Plasma::new()),
        DemoKind::Flame      => run_renderer_demo(fb, flame::FlameDemo::new()),
        DemoKind::Starfield  => run_renderer_demo(fb, starfield::StarfieldDemo::new()),
        DemoKind::Tunnel     => run_renderer_demo(fb, tunnel::TunnelDemo::new()),
        DemoKind::Parallax   => run_renderer_demo(fb, parallax::ParallaxDemo::new()),
        DemoKind::Juggler    => run_renderer_demo(fb, juggler::JugglerDemo::new()),
    }
}

fn run_renderer_demo<D: Demo>(fb: Framebuffer, mut demo: D) -> ! {
    let mut renderer = Renderer::new(fb);
    loop {
        demo.render(&mut renderer);
        renderer.present();
    }
}