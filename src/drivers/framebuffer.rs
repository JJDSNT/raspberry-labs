use core::ptr::{addr_of_mut, read_volatile, write_volatile};

use crate::platform::mailbox::mailbox_call;
use crate::log;

const MBOX_CH_PROP: u8 = 8;

#[repr(C, align(16))]
struct MailboxBuffer {
    data: [u32; 36],
}

static mut MBOX: MailboxBuffer = MailboxBuffer { data: [0; 36] };

pub struct Framebuffer {
    pub ptr: *mut u8,
    pub width: u32,
    pub height: u32,
    pub pitch: u32,
    pub isrgb: u32,
}

impl Framebuffer {
    pub fn init(width: u32, height: u32, depth: u32) -> Option<Self> {
        unsafe {
            log!("FB", "building mailbox request");
            log!("FB", "req {}x{}x{}", width, height, depth);

            let m = addr_of_mut!(MBOX.data) as *mut u32;

            // total size = 34 words = 136 bytes
            write_volatile(m.add(0), 34 * 4);
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

            // set depth
            write_volatile(m.add(12), 0x0004_8005);
            write_volatile(m.add(13), 4);
            write_volatile(m.add(14), 4);
            write_volatile(m.add(15), depth);

            // set pixel order (1 = RGB)
            write_volatile(m.add(16), 0x0004_8006);
            write_volatile(m.add(17), 4);
            write_volatile(m.add(18), 4);
            write_volatile(m.add(19), 1);

            // allocate framebuffer
            write_volatile(m.add(20), 0x0004_0001);
            write_volatile(m.add(21), 8);
            write_volatile(m.add(22), 8);
            write_volatile(m.add(23), 16);
            write_volatile(m.add(24), 0);

            // get pitch
            write_volatile(m.add(25), 0x0004_0008);
            write_volatile(m.add(26), 4);
            write_volatile(m.add(27), 4);
            write_volatile(m.add(28), 0);

            // get pixel order
            write_volatile(m.add(29), 0x0004_0006);
            write_volatile(m.add(30), 4);
            write_volatile(m.add(31), 4);
            write_volatile(m.add(32), 0);

            // end tag
            write_volatile(m.add(33), 0);

            log!("FB", "calling mailbox...");
            if !mailbox_call(MBOX_CH_PROP, m) {
                log!("FB", "mailbox call failed");
                return None;
            }

            log!("FB", "mailbox returned");

            let fb_ptr = read_volatile(m.add(24)) & 0x3FFF_FFFF;
            let pitch = read_volatile(m.add(28));
            let isrgb = read_volatile(m.add(32));

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
            })
        }
    }

    pub fn clear(&mut self, color: u32) {
        log!("FB", "clear color=0x{:08X}", color);

        for y in 0..self.height as usize {
            let row = unsafe { self.ptr.add(y * self.pitch as usize) as *mut u32 };
            for x in 0..self.width as usize {
                unsafe {
                    write_volatile(row.add(x), color);
                }
            }
        }

        log!("FB", "clear done");
    }
}