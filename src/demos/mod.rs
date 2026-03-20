pub mod flame;
pub mod starfield;
pub mod plasma;
pub mod rasterbars;
pub mod scroller;
pub mod tunnel;
pub mod parallax;

use crate::drivers::framebuffer::Framebuffer;
use crate::gfx::renderer::Renderer;

pub trait Demo {
    fn render(&mut self, renderer: &mut Renderer);
}

#[derive(Clone, Copy, Debug)]
pub enum DemoKind {
    Gradient,
    TestPattern,
    RasterBars,
    Plasma,
    Flame,
    Starfield,
    Tunnel,
    Parallax,
}

impl DemoKind {
    pub const fn as_str(&self) -> &'static str {
        match self {
            DemoKind::Gradient    => "Gradient",
            DemoKind::TestPattern => "TestPattern",
            DemoKind::RasterBars  => "RasterBars",
            DemoKind::Plasma      => "Plasma",
            DemoKind::Flame       => "Flame",
            DemoKind::Starfield   => "Starfield",
            DemoKind::Tunnel      => "Tunnel",
            DemoKind::Parallax    => "Parallax",
        }
    }
}

pub fn run_demo(kind: DemoKind, fb: Framebuffer) -> ! {
    match kind {
        DemoKind::Gradient    => run_gradient(fb),
        DemoKind::TestPattern => run_test_pattern(fb),
        DemoKind::RasterBars  => run_renderer_demo(fb, rasterbars::RasterBarsDemo::new()),
        DemoKind::Plasma      => run_renderer_demo(fb, plasma::Plasma::new()),
        DemoKind::Flame       => run_renderer_demo(fb, flame::FlameDemo::new()),
        DemoKind::Starfield   => run_renderer_demo(fb, starfield::StarfieldDemo::new()),
        DemoKind::Tunnel      => run_renderer_demo(fb, tunnel::TunnelDemo::new()),
        DemoKind::Parallax    => run_renderer_demo(fb, parallax::ParallaxDemo::new()),
    }
}

fn run_renderer_demo<D: Demo>(fb: Framebuffer, mut demo: D) -> ! {
    let mut renderer = Renderer::new(fb);

    loop {
        demo.render(&mut renderer);
        renderer.present();
    }
}

fn run_gradient(mut fb: Framebuffer) -> ! {
    fb.draw_gradient();

    let white = fb.color_rgb(255, 255, 255);
    let red   = fb.color_rgb(255, 0,   0  );
    let green = fb.color_rgb(0,   255, 0  );
    let blue  = fb.color_rgb(0,   0,   255);

    let w = fb.width;
    let h = fb.height;

    fb.fill_rect(w / 16,        h / 10, w / 8, h / 8, white);
    fb.fill_rect((w / 16) * 4,  h / 10, w / 8, h / 8, red);
    fb.fill_rect((w / 16) * 7,  h / 10, w / 8, h / 8, green);
    fb.fill_rect((w / 16) * 10, h / 10, w / 8, h / 8, blue);

    fb.put_pixel(w / 2, h / 2, white);

    loop {
        core::hint::spin_loop();
    }
}

fn run_test_pattern(mut fb: Framebuffer) -> ! {
    fb.test_pattern();

    loop {
        core::hint::spin_loop();
    }
}