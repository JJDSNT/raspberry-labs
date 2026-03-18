const UART0_BASE: usize = 0x3F201000;

const UART_DR: usize = UART0_BASE + 0x00;
const UART_FR: usize = UART0_BASE + 0x18;
const UART_IBRD: usize = UART0_BASE + 0x24;
const UART_FBRD: usize = UART0_BASE + 0x28;
const UART_LCRH: usize = UART0_BASE + 0x2C;
const UART_CR: usize = UART0_BASE + 0x30;
const UART_ICR: usize = UART0_BASE + 0x44;

const FR_TXFF: u32 = 1 << 5;

#[inline(always)]
fn mmio_write(addr: usize, value: u32) {
    unsafe { core::ptr::write_volatile(addr as *mut u32, value) }
}

#[inline(always)]
fn mmio_read(addr: usize) -> u32 {
    unsafe { core::ptr::read_volatile(addr as *const u32) }
}

pub fn uart_init() {
    mmio_write(UART_CR, 0x0000);      // disable UART
    mmio_write(UART_ICR, 0x07FF);     // clear interrupts

    // divisores típicos para 115200 com UART clock de 48 MHz
    mmio_write(UART_IBRD, 26);
    mmio_write(UART_FBRD, 3);

    // 8N1 + FIFO habilitado
    mmio_write(UART_LCRH, (1 << 4) | (3 << 5));

    // UART, TX e RX enable
    mmio_write(UART_CR, (1 << 0) | (1 << 8) | (1 << 9));
}

pub fn uart_putc(c: u8) {
    while mmio_read(UART_FR) & FR_TXFF != 0 {}
    mmio_write(UART_DR, c as u32);
}

pub fn uart_write_str(s: &str) {
    for b in s.bytes() {
        if b == b'\n' {
            uart_putc(b'\r');
        }
        uart_putc(b);
    }
}