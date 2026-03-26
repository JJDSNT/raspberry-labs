#![allow(dead_code)]

pub struct SoftFramebuffer<'a> {
    width: usize,
    height: usize,
    pixels: &'a mut [u32],
}

impl<'a> SoftFramebuffer<'a> {
    pub fn new(width: usize, height: usize, pixels: &'a mut [u32]) -> Self {
        assert_eq!(pixels.len(), width * height);
        Self {
            width,
            height,
            pixels,
        }
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn pixels(&self) -> &[u32] {
        self.pixels
    }

    pub fn pixels_mut(&mut self) -> &mut [u32] {
        self.pixels
    }

    pub fn clear(&mut self, color: u32) {
        for pixel in self.pixels.iter_mut() {
            *pixel = color;
        }
    }

    pub fn put_pixel(&mut self, x: i32, y: i32, color: u32) {
        if x < 0 || y < 0 {
            return;
        }

        let x = x as usize;
        let y = y as usize;

        if x >= self.width || y >= self.height {
            return;
        }

        let index = y * self.width + x;
        self.pixels[index] = color;
    }

    pub fn get_pixel(&self, x: i32, y: i32) -> Option<u32> {
        if x < 0 || y < 0 {
            return None;
        }

        let x = x as usize;
        let y = y as usize;

        if x >= self.width || y >= self.height {
            return None;
        }

        let index = y * self.width + x;
        Some(self.pixels[index])
    }
}