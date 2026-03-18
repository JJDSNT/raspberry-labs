use crate::platform::mailbox::mailbox_call;

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
            let m = &mut MBOX.data;

            m[0] = 35 * 4;
            m[1] = 0;

            m[2] = 0x0004_8003;
            m[3] = 8;
            m[4] = 8;
            m[5] = width;
            m[6] = height;

            m[7] = 0x0004_8004;
            m[8] = 8;
            m[9] = 8;
            m[10] = width;
            m[11] = height;

            m[12] = 0x0004_8005;
            m[13] = 4;
            m[14] = 4;
            m[15] = depth;

            m[16] = 0x0004_8006;
            m[17] = 4;
            m[18] = 4;
            m[19] = 1;

            m[20] = 0x0004_0001;
            m[21] = 8;
            m[22] = 8;
            m[23] = 16;
            m[24] = 0;

            m[25] = 0x0004_0008;
            m[26] = 4;
            m[27] = 4;
            m[28] = 0;

            m[29] = 0x0004_0006;
            m[30] = 4;
            m[31] = 4;
            m[32] = 0;

            m[33] = 0;

            if !mailbox_call(MBOX_CH_PROP, m.as_mut_ptr()) {
                return None;
            }

            let fb_ptr = m[24] & 0x3FFF_FFFF;
            let pitch = m[28];
            let isrgb = m[32];

            if fb_ptr == 0 || pitch == 0 {
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
        for y in 0..self.height as usize {
            let row = unsafe { self.ptr.add(y * self.pitch as usize) as *mut u32 };
            for x in 0..self.width as usize {
                unsafe {
                    core::ptr::write_volatile(row.add(x), color);
                }
            }
        }
    }
}