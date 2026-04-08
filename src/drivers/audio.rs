// src/drivers/audio.rs
//
// Fachada de áudio do sistema.
//
// A implementação concreta mora na plataforma.
// Por enquanto, Raspberry Pi 3B.

pub use crate::platform::raspi3::audio::{
    BUFFER_SAMPLES,
    SAMPLE_RATE,
    STEREO_WORDS,
};

#[inline]
pub fn init() {
    crate::platform::raspi3::audio::init();
}

#[inline]
pub fn playing_buffer() -> usize {
    crate::platform::raspi3::audio::playing_buffer()
}

#[inline]
pub fn fill_back_buffer(fill: impl FnOnce(&mut [u32])) {
    crate::platform::raspi3::audio::fill_back_buffer(fill);
}

#[inline]
pub fn dma_active() -> bool {
    crate::platform::raspi3::audio::dma_active()
}

#[inline]
pub fn dma_error() -> u32 {
    crate::platform::raspi3::audio::dma_error()
}

#[inline]
pub fn dma_source() -> u32 {
    crate::platform::raspi3::audio::dma_source()
}

#[inline]
pub fn pcm_status() -> u32 {
    crate::platform::raspi3::audio::pcm_status()
}