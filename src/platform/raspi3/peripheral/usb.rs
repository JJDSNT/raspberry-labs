// src/platform/raspi3/peripheral/usb.rs
//
// Registradores do controlador USB DWC2 (Synopsys DesignWare USB 2.0 OTG)
// no Raspberry Pi 3 (BCM2837).
//
// Base: 0x3F98_0000
//
// Referências:
//   - Synopsys DWC2 Databook
//   - Linux kernel: drivers/usb/dwc2/
//   - U-Boot: drivers/usb/host/dwc2.c
//

pub const BASE: usize = 0x3F98_0000;

// ---------------------------------------------------------------------------
// Core Global Registers (offset a partir de BASE)
// ---------------------------------------------------------------------------
pub const GOTGCTL:   usize = BASE + 0x000; // OTG Control and Status
pub const GOTGINT:   usize = BASE + 0x004; // OTG Interrupt
pub const GAHBCFG:   usize = BASE + 0x008; // AHB Configuration
pub const GUSBCFG:   usize = BASE + 0x00C; // USB Configuration
pub const GRSTCTL:   usize = BASE + 0x010; // Reset
pub const GINTSTS:   usize = BASE + 0x014; // Interrupt Status
pub const GINTMSK:   usize = BASE + 0x018; // Interrupt Mask
pub const GRXSTSR:   usize = BASE + 0x01C; // Receive Status Debug Read (Host)
pub const GRXSTSP:   usize = BASE + 0x020; // Receive Status Read/Pop (Host)
pub const GRXFSIZ:   usize = BASE + 0x024; // Receive FIFO Size
pub const GNPTXFSIZ: usize = BASE + 0x028; // Non-Periodic Transmit FIFO Size
pub const GNPTXSTS:  usize = BASE + 0x02C; // Non-Periodic Transmit FIFO/Queue Status
pub const GHWCFG1:   usize = BASE + 0x044; // Hardware Configuration 1
pub const GHWCFG2:   usize = BASE + 0x048; // Hardware Configuration 2
pub const GHWCFG3:   usize = BASE + 0x04C; // Hardware Configuration 3
pub const GHWCFG4:   usize = BASE + 0x050; // Hardware Configuration 4
pub const GDFIFOCFG: usize = BASE + 0x05C; // Global DFIFO Configuration
pub const HPTXFSIZ:  usize = BASE + 0x100; // Host Periodic Transmit FIFO Size

// ---------------------------------------------------------------------------
// Host Mode Registers
// ---------------------------------------------------------------------------
pub const HCFG:      usize = BASE + 0x400; // Host Configuration
pub const HFIR:      usize = BASE + 0x404; // Host Frame Interval
pub const HFNUM:     usize = BASE + 0x408; // Host Frame Number / Frame Time Remaining
pub const HPTXSTS:   usize = BASE + 0x410; // Host Periodic Transmit FIFO/Queue Status
pub const HAINT:     usize = BASE + 0x414; // Host All Channels Interrupt
pub const HAINTMSK:  usize = BASE + 0x418; // Host All Channels Interrupt Mask
pub const HPRT:      usize = BASE + 0x440; // Host Port Control and Status

// Host Channel registers — cada canal tem 8 registradores de 32 bits
// Canal N começa em BASE + 0x500 + N * 0x20
pub const HC_BASE:   usize = BASE + 0x500;
pub const HC_SIZE:   usize = 0x20; // tamanho de cada bloco de canal

pub const fn hc_char(n: usize)   -> usize { HC_BASE + n * HC_SIZE + 0x00 }
pub const fn hc_splt(n: usize)   -> usize { HC_BASE + n * HC_SIZE + 0x04 }
pub const fn hc_int(n: usize)    -> usize { HC_BASE + n * HC_SIZE + 0x08 }
pub const fn hc_intmsk(n: usize) -> usize { HC_BASE + n * HC_SIZE + 0x0C }
pub const fn hc_tsiz(n: usize)   -> usize { HC_BASE + n * HC_SIZE + 0x10 }
pub const fn hc_dma(n: usize)    -> usize { HC_BASE + n * HC_SIZE + 0x14 }

// ---------------------------------------------------------------------------
// Bits do GAHBCFG
// ---------------------------------------------------------------------------
pub const GAHBCFG_GLBL_INTR_EN: u32 = 1 << 0;
pub const GAHBCFG_DMA_EN:       u32 = 1 << 5;
pub const GAHBCFG_AHB_SINGLE:   u32 = 1 << 23;

// ---------------------------------------------------------------------------
// Bits do GRSTCTL
// ---------------------------------------------------------------------------
pub const GRSTCTL_CSFTRST:   u32 = 1 << 0;  // Core Soft Reset
pub const GRSTCTL_AHBIDLE:   u32 = 1 << 31; // AHB Master Idle

// ---------------------------------------------------------------------------
// Bits do GINTSTS / GINTMSK
// ---------------------------------------------------------------------------
pub const GINT_SOF:          u32 = 1 << 3;
pub const GINT_RXFLVL:       u32 = 1 << 4;
pub const GINT_NPTXFEMP:     u32 = 1 << 5;
pub const GINT_GINNAKEFF:    u32 = 1 << 6;
pub const GINT_GOUTNAKEFF:   u32 = 1 << 7;
pub const GINT_HPRTINT:      u32 = 1 << 24;
pub const GINT_HCINT:        u32 = 1 << 25;
pub const GINT_PTXFEMP:      u32 = 1 << 26;
pub const GINT_CONIDSTSCHNG: u32 = 1 << 28;
pub const GINT_DISCONNINT:   u32 = 1 << 29;
pub const GINT_SESSREQINT:   u32 = 1 << 30;
pub const GINT_WKUPINT:      u32 = 1 << 31;

// ---------------------------------------------------------------------------
// Número máximo de canais host no BCM2837
// ---------------------------------------------------------------------------
pub const HOST_CHANNEL_COUNT: usize = 8;