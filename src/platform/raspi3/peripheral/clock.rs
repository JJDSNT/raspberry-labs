// src/platform/raspi3/peripheral/clock.rs
//
// Clock manager do BCM2837 / Raspberry Pi 3B.
// Camada de acesso ao periférico.
//
// Por enquanto, este módulo cobre apenas o clock do PCM.
//
// Uso típico para áudio PCM 44.1 kHz:
// - PLLD = 500 MHz
// - bit clock desejado = 44_100 * 32 = 1_411_200 Hz
// - divisor ~= 354.33
// - DIVI = 354
// - DIVF = 338
//
// Sequência:
// 1. stop_pcm()
// 2. set_pcm_divider(divi, divf)
// 3. start_pcm_plld_mash1()
// 4. wait_pcm_busy_*()

use core::arch::asm;
use crate::platform::raspi3::mmio;

// ---------------------------------------------------------------------------
// Base MMIO
// ---------------------------------------------------------------------------

const MMIO_BASE: usize = 0x3F00_0000;

// Clock manager — PCM
pub const CM_PCMCTL: usize = MMIO_BASE + 0x0010_1098;
pub const CM_PCMDIV: usize = MMIO_BASE + 0x0010_109C;

// Senha exigida pelo clock manager
pub const CM_PASSWD: u32 = 0x5A00_0000;

// ---------------------------------------------------------------------------
// Bits / campos
// ---------------------------------------------------------------------------

// CTL
pub const CTL_SRC_SHIFT: u32 = 0;
pub const CTL_ENAB: u32      = 1 << 4;
pub const CTL_KILL: u32      = 1 << 5;
pub const CTL_BUSY: u32      = 1 << 7;
pub const CTL_MASH_SHIFT: u32 = 9;

// Sources
pub const SRC_GND: u32  = 0;
pub const SRC_OSC: u32  = 1;
pub const SRC_TESTDBG0: u32 = 2;
pub const SRC_TESTDBG1: u32 = 3;
pub const SRC_PLLA: u32 = 4;
pub const SRC_PLLC: u32 = 5;
pub const SRC_PLLD: u32 = 6;
pub const SRC_HDMI: u32 = 7;

// DIV
pub const DIV_DIVF_MASK: u32 = 0xFFF;
pub const DIV_DIVI_SHIFT: u32 = 12;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

#[inline(always)]
fn delay_nop(n: u32) {
    for _ in 0..n {
        unsafe { asm!("nop", options(nostack, preserves_flags)); }
    }
}

pub fn read_pcmctl() -> u32 {
    mmio::read(CM_PCMCTL)
}

pub fn read_pcmdiv() -> u32 {
    mmio::read(CM_PCMDIV)
}

pub fn pcm_busy() -> bool {
    read_pcmctl() & CTL_BUSY != 0
}

pub fn wait_pcm_busy_set() {
    while !pcm_busy() {
        delay_nop(10);
    }
}

pub fn wait_pcm_busy_clear() {
    while pcm_busy() {
        delay_nop(10);
    }
}

// ---------------------------------------------------------------------------
// API de configuração
// ---------------------------------------------------------------------------

/// Para o clock PCM.
///
/// Mantém o source em PLLD por compatibilidade com o fluxo atual.
pub fn stop_pcm() {
    mmio::write(CM_PCMCTL, CM_PASSWD | SRC_PLLD);
}

/// Programa o divisor do clock PCM.
///
/// - divi: parte inteira (12 bits)
/// - divf: parte fracionária (12 bits)
pub fn set_pcm_divider(divi: u32, divf: u32) {
    let v = ((divi & 0xFFF) << DIV_DIVI_SHIFT) | (divf & DIV_DIVF_MASK);
    mmio::write(CM_PCMDIV, CM_PASSWD | v);
}

/// Liga o clock PCM usando PLLD e MASH=1.
///
/// Equivalente ao fluxo já usado no seu áudio:
/// - source = PLLD (6)
/// - MASH = 1
/// - ENAB = 1
pub fn start_pcm_plld_mash1() {
    let ctl = (1 << CTL_MASH_SHIFT) | CTL_ENAB | SRC_PLLD;
    mmio::write(CM_PCMCTL, CM_PASSWD | ctl);
}

/// Configuração direta do PCM clock.
///
/// `source` usa um dos SRC_*
/// `mash` usa 0..3
pub fn start_pcm(source: u32, mash: u32) {
    let ctl = ((mash & 0x3) << CTL_MASH_SHIFT) | CTL_ENAB | (source & 0xF);
    mmio::write(CM_PCMCTL, CM_PASSWD | ctl);
}

/// Desliga e aguarda BUSY=0.
pub fn stop_pcm_and_wait() {
    stop_pcm();
    wait_pcm_busy_clear();
}

/// Liga e aguarda BUSY=1.
pub fn start_pcm_plld_mash1_and_wait() {
    start_pcm_plld_mash1();
    wait_pcm_busy_set();
}

// ---------------------------------------------------------------------------
// Presets úteis
// ---------------------------------------------------------------------------

/// Preset usado hoje para PCM em 44.1 kHz estéreo com frame de 32 clocks.
///
/// PLLD = 500 MHz
/// divisor ~= 354.33
/// divi = 354
/// divf = 338
pub fn configure_pcm_for_44k1_x_32fs() {
    stop_pcm_and_wait();
    set_pcm_divider(354, 338);
    start_pcm_plld_mash1_and_wait();
}