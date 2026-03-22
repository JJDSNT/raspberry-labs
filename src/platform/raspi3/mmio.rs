// src/platform/raspi3/mmio.rs
//
// Acesso a registradores MMIO do BCM2837.
//
// O hardware BCM2837 e o VideoCore IV são sempre little-endian,
// independentemente do modo do CPU. Os wrappers to_le/from_le garantem
// acesso correto em builds LE (no-op) e BE (swap necessário).

#[inline(always)]
pub fn write(addr: usize, value: u32) {
    unsafe {
        core::ptr::write_volatile(addr as *mut u32, value.to_le());
    }
}

#[inline(always)]
pub fn read(addr: usize) -> u32 {
    unsafe { u32::from_le(core::ptr::read_volatile(addr as *const u32)) }
}
