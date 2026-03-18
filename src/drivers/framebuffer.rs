use core::ptr::{addr_of_mut, read_volatile, write_volatile};

use crate::log;
use crate::platform::mailbox::mailbox_call;

const MBOX_CH_PROP: u8 = 8;

#[repr(C, align(16))]
struct MailboxBuffer {
    data: [u32; 40],
}

static mut MBOX: MailboxBuffer = MailboxBuffer { data: [0; 40] };

pub struct Framebuffer {
    pub ptr: *mut u8,
    pub width: u32,
    pub height: u32,
    pub pitch: u32,
    pub isrgb: u32,
    pub depth: u32,
}

impl Framebuffer {
    pub fn init(width: u32, height: u32, depth: u32) -> Option<Self> {
        unsafe {
            log!("FB", "building mailbox request");
            log!("FB", "req {}x{}x{}", width, height, depth);

            let m = addr_of_mut!(MBOX.data) as *mut u32;

            // 40 words = 160 bytes (FIX: era 39 * 4, faltava o último word)
            write_volatile(m.add(0), 40 * 4);
            write_volatile(m.add(1), 0);

            // set physical width/height
            write_volatile(m.add(2), 0x0004_8003);
            write_volatile(m.add(3), 8);
            write_volatile(m.add(4), 8);
            write_volatile(m.add(5), width);
            write_volatile(m.add(6), height);

            // set virtual width/height
            write_volatile(m.add(7), 0x0004_8004);
            write_volatile(m.add(8), 8);
            write_volatile(m.add(9), 8);
            write_volatile(m.add(10), width);
            write_volatile(m.add(11), height);

            // set virtual offset = (0, 0)
            write_volatile(m.add(12), 0x0004_8009);
            write_volatile(m.add(13), 8);
            write_volatile(m.add(14), 8);
            write_volatile(m.add(15), 0);
            write_volatile(m.add(16), 0);

            // set depth
            write_volatile(m.add(17), 0x0004_8005);
            write_volatile(m.add(18), 4);
            write_volatile(m.add(19), 4);
            write_volatile(m.add(20), depth);

            // set pixel order (1 = RGB)
            write_volatile(m.add(21), 0x0004_8006);
            write_volatile(m.add(22), 4);
            write_volatile(m.add(23), 4);
            write_volatile(m.add(24), 1);

            // allocate framebuffer
            write_volatile(m.add(25), 0x0004_0001);
            write_volatile(m.add(26), 8);
            write_volatile(m.add(27), 8);
            // FIX: alinhamento 4096 (era 16). O GPU escreve o endereço aqui na resposta.
            write_volatile(m.add(28), 4096);
            write_volatile(m.add(29), 0);

            // get pitch
            write_volatile(m.add(30), 0x0004_0008);
            write_volatile(m.add(31), 4);
            write_volatile(m.add(32), 4);
            write_volatile(m.add(33), 0);

            // get pixel order
            write_volatile(m.add(34), 0x0004_0006);
            write_volatile(m.add(35), 4);
            write_volatile(m.add(36), 4);
            write_volatile(m.add(37), 0);

            // end tag
            write_volatile(m.add(38), 0);
            // FIX: padding para completar os 40 words
            write_volatile(m.add(39), 0);

            log!("FB", "calling mailbox...");
            if !mailbox_call(MBOX_CH_PROP, m) {
                log!("FB", "mailbox call failed");
                return None;
            }

            log!("FB", "mailbox returned");

            // FIX: o GPU escreve o endereço do framebuffer no índice 28 (não 29).
            // O índice 29 é o tamanho alocado, não o ponteiro.
            let fb_ptr = read_volatile(m.add(28)) & 0x3FFF_FFFF;
            let pitch = read_volatile(m.add(33));
            let isrgb = read_volatile(m.add(37));

            log!("FB", "fb_ptr=0x{:08X}", fb_ptr);
            log!("FB", "pitch={}", pitch);
            log!("FB", "isrgb={}", isrgb);

            if fb_ptr == 0 || pitch == 0 {
                log!("FB", "invalid framebuffer response");
                return None;
            }

            Some(Self {
                ptr: fb_ptr as *mut u8,
                width,
                height,
                pitch,
                isrgb,
                depth,
            })
        }
    }

