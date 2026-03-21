// src/platform/raspi3/peripheral/uart.rs
//
// Registradores do UART PL011 no Raspberry Pi 3.
//

/// Base address do UART0 (PL011)
pub const BASE: usize = 0x3F20_1000;

// Offsets dos registradores
pub const DR:   usize = 0x00; // Data Register
pub const FR:   usize = 0x18; // Flag Register
pub const IBRD: usize = 0x24; // Integer Baud Rate Divisor
pub const FBRD: usize = 0x28; // Fractional Baud Rate Divisor
pub const LCRH: usize = 0x2C; // Line Control Register
pub const CR:   usize = 0x30; // Control Register
pub const IMSC: usize = 0x38; // Interrupt Mask Set/Clear
pub const ICR:  usize = 0x44; // Interrupt Clear Register

// Bits do FR
pub const FR_RXFE: u32 = 1 << 4; // Receive FIFO empty
pub const FR_TXFF: u32 = 1 << 5; // Transmit FIFO full
pub const FR_RXFF: u32 = 1 << 6; // Receive FIFO full
pub const FR_TXFE: u32 = 1 << 7; // Transmit FIFO empty

// Bits do CR
pub const CR_UARTEN: u32 = 1 << 0;  // UART enable
pub const CR_TXE:    u32 = 1 << 8;  // Transmit enable
pub const CR_RXE:    u32 = 1 << 9;  // Receive enable

// Bits do LCRH
pub const LCRH_FEN:  u32 = 1 << 4;  // FIFO enable
pub const LCRH_8BIT: u32 = 0b11 << 5; // 8-bit word length

// Baud rate para 115200 @ 48MHz clock
// IBRD = 26, FBRD = 3
pub const BAUD_IBRD: u32 = 26;
pub const BAUD_FBRD: u32 = 3;