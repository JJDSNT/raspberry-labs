// src/platform/raspi3/interrupts.rs

use crate::platform::raspi3::mmio;

// ---------------------------------------------------------------------------
// ARM Local Peripherals — timers e IRQs por-core
// Base: 0x4000_0000
// ---------------------------------------------------------------------------
const LOCAL_PERIPH_BASE: usize = 0x4000_0000;

const LOCAL_TIMER_INT_CONTROL0: usize = LOCAL_PERIPH_BASE + 0x40;
const LOCAL_IRQ_PENDING0:       usize = LOCAL_PERIPH_BASE + 0x60;

pub const CNTPNSIRQ_BIT: u32 = 1 << 1;
pub const CNTPSIRQ_BIT:  u32 = 1 << 0;
pub const CNTHPIRQ_BIT:  u32 = 1 << 2;
pub const CNTVIRQ_BIT:   u32 = 1 << 3;

// Bit 8 do IRQ pending local indica que há IRQ pendente no VIC (GPU IRQs)
pub const LOCAL_IRQ_GPU_BIT: u32 = 1 << 8;

pub fn enable_core0_cntpnsirq() {
    let val = mmio::read(LOCAL_TIMER_INT_CONTROL0);
    mmio::write(LOCAL_TIMER_INT_CONTROL0, val | CNTPNSIRQ_BIT);
}

pub fn disable_core0_cntpnsirq() {
    let val = mmio::read(LOCAL_TIMER_INT_CONTROL0);
    mmio::write(LOCAL_TIMER_INT_CONTROL0, val & !CNTPNSIRQ_BIT);
}

#[inline(always)]
pub fn core0_timer_int_control() -> u32 {
    mmio::read(LOCAL_TIMER_INT_CONTROL0)
}

#[inline(always)]
pub fn core0_irq_pending() -> u32 {
    mmio::read(LOCAL_IRQ_PENDING0)
}

#[inline(always)]
pub fn core0_has_local_irq_pending() -> bool {
    (core0_irq_pending() & 0x7ff) != 0
}

pub fn init_core0_timer_irq() {
    enable_core0_cntpnsirq();
}

// ---------------------------------------------------------------------------
// VIC — VideoCore Interrupt Controller (GPU IRQs)
// Base: 0x3F00_B200
//
// O DWC2 (USB) usa IRQ 9 do VIC (IRQ pending 1, bit 9).
// ---------------------------------------------------------------------------
const VIC_BASE: usize = 0x3F00_B200;

const VIC_IRQ_BASIC_PENDING: usize = VIC_BASE + 0x00;
const VIC_IRQ_PENDING1:      usize = VIC_BASE + 0x04;
const VIC_IRQ_PENDING2:      usize = VIC_BASE + 0x08;
const VIC_ENABLE_IRQS1:      usize = VIC_BASE + 0x10;
const VIC_ENABLE_IRQS2:      usize = VIC_BASE + 0x14;
const VIC_ENABLE_BASIC_IRQS: usize = VIC_BASE + 0x18;
const VIC_DISABLE_IRQS1:     usize = VIC_BASE + 0x1C;
const VIC_DISABLE_IRQS2:     usize = VIC_BASE + 0x20;

// IRQ 9 = USB HCD (DWC2) — bit 9 do pending 1
pub const VIC_USB_IRQ_BIT: u32 = 1 << 9;

/// Habilita a IRQ do USB (IRQ 9) no VIC.
pub fn vic_enable_usb_irq() {
    mmio::write(VIC_ENABLE_IRQS1, VIC_USB_IRQ_BIT);
}

/// Desabilita a IRQ do USB no VIC.
pub fn vic_disable_usb_irq() {
    mmio::write(VIC_DISABLE_IRQS1, VIC_USB_IRQ_BIT);
}

/// Verifica se a IRQ do USB está pendente no VIC.
#[inline(always)]
pub fn vic_usb_irq_pending() -> bool {
    (mmio::read(VIC_IRQ_PENDING1) & VIC_USB_IRQ_BIT) != 0
}

/// Verifica se há qualquer IRQ pendente no VIC.
#[inline(always)]
pub fn vic_any_pending() -> bool {
    mmio::read(VIC_IRQ_BASIC_PENDING) != 0
        || mmio::read(VIC_IRQ_PENDING1) != 0
        || mmio::read(VIC_IRQ_PENDING2) != 0
}