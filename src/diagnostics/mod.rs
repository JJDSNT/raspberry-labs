// src/diagnostics/mod.rs

pub mod gradient;
pub mod test_pattern;
pub mod smpte;

use crate::demos::Demo;
use crate::drivers::framebuffer::Framebuffer;
use crate::gfx::renderer::Renderer;
use crate::kernel::time;
use crate::media::MediaClock;

#[derive(Clone, Copy, Debug)]
pub enum DiagKind {
    Gradient,
    TestPattern,
    Smpte,
}

impl DiagKind {
    pub const fn as_str(&self) -> &'static str {
        match self {
            DiagKind::Gradient    => "Gradient",
            DiagKind::TestPattern => "TestPattern",
            DiagKind::Smpte       => "Smpte",
        }
    }
}

pub fn run_diag(kind: DiagKind, fb: Framebuffer) -> ! {
    match kind {
        DiagKind::Gradient    => run_renderer_demo(fb, gradient::GradientDiag::new()),
        DiagKind::TestPattern => run_renderer_demo(fb, test_pattern::TestPatternDiag::new()),
        DiagKind::Smpte       => run_renderer_demo(fb, smpte::SmpteDiag::new()),
    }
}

fn run_renderer_demo<D: Demo>(fb: Framebuffer, mut diag: D) -> ! {
    let mut renderer = Renderer::new(fb);

    let ticks_per_second = time::ticks_per_second();
    assert!(ticks_per_second != 0, "kernel time not initialized");

    let mut clock = MediaClock::new(ticks_per_second, 60);

    loop {
        let frame = clock.begin_frame();
        diag.render(&mut renderer, &frame);
        renderer.present();
    }
}