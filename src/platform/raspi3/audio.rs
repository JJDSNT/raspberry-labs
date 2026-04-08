// src/platform/raspi3/audio.rs
//
// Driver de áudio PCM/I2S para BCM2837 (Raspberry Pi 3B).
//
// Este módulo orquestra:
// - clock PCM
// - periférico PCM/I2S
// - DMA ping-pong
//
// Formato do buffer:
//   [L0, R0, L1, R1, ...]
// cada u32 contém a amostra i16 nos 16 bits inferiores.

use crate::log;
use crate::kernel::sync::IrqSafeSpinLock;
use crate::platform::raspi3::peripheral::{clock, dma, pcm};

// ---------------------------------------------------------------------------
// Configuração
// ---------------------------------------------------------------------------

pub const SAMPLE_RATE: u32 = 44_100;

/// Amostras por canal por buffer (~23 ms em 44.1 kHz)
pub const BUFFER_SAMPLES: usize = 1024;

/// Words u32 por buffer: [L0, R0, L1, R1, ...]
pub const STEREO_WORDS: usize = BUFFER_SAMPLES * 2;

/// Canal DMA usado para áudio
const AUDIO_DMA_CH: usize = 2;

// ---------------------------------------------------------------------------
// Estado estático
// ---------------------------------------------------------------------------

#[repr(align(32))]
struct AlignedBuf([u32; STEREO_WORDS]);

static mut AUDIO_BUFS: [AlignedBuf; 2] = [
    AlignedBuf([0u32; STEREO_WORDS]),
    AlignedBuf([0u32; STEREO_WORDS]),
];

static mut DMA_CBS: [dma::DmaCb; 2] = [
    dma::DmaCb::zeroed(),
    dma::DmaCb::zeroed(),
];

static INITIALIZED: IrqSafeSpinLock<bool> = IrqSafeSpinLock::new(false);

// ---------------------------------------------------------------------------
// Inicialização
// ---------------------------------------------------------------------------

#[allow(static_mut_refs)]
pub fn init() {
    {
        let init = INITIALIZED.lock();
        if *init {
            return;
        }
    }

    log!("AUDIO", "init PCM 44100Hz stereo 16-bit");

    unsafe {
        // 1) Desliga PCM
        pcm::disable();

        // 2) Configura clock PCM para 44.1k * 32fs
        clock::configure_pcm_for_44k1_x_32fs();
        log!("AUDIO", "clock PCM ok");

        // 3) Configura PCM TX estéreo 16-bit
        pcm::configure_default_tx();

        // 4) Buffers começam em silêncio
        for b in 0..2 {
            for w in AUDIO_BUFS[b].0.iter_mut() {
                *w = 0;
            }
        }

        // 5) Control blocks ping-pong
        for i in 0usize..2 {
            let next_i = (i + 1) % 2;
            DMA_CBS[i] = dma::DmaCb {
                ti:     dma::TI_PCM_TX,
                src:    dma::bus_addr(AUDIO_BUFS[i].0.as_ptr()),
                dst:    pcm::FIFO_BUS_ADDR,
                len:    (STEREO_WORDS * 4) as u32,
                stride: 0,
                next:   dma::bus_addr(DMA_CBS.as_ptr().add(next_i)),
                _pad:   [0; 2],
            };
        }
        dma::dsb();

        // 6) Habilita e inicia DMA
        dma::enable_channel(AUDIO_DMA_CH);
        dma::reset_channel(AUDIO_DMA_CH);
        dma::clear_debug(AUDIO_DMA_CH);
        dma::start_channel(AUDIO_DMA_CH, dma::bus_addr(DMA_CBS.as_ptr()));

        // 7) Liga PCM TX com DMA
        pcm::enable_tx_block();
        dma::spin_delay(100);
        pcm::enable_tx_with_dma();

        log!("AUDIO", "DMA PCM TX ativo");
    }

    *INITIALIZED.lock() = true;
}

// ---------------------------------------------------------------------------
// API pública
// ---------------------------------------------------------------------------

#[allow(static_mut_refs)]
/// Buffer que o DMA está atualmente reproduzindo.
pub fn playing_buffer() -> usize {
    let src = dma::read_source_ad(AUDIO_DMA_CH);

    unsafe {
        let b0 = dma::bus_addr(AUDIO_BUFS[0].0.as_ptr());
        let b1 = dma::bus_addr(AUDIO_BUFS[1].0.as_ptr());

        if src.wrapping_sub(b0) < (STEREO_WORDS * 4) as u32 {
            0
        } else if src.wrapping_sub(b1) < (STEREO_WORDS * 4) as u32 {
            1
        } else {
            0
        }
    }
}

/// Preenche o buffer "back" (não em reprodução).
#[allow(static_mut_refs)]
pub fn fill_back_buffer(fill: impl FnOnce(&mut [u32])) {
    let back = 1 - playing_buffer();
    unsafe {
        fill(&mut AUDIO_BUFS[back].0);
    }
    dma::dsb();
}

// ---------------------------------------------------------------------------
// Debug opcional
// ---------------------------------------------------------------------------

pub fn dma_active() -> bool {
    dma::is_active(AUDIO_DMA_CH)
}

pub fn dma_error() -> u32 {
    dma::read_debug(AUDIO_DMA_CH)
}

pub fn dma_source() -> u32 {
    dma::read_source_ad(AUDIO_DMA_CH)
}

pub fn pcm_status() -> u32 {
    pcm::read_cs()
}