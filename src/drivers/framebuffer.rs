// src/drivers/framebuffer.rs

use core::ptr::{addr_of_mut, read_volatile, write_volatile};

use crate::log;
use crate::platform::raspi3::mailbox::mailbox_call;

const MBOX_CH_PROP: u8 = 8;
const MBOX_WORDS: usize = 40;

// Mailbox tags
const TAG_SET_PHYS_WH: u32 = 0x0004_8003;
const TAG_SET_VIRT_WH: u32 = 0x0004_8004;
const TAG_SET_VIRT_OFFSET: u32 = 0x0004_8009;
const TAG_SET_DEPTH: u32 = 0x0004_8005;
const TAG_SET_PIXEL_ORDER: u32 = 0x0004_8006;
const TAG_ALLOCATE_FB: u32 = 0x0004_0001;
const TAG_GET_PITCH: u32 = 0x0004_0008;
const TAG_END: u32 = 0x0000_0000;

#[repr(C, align(16))]
struct MailboxBuffer {
    data: [u32; MBOX_WORDS],
}

static mut MBOX: MailboxBuffer = MailboxBuffer {
    data: [0; MBOX_WORDS],
};

pub struct Framebuffer {
    pub ptr: *mut u8,
    pub width: u32,
    pub height: u32,
    pub pitch: u32,
    pub isrgb: u32,
    pub depth: u32,
}

