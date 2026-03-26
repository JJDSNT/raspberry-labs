use core::arch::asm;

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
        core::ptr::write_volatile(addr as *mut u32, value.to_le());
    }
}

#[inline(always)]
fn mmio_read(addr: usize) -> u32 {
    unsafe { u32::from_le(core::ptr::read_volatile(addr as *const u32)) }
}

/// Barreira de memória completa — garante ordenação de leituras e escritas
/// antes que qualquer operação seguinte seja visível ao hardware.
#[inline(always)]
fn dmb() {
    unsafe {
        asm!("dmb sy", options(nostack, preserves_flags));
    }
}

pub fn mailbox_call(channel: u8, mbox: *mut u32) -> bool {
    // CORRETO: alinha em 16 bytes E converte para bus address do VideoCore
    let addr = ((mbox as usize as u32) & !0xF) | 0xC000_0000;
    let value = addr | (channel as u32 & 0xF);

    dmb(); // ✅ já estava correto

    while mmio_read(MBOX_STATUS) & MBOX_FULL != 0 {}
    mmio_write(MBOX_WRITE, value);

    loop {
        while mmio_read(MBOX_STATUS) & MBOX_EMPTY != 0 {}

        let resp = mmio_read(MBOX_READ);
        if resp == value {
            dmb(); // ✅ já estava correto
            unsafe {
                return u32::from_le(core::ptr::read_volatile(mbox.add(1))) == 0x8000_0000;
            }
        }
    }
}