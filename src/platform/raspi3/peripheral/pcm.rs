// src/platform/raspi3/peripheral/pcm.rs
//
// PCM / I2S do BCM2837 (Raspberry Pi 3B).
// Camada de acesso ao periférico.
//
// Esta camada expõe apenas o hardware PCM.
// Clock e DMA ficam em módulos separados.
//
// Configuração alvo usada no projeto:
// - 44.1 kHz
// - estéreo
// - 16-bit por canal
// - frame de 32 clocks
// - TX via DMA

use crate::platform::raspi3::mmio;

// ---------------------------------------------------------------------------
// Base MMIO
// ---------------------------------------------------------------------------

const MMIO_BASE: usize = 0x3F00_0000;
const PCM_BASE: usize  = MMIO_BASE + 0x0020_3000;

// Registradores
pub const CS_A: usize     = PCM_BASE + 0x00;
pub const FIFO_A: usize   = PCM_BASE + 0x04;
pub const MODE_A: usize   = PCM_BASE + 0x08;
pub const RXC_A: usize    = PCM_BASE + 0x0C;
pub const TXC_A: usize    = PCM_BASE + 0x10;
pub const DREQ_A: usize   = PCM_BASE + 0x14;
pub const INTEN_A: usize  = PCM_BASE + 0x18;
pub const INTSTC_A: usize = PCM_BASE + 0x1C;
pub const GRAY: usize     = PCM_BASE + 0x20;

// Endereço de barramento do FIFO para DMA
pub const FIFO_BUS_ADDR: u32 = 0x7E20_3004;

// ---------------------------------------------------------------------------
// Bits de CS_A
// ---------------------------------------------------------------------------

pub const CS_EN: u32    = 1 << 0;
pub const CS_RXON: u32  = 1 << 1;
pub const CS_TXON: u32  = 1 << 2;
pub const CS_TXCLR: u32 = 1 << 3;
pub const CS_RXCLR: u32 = 1 << 4;
pub const CS_TXTHR: u32 = 1 << 5;
pub const CS_RXTHR: u32 = 1 << 6;
pub const CS_DMAEN: u32 = 1 << 9;

// Status úteis
pub const CS_TXD: u32   = 1 << 19;
pub const CS_RXD: u32   = 1 << 20;
pub const CS_TXERR: u32 = 1 << 15;
pub const CS_RXERR: u32 = 1 << 16;
pub const CS_TXW: u32   = 1 << 17;
pub const CS_RXR: u32   = 1 << 18;

// ---------------------------------------------------------------------------
// Helpers de configuração
// ---------------------------------------------------------------------------

/// MODE_A:
/// - FLEN  = bits [19:10]
/// - FSLEN = bits [9:0]
///
/// Para frame de 32 clocks e FS de 16 clocks:
///   FLEN  = 31
///   FSLEN = 15
#[inline(always)]
pub const fn mode_framing(flen: u32, fslen: u32) -> u32 {
    ((flen & 0x3FF) << 10) | (fslen & 0x3FF)
}

/// TXC_A / RXC_A:
/// Cada canal tem:
/// - EN    : habilita canal
/// - POS   : posição inicial no frame
/// - WIDEX : largura estendida (não usada aqui)
/// - WID   : largura em clocks menos 8? no uso atual do projeto foi usado 8
///
/// Mantemos o formato exato que já estava funcionando no seu código.
///
/// CH1:
/// - EN  = bit 30
/// - POS = bits [29:20]
/// - WID = bits [19:16]
///
/// CH2:
/// - EN  = bit 14
/// - POS = bits [13:4]
/// - WID = bits [3:0]
#[inline(always)]
pub const fn txc_stereo_16(ch1_pos: u32, ch2_pos: u32) -> u32 {
    (1 << 30) | ((ch1_pos & 0x3FF) << 20) | (8 << 16) |
    (1 << 14) | ((ch2_pos & 0x3FF) << 4)  | 8
}

/// DREQ_A:
/// - TX panic threshold: bits [15:8]
/// - TX threshold      : bits [7:0]
#[inline(always)]
pub const fn dreq(tx_panic: u32, tx: u32) -> u32 {
    ((tx_panic & 0xFF) << 8) | (tx & 0xFF)
}

// ---------------------------------------------------------------------------
// API básica
// ---------------------------------------------------------------------------

pub fn disable() {
    mmio::write(CS_A, 0);
}

pub fn enable() {
    mmio::write(CS_A, CS_EN);
}

pub fn clear_tx_fifo() {
    let v = mmio::read(CS_A);
    mmio::write(CS_A, v | CS_TXCLR);
}

pub fn clear_rx_fifo() {
    let v = mmio::read(CS_A);
    mmio::write(CS_A, v | CS_RXCLR);
}

pub fn clear_interrupts() {
    mmio::write(INTSTC_A, 0x0F);
}

pub fn write_mode(mode: u32) {
    mmio::write(MODE_A, mode);
}

pub fn write_txc(txc: u32) {
    mmio::write(TXC_A, txc);
}

pub fn write_rxc(rxc: u32) {
    mmio::write(RXC_A, rxc);
}

pub fn write_dreq(val: u32) {
    mmio::write(DREQ_A, val);
}

pub fn write_fifo(word: u32) {
    mmio::write(FIFO_A, word);
}

pub fn read_cs() -> u32 {
    mmio::read(CS_A)
}

pub fn tx_fifo_can_accept() -> bool {
    read_cs() & CS_TXD != 0
}

pub fn rx_fifo_has_data() -> bool {
    read_cs() & CS_RXD != 0
}

pub fn tx_error() -> bool {
    read_cs() & CS_TXERR != 0
}

pub fn rx_error() -> bool {
    read_cs() & CS_RXERR != 0
}

// ---------------------------------------------------------------------------
// Sequências práticas
// ---------------------------------------------------------------------------

/// Configuração padrão usada pelo projeto:
/// - frame = 32 clocks
/// - FS    = 16 clocks
/// - CH1   = posição 1
/// - CH2   = posição 17
/// - TX panic = 0x10
/// - TX threshold = 0x30
pub fn configure_default_tx() {
    write_mode(mode_framing(31, 15));
    write_txc(txc_stereo_16(1, 17));
    write_dreq(dreq(0x10, 0x30));
    clear_interrupts();
}

/// Liga somente o bloco PCM, limpando TX FIFO.
pub fn enable_tx_block() {
    mmio::write(CS_A, CS_TXCLR | CS_EN);
}

/// Liga TX com DMA já habilitado.
pub fn enable_tx_with_dma() {
    mmio::write(CS_A, CS_DMAEN | CS_TXON | CS_EN);
}