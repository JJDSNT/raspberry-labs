// src/platform/raspi3/peripheral/gpio.rs
//
// Registradores GPIO do BCM2837 (Raspberry Pi 3).
// Necessário para configurar pinos do UART, SD card, USB, etc.
//

pub const BASE: usize = 0x3F20_0000;

// Function select (3 bits por pino, 10 pinos por registrador)
pub const GPFSEL0: usize = BASE + 0x00; // pinos 0-9
pub const GPFSEL1: usize = BASE + 0x04; // pinos 10-19
pub const GPFSEL2: usize = BASE + 0x08; // pinos 20-29
pub const GPFSEL3: usize = BASE + 0x0C; // pinos 30-39
pub const GPFSEL4: usize = BASE + 0x10; // pinos 40-49
pub const GPFSEL5: usize = BASE + 0x14; // pinos 50-53

// Set / Clear
pub const GPSET0: usize = BASE + 0x1C;
pub const GPSET1: usize = BASE + 0x20;
pub const GPCLR0: usize = BASE + 0x28;
pub const GPCLR1: usize = BASE + 0x2C;

// Pull-up/down (legado BCM2835 — ainda usado no Pi 3)
pub const GPPUD:     usize = BASE + 0xE4;
pub const GPPUDCLK0: usize = BASE + 0xF0;
pub const GPPUDCLK1: usize = BASE + 0xF4;

// Valores de function select
pub const FSEL_INPUT:  u32 = 0b000;
pub const FSEL_OUTPUT: u32 = 0b001;
pub const FSEL_ALT0:   u32 = 0b100;
pub const FSEL_ALT1:   u32 = 0b101;
pub const FSEL_ALT2:   u32 = 0b110;
pub const FSEL_ALT3:   u32 = 0b111;
pub const FSEL_ALT4:   u32 = 0b011;
pub const FSEL_ALT5:   u32 = 0b010;

// Valores de pull-up/down
pub const PUD_OFF:  u32 = 0b00;
pub const PUD_DOWN: u32 = 0b01;
pub const PUD_UP:   u32 = 0b10;