const MMIO_BASE: usize = 0x3F00_0000;
const MAILBOX_BASE: usize = MMIO_BASE + 0x0000_B880;

const MBOX_READ: usize = MAILBOX_BASE + 0x00;
const MBOX_STATUS: usize = MAILBOX_BASE + 0x18;
const MBOX_WRITE: usize = MAILBOX_BASE + 0x20;

const MBOX_FULL: u32 = 0x8000_0000;
const MBOX_EMPTY: u32 = 0x4000_0000;

#[inline(always)]
fn mmio_write(addr: usize, value: u32) {
    unsafe {
        core::ptr::write_volatile(addr as *mut u32, value);
    }
}

#[inline(always)]
fn mmio_read(addr: usize) -> u32 {
    unsafe { core::ptr::read_volatile(addr as *const u32) }
}

pub fn mailbox_call(channel: u8, mbox: *mut u32) -> bool {
    let addr = (mbox as usize as u32) & !0xF;
    let value = addr | (channel as u32 & 0xF);

    while mmio_read(MBOX_STATUS) & MBOX_FULL != 0 {}

    mmio_write(MBOX_WRITE, value);

    loop {
        while mmio_read(MBOX_STATUS) & MBOX_EMPTY != 0 {}

        let resp = mmio_read(MBOX_READ);
        if resp == value {
            unsafe {
                return core::ptr::read_volatile(mbox.add(1)) == 0x8000_0000;
            }
        }
    }
}