    #[inline(always)]
    pub fn bytes_per_pixel(&self) -> usize {
        (self.depth / 8) as usize
    }

    #[inline(always)]
    pub fn color_rgb(&self, r: u8, g: u8, b: u8) -> u32 {
        if self.isrgb != 0 {
            // XRGB
            ((r as u32) << 16) | ((g as u32) << 8) | (b as u32)
        } else {
            // XBGR
            ((b as u32) << 16) | ((g as u32) << 8) | (r as u32)
        }
    }

    #[inline(always)]
    fn pixel_offset(&self, x: u32, y: u32) -> usize {
        y as usize * self.pitch as usize + x as usize * self.bytes_per_pixel()
    }

    pub fn put_pixel(&mut self, x: u32, y: u32, color: u32) {
        if x >= self.width || y >= self.height {
            return;
        }

        if self.depth != 32 {
            log!("FB", "put_pixel only supports 32bpp for now");
            return;
        }

        let offset = self.pixel_offset(x, y);
        unsafe {
            write_volatile(self.ptr.add(offset) as *mut u32, color);
        }
    }

    // FIX: simplificado — delega para fill_rect evitando duplicação
    pub fn clear(&mut self, color: u32) {
        log!("FB", "clear color=0x{:08X}", color);
        self.fill_rect(0, 0, self.width, self.height, color);
        log!("FB", "clear done");
    }

    pub fn fill_rect(&mut self, x: u32, y: u32, w: u32, h: u32, color: u32) {
        if self.depth != 32 {
            log!("FB", "fill_rect only supports 32bpp for now");
            return;
        }

        let x0 = x.min(self.width);
        let y0 = y.min(self.height);
        let x1 = x.saturating_add(w).min(self.width);
        let y1 = y.saturating_add(h).min(self.height);

        // FIX: removidos os casts `as u32` redundantes/ambíguos
        for yy in y0..y1 {
            let row = unsafe { self.ptr.add(yy as usize * self.pitch as usize) as *mut u32 };
            for xx in x0..x1 {
                unsafe {
                    write_volatile(row.add(xx as usize), color);
                }
            }
        }
    }

    pub fn draw_gradient(&mut self) {
        log!("FB", "drawing gradient");

        if self.depth != 32 {
            log!("FB", "draw_gradient only supports 32bpp for now");
            return;
        }

        let width_max = self.width.saturating_sub(1).max(1);
        let height_max = self.height.saturating_sub(1).max(1);

        for y in 0..self.height {
            let row = unsafe { self.ptr.add(y as usize * self.pitch as usize) as *mut u32 };

            for x in 0..self.width {
                let r = ((x * 255) / width_max) as u8;
                let g = ((y * 255) / height_max) as u8;
                let b = 128u8;

                let color = self.color_rgb(r, g, b);

                unsafe {
                    write_volatile(row.add(x as usize), color);
                }
            }
        }

        log!("FB", "gradient done");
    }

    pub fn test_pattern(&mut self) {
        log!("FB", "drawing test pattern");

        let black = self.color_rgb(0, 0, 0);
        let red = self.color_rgb(255, 0, 0);
        let green = self.color_rgb(0, 255, 0);
        let blue = self.color_rgb(0, 0, 255);
        let white = self.color_rgb(255, 255, 255);

        self.clear(black);

        self.fill_rect(0, 0, self.width / 3, self.height, red);
        self.fill_rect(self.width / 3, 0, self.width / 3, self.height, green);
        self.fill_rect((self.width / 3) * 2, 0, self.width / 3, self.height, blue);

        let cx = self.width / 2;
        let cy = self.height / 2;
        self.fill_rect(cx.saturating_sub(16), cy.saturating_sub(16), 32, 32, white);

        log!("FB", "test pattern done");
    }
}