impl Framebuffer {
    pub fn init(width: u32, height: u32, requested_depth: u32) -> Option<Self> {
        unsafe {
            log!(
                "FB",
                "building mailbox request ({}x{}x{})",
                width,
                height,
                requested_depth
            );

            let m = addr_of_mut!(MBOX.data) as *mut u32;

            for i in 0..MBOX_WORDS {
                write_volatile(m.add(i), 0);
            }

            // Tamanho total do buffer em bytes
            write_volatile(m.add(0), (MBOX_WORDS as u32 * 4).to_le());
            // Request code
            write_volatile(m.add(1), 0u32.to_le());

            // Tag: Set Physical Display Width/Height
            write_volatile(m.add(2), TAG_SET_PHYS_WH.to_le());
            write_volatile(m.add(3), 8u32.to_le());
            write_volatile(m.add(4), 8u32.to_le());
            write_volatile(m.add(5), width.to_le());
            write_volatile(m.add(6), height.to_le());

            // Tag: Set Virtual Display Width/Height
            write_volatile(m.add(7), TAG_SET_VIRT_WH.to_le());
            write_volatile(m.add(8), 8u32.to_le());
            write_volatile(m.add(9), 8u32.to_le());
            write_volatile(m.add(10), width.to_le());
            write_volatile(m.add(11), height.to_le());

            // Tag: Set Virtual Offset
            write_volatile(m.add(12), TAG_SET_VIRT_OFFSET.to_le());
            write_volatile(m.add(13), 8u32.to_le());
            write_volatile(m.add(14), 8u32.to_le());
            write_volatile(m.add(15), 0u32.to_le());
            write_volatile(m.add(16), 0u32.to_le());

            // Tag: Set Depth
            write_volatile(m.add(17), TAG_SET_DEPTH.to_le());
            write_volatile(m.add(18), 4u32.to_le());
            write_volatile(m.add(19), 4u32.to_le());
            write_volatile(m.add(20), requested_depth.to_le());

            // Tag: Set Pixel Order (1 = RGB, 0 = BGR)
            write_volatile(m.add(21), TAG_SET_PIXEL_ORDER.to_le());
            write_volatile(m.add(22), 4u32.to_le());
            write_volatile(m.add(23), 4u32.to_le());
            write_volatile(m.add(24), 1u32.to_le());

            // Tag: Allocate Framebuffer
            write_volatile(m.add(25), TAG_ALLOCATE_FB.to_le());
            write_volatile(m.add(26), 8u32.to_le());
            write_volatile(m.add(27), 8u32.to_le());
            write_volatile(m.add(28), 4096u32.to_le()); // alignment
            write_volatile(m.add(29), 0u32.to_le());    // size out

            // Tag: Get Pitch
            write_volatile(m.add(30), TAG_GET_PITCH.to_le());
            write_volatile(m.add(31), 4u32.to_le());
            write_volatile(m.add(32), 4u32.to_le());
            write_volatile(m.add(33), 0u32.to_le());

            // Tag: End
            write_volatile(m.add(34), TAG_END.to_le());

            let addr = m as usize;
            let size = core::mem::size_of::<MailboxBuffer>();

            crate::arch::aarch64::regs::cache::flush_range(addr, addr + size);

            log!("FB", "calling mailbox...");
            if !mailbox_call(MBOX_CH_PROP, m) {
                log!("FB", "mailbox call failed");
                return None;
            }

            crate::arch::aarch64::regs::cache::invalidate_range(addr, addr + size);

            let real_width = read_tag_u32(m, TAG_SET_PHYS_WH, 0).unwrap_or(width);
            let real_height = read_tag_u32(m, TAG_SET_PHYS_WH, 1).unwrap_or(height);
            let real_depth = read_tag_u32(m, TAG_SET_DEPTH, 0).unwrap_or(0);
            let isrgb = read_tag_u32(m, TAG_SET_PIXEL_ORDER, 0).unwrap_or(0);
            let fb_ptr = read_tag_u32(m, TAG_ALLOCATE_FB, 0).unwrap_or(0) & 0x3FFF_FFFF;
            let fb_size = read_tag_u32(m, TAG_ALLOCATE_FB, 1).unwrap_or(0);
            let pitch = read_tag_u32(m, TAG_GET_PITCH, 0).unwrap_or(0);

            log!(
                "FB",
                "GPU: ptr=0x{:08X}, size={}, pitch={}, depth={}, isrgb={}, width={}, height={}",
                fb_ptr,
                fb_size,
                pitch,
                real_depth,
                isrgb,
                real_width,
                real_height
            );

            if fb_ptr == 0 {
                log!("FB", "Invalid GPU response: null framebuffer ptr");
                return None;
            }

            if pitch == 0 {
                log!("FB", "Invalid GPU response: zero pitch");
                return None;
            }

            if real_width == 0 || real_height == 0 {
                log!(
                    "FB",
                    "Invalid GPU response: invalid dimensions {}x{}",
                    real_width,
                    real_height
                );
                return None;
            }

            if real_depth != 32 {
                log!(
                    "FB",
                    "Unsupported depth returned by GPU: {} (expected 32)",
                    real_depth
                );
                return None;
            }

            let min_pitch = real_width.saturating_mul(real_depth / 8);
            if pitch < min_pitch {
                log!(
                    "FB",
                    "Invalid GPU response: pitch {} < minimum {}",
                    pitch,
                    min_pitch
                );
                return None;
            }

            Some(Self {
                ptr: fb_ptr as *mut u8,
                width: real_width,
                height: real_height,
                pitch,
                isrgb,
                depth: real_depth,
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
            ((b as u32) << 16) | ((g as u32) << 8) | (r as u32)
        } else {
            ((r as u32) << 16) | ((g as u32) << 8) | (b as u32)
        }
    }

    #[inline(always)]
    fn pixel_offset(&self, x: u32, y: u32) -> usize {
        y as usize * self.pitch as usize + x as usize * self.bytes_per_pixel()
    }

    #[inline(always)]
    fn decode_argb(argb: u32) -> (u8, u8, u8, u8) {
        let a = ((argb >> 24) & 0xFF) as u8;
        let r = ((argb >> 16) & 0xFF) as u8;
        let g = ((argb >> 8) & 0xFF) as u8;
        let b = (argb & 0xFF) as u8;
        (a, r, g, b)
    }

    pub fn put_pixel(&mut self, x: u32, y: u32, color: u32) {
        if x >= self.width || y >= self.height {
            return;
        }
        if self.depth != 32 {
            return;
        }

        let offset = self.pixel_offset(x, y);
        unsafe {
            write_volatile(self.ptr.add(offset) as *mut u32, color.to_le());
        }
    }

    pub fn blit_argb(&mut self, src: &[u32]) {
        if self.depth != 32 {
            log!(
                "FB",
                "blit_argb only supports 32bpp (current: {})",
                self.depth
            );
            return;
        }

        let width = self.width as usize;
        let height = self.height as usize;
        let needed = width.saturating_mul(height);

        if src.len() < needed {
            log!("FB", "source too small: {} < {}", src.len(), needed);
            return;
        }

        if self.pitch < (self.width * 4) {
            log!(
                "FB",
                "invalid pitch for blit: pitch={} width={}",
                self.pitch,
                self.width
            );
            return;
        }

        for y in 0..height {
            let dst_row = unsafe { self.ptr.add(y * self.pitch as usize) as *mut u32 };
            let src_row = &src[y * width..(y + 1) * width];

            for x in 0..width {
                let (_, r, g, b) = Self::decode_argb(src_row[x]);
                let hw_color = self.color_rgb(r, g, b);
                unsafe {
                    write_volatile(dst_row.add(x), hw_color.to_le());
                }
            }
        }
    }

    pub fn clear(&mut self, color: u32) {
        self.fill_rect(0, 0, self.width, self.height, color);
    }

    pub fn fill_rect(&mut self, x: u32, y: u32, w: u32, h: u32, color: u32) {
        if self.depth != 32 {
            return;
        }

        let x0 = x.min(self.width);
        let y0 = y.min(self.height);
        let x1 = x.saturating_add(w).min(self.width);
        let y1 = y.saturating_add(h).min(self.height);

        for yy in y0..y1 {
            let row = unsafe { self.ptr.add(yy as usize * self.pitch as usize) as *mut u32 };
            for xx in x0..x1 {
                unsafe {
                    write_volatile(row.add(xx as usize), color.to_le());
                }
            }
        }
    }
}

#[inline(always)]
fn read_tag_u32(m: *mut u32, wanted_tag: u32, value_index: usize) -> Option<u32> {
    let mut i = 2usize;

    while i + 2 < MBOX_WORDS {
        let tag = u32::from_le(unsafe { read_volatile(m.add(i)) });
        if tag == TAG_END {
            return None;
        }

        let value_buf_size_bytes = u32::from_le(unsafe { read_volatile(m.add(i + 1)) }) as usize;
        let _resp_size_bytes = u32::from_le(unsafe { read_volatile(m.add(i + 2)) }) as usize;

        if value_buf_size_bytes == 0 {
            return None;
        }

        let value_words = value_buf_size_bytes / 4;
        let value_start = i + 3;
        let value_pos = value_start + value_index;

        if tag == wanted_tag {
            if value_index < value_words && value_pos < MBOX_WORDS {
                let v = unsafe { read_volatile(m.add(value_pos)) };
                return Some(u32::from_le(v));
            } else {
                return None;
            }
        }

        i = value_start + value_words;
    }

    None
}