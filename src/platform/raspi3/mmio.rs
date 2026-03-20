// src/platform/raspi3/mmio.rs

#[inline(always)]
pub fn write(addr: usize, value: u32) {
    unsafe {
        core::ptr::write_volatile(addr as *mut u32, value);
    }
}

#[inline(always)]
pub fn read(addr: usize) -> u32 {
    unsafe { core::ptr::read_volatile(addr as *const u32) }
}