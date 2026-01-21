// Declara os arquivos que estão na mesma pasta
pub mod flame;
pub mod starfield;
pub mod plasma;
pub mod scroller;

// Você também pode definir uma "assinatura" comum para todas as demos
pub trait Demo {
    fn update(&mut self);
    fn draw(&self, fb: &mut crate::fb_driver::Framebuffer);
}