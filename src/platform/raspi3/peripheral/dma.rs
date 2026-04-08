// src/platform/raspi3/peripheral/dma.rs
//
// DMA do BCM2837 / Raspberry Pi 3B.
// Camada de acesso ao periférico (MMIO + control blocks).
//
// Uso:
// - passe endereços em BUS ADDRESS (VideoCore view), não ARM physical
// - cada canal ocupa 0x100 bytes
// - control blocks devem ficar alinhados em 32 bytes

use core::arch::asm;
use crate::platform::raspi3::mmio;

// ---------------------------------------------------------------------------
// Base MMIO
// ---------------------------------------------------------------------------

const MMIO_BASE: usize = 0x3F00_0000;
const DMA_BASE: usize  = MMIO_BASE + 0x0000_7000;

// Registrador global
pub const DMA_ENABLE: usize = DMA_BASE + 0x0FF0;

// ---------------------------------------------------------------------------
// Helpers de endereço por canal
// ---------------------------------------------------------------------------

#[inline(always)]
pub const fn channel_base(ch: usize) -> usize {
    DMA_BASE + ch * 0x100
}

#[inline(always)]
pub const fn cs(ch: usize) -> usize {
    channel_base(ch) + 0x00
}

#[inline(always)]
pub const fn conblk_ad(ch: usize) -> usize {
    channel_base(ch) + 0x04
}

#[inline(always)]
pub const fn ti(ch: usize) -> usize {
    channel_base(ch) + 0x08
}

#[inline(always)]
pub const fn source_ad(ch: usize) -> usize {
    channel_base(ch) + 0x0C
}

#[inline(always)]
pub const fn dest_ad(ch: usize) -> usize {
    channel_base(ch) + 0x10
}

#[inline(always)]
pub const fn txfr_len(ch: usize) -> usize {
    channel_base(ch) + 0x14
}

#[inline(always)]
pub const fn stride(ch: usize) -> usize {
    channel_base(ch) + 0x18
}

#[inline(always)]
pub const fn nextconbk(ch: usize) -> usize {
    channel_base(ch) + 0x1C
}

#[inline(always)]
pub const fn debug(ch: usize) -> usize {
    channel_base(ch) + 0x20
}

// ---------------------------------------------------------------------------
// Bits
// ---------------------------------------------------------------------------

// CS
pub const CS_ACTIVE: u32 = 1 << 0;
pub const CS_END: u32    = 1 << 1;
pub const CS_INT: u32    = 1 << 2;
pub const CS_DREQ: u32   = 1 << 3;
pub const CS_PAUSED: u32 = 1 << 4;
pub const CS_ABORT: u32  = 1 << 30;
pub const CS_RESET: u32  = 1 << 31;

// DEBUG
pub const DEBUG_READ_ERROR: u32              = 1 << 2;
pub const DEBUG_FIFO_ERROR: u32              = 1 << 1;
pub const DEBUG_READ_LAST_NOT_SET_ERROR: u32 = 1 << 0;
pub const DEBUG_CLEAR_ALL: u32               =
    DEBUG_READ_ERROR | DEBUG_FIFO_ERROR | DEBUG_READ_LAST_NOT_SET_ERROR;

// TI
pub const TI_INTEN: u32      = 1 << 0;
pub const TI_TDMODE: u32     = 1 << 1;
pub const TI_WAIT_RESP: u32  = 1 << 3;
pub const TI_DEST_INC: u32   = 1 << 4;
pub const TI_DEST_WIDTH: u32 = 1 << 5;
pub const TI_DEST_DREQ: u32  = 1 << 6;
pub const TI_SRC_INC: u32    = 1 << 8;
pub const TI_SRC_WIDTH: u32  = 1 << 9;
pub const TI_SRC_DREQ: u32   = 1 << 10;
pub const TI_PERMAP_SHIFT: u32 = 16;

// DREQ peripheral map
pub const PERMAP_PCM_TX: u32 = 2;
pub const PERMAP_PCM_RX: u32 = 3;

// TI pronto para PCM TX
pub const TI_PCM_TX: u32 =
    TI_SRC_INC |
    TI_DEST_DREQ |
    TI_WAIT_RESP |
    (PERMAP_PCM_TX << TI_PERMAP_SHIFT);

// ---------------------------------------------------------------------------
// Control Block
// ---------------------------------------------------------------------------

#[repr(C, align(32))]
#[derive(Clone, Copy)]
pub struct DmaCb {
    pub ti:     u32,
    pub src:    u32,
    pub dst:    u32,
    pub len:    u32,
    pub stride: u32,
    pub next:   u32,
    pub _pad:   [u32; 2],
}

impl DmaCb {
    pub const fn zeroed() -> Self {
        Self {
            ti: 0,
            src: 0,
            dst: 0,
            len: 0,
            stride: 0,
            next: 0,
            _pad: [0; 2],
        }
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// ARM physical -> bus address para DMA (uncached VC alias)
#[inline(always)]
pub fn bus_addr<T>(ptr: *const T) -> u32 {
    (ptr as usize as u32) | 0xC000_0000
}

#[inline(always)]
pub fn dsb() {
    unsafe { asm!("dsb sy", options(nostack, preserves_flags)); }
}

#[inline(always)]
pub fn spin_delay(n: u32) {
    for _ in 0..n {
        unsafe { asm!("nop", options(nostack, preserves_flags)); }
    }
}

// ---------------------------------------------------------------------------
// API de canal
// ---------------------------------------------------------------------------

pub fn enable_channel(ch: usize) {
    let v = mmio::read(DMA_ENABLE);
    mmio::write(DMA_ENABLE, v | (1 << ch));
}

pub fn disable_channel(ch: usize) {
    let v = mmio::read(DMA_ENABLE);
    mmio::write(DMA_ENABLE, v & !(1 << ch));
}

pub fn reset_channel(ch: usize) {
    mmio::write(cs(ch), CS_RESET);
    spin_delay(100);
}

pub fn abort_channel(ch: usize) {
    mmio::write(cs(ch), CS_ABORT);
    spin_delay(100);
}

pub fn clear_debug(ch: usize) {
    mmio::write(debug(ch), DEBUG_CLEAR_ALL);
}

pub fn start_channel(ch: usize, cb_bus_addr: u32) {
    clear_debug(ch);
    mmio::write(conblk_ad(ch), cb_bus_addr);
    mmio::write(cs(ch), CS_ACTIVE);
}

pub fn stop_channel(ch: usize) {
    mmio::write(cs(ch), 0);
}

pub fn read_cs(ch: usize) -> u32 {
    mmio::read(cs(ch))
}

pub fn read_conblk_ad(ch: usize) -> u32 {
    mmio::read(conblk_ad(ch))
}

pub fn read_source_ad(ch: usize) -> u32 {
    mmio::read(source_ad(ch))
}

pub fn read_dest_ad(ch: usize) -> u32 {
    mmio::read(dest_ad(ch))
}

pub fn read_txfr_len(ch: usize) -> u32 {
    mmio::read(txfr_len(ch))
}

pub fn read_nextconbk(ch: usize) -> u32 {
    mmio::read(nextconbk(ch))
}

pub fn read_debug(ch: usize) -> u32 {
    mmio::read(debug(ch))
}

pub fn is_active(ch: usize) -> bool {
    read_cs(ch) & CS_ACTIVE != 0
}

pub fn has_error(ch: usize) -> bool {
    read_debug(ch) & DEBUG_CLEAR_ALL != 0
}