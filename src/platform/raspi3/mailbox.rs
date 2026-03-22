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
    // FIX: máscara 0x3FFF_FFFF converte para endereço de barramento do GPU.
    // A máscara original (!0xF) apenas alinhava em 16 bytes mas mantinha
    // bits altos que o GPU não consegue endereçar, corrompendo a mensagem.
    let addr = (mbox as usize as u32) & 0x3FFF_FFFF;
    let value = addr | (channel as u32 & 0xF);

    // FIX: barreira antes de enviar — garante que todas as escritas no buffer
    // feitas pelo framebuffer.rs sejam visíveis ao GPU antes do envio.
    dmb();

    while mmio_read(MBOX_STATUS) & MBOX_FULL != 0 {}

    mmio_write(MBOX_WRITE, value);

    loop {
        while mmio_read(MBOX_STATUS) & MBOX_EMPTY != 0 {}

        let resp = mmio_read(MBOX_READ);
        if resp == value {
            // FIX: barreira após receber resposta — garante que a leitura
            // do buffer a seguir enxerga os dados escritos pelo GPU.
            dmb();

            unsafe {
                return u32::from_le(core::ptr::read_volatile(mbox.add(1))) == 0x8000_0000;
            }
        }
    }
}