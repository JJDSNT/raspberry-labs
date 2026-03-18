pub mod flame;
pub mod starfield;
pub mod plasma;
pub mod rasterbars;
pub mod scroller;

pub trait Demo {
    fn update(&mut self);
    fn draw(&self, fb: &mut crate::drivers::framebuffer::Framebuffer);
}