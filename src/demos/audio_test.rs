// src/demos/audio_test.rs
//
// Demo de teste de áudio: toca tons via PCM e exibe um
// osciloscópio em tempo real com a forma de onda gerada.

use crate::audio::mixer::Mixer;
use crate::demos::Demo;
use crate::drivers::audio::{self, BUFFER_SAMPLES, SAMPLE_RATE, STEREO_WORDS};
use crate::gfx::renderer::Renderer;
use crate::media::FrameContext;

// ---------------------------------------------------------------------------
// Sequência de notas (freq Hz, duração em frames @ 60 fps)
// ---------------------------------------------------------------------------

const NOTES: &[(u32, u32)] = &[
    (440, 90),
    (494, 60),
    (523, 60),
    (587, 60),
    (659, 90),
    (698, 60),
    (784, 90),
    (880, 120),
    (0,   30),
];

const AMPLITUDE: i16 = 24_000;
const SCOPE_SAMPLES: usize = 512;

// ---------------------------------------------------------------------------
// Estado do demo
// ---------------------------------------------------------------------------

pub struct AudioTestDemo {
    mixer:        Mixer,
    note_idx:     usize,
    note_frames:  u32,
    frame_count:  u64,
    initialized:  bool,

    scope_buf:    [i16; SCOPE_SAMPLES],
    scope_head:   usize,

    audio_tmp:    [u32; STEREO_WORDS],

    // Último buffer observado em reprodução pelo DMA
    last_playing_buf: usize,
}

impl AudioTestDemo {
    pub fn new() -> Self {
        Self {
            mixer:            Mixer::new(SAMPLE_RATE),
            note_idx:         0,
            note_frames:      NOTES[0].1,
            frame_count:      0,
            initialized:      false,
            scope_buf:        [0; SCOPE_SAMPLES],
            scope_head:       0,
            audio_tmp:        [0; STEREO_WORDS],
            last_playing_buf: usize::MAX, // força sincronização inicial
        }
    }

    fn next_note(&mut self) {
        self.note_idx = (self.note_idx + 1) % NOTES.len();
        let (freq, dur) = NOTES[self.note_idx];
        self.note_frames = dur;
        self.mixer.play_tone(freq, AMPLITUDE, 0);
    }

    fn generate_audio(&mut self) {
        self.mixer.generate(&mut self.audio_tmp);

        // Captura canal esquerdo para o osciloscópio
        for i in 0..BUFFER_SAMPLES {
            let raw = self.audio_tmp[i * 2] as i16;
            self.scope_buf[self.scope_head] = raw;
            self.scope_head = (self.scope_head + 1) % SCOPE_SAMPLES;
        }

        let src = &self.audio_tmp;
        audio::fill_back_buffer(|dst| {
            dst.copy_from_slice(src);
        });
    }

    /// Sincroniza a produção de áudio com a troca de buffer do DMA.
    fn sync_audio(&mut self) {
        loop {
            let current = audio::playing_buffer();

            if current == self.last_playing_buf {
                break;
            }

            self.last_playing_buf = current;
            self.generate_audio();
        }
    }

    fn draw_scope(&self, renderer: &mut Renderer) {
        let w = renderer.width() as i32;
        let h = renderer.height() as i32;

        let scope_x0 = 40;
        let scope_x1 = w - 40;
        let scope_y = h / 2;
        let scope_amp = (h / 4).max(1);

        renderer.draw_line(scope_x0, scope_y, scope_x1, scope_y, 0xFF_30_30_30);

        renderer.draw_line(scope_x0, scope_y - scope_amp, scope_x1, scope_y - scope_amp, 0xFF_20_50_20);
        renderer.draw_line(scope_x0, scope_y + scope_amp, scope_x1, scope_y + scope_amp, 0xFF_20_50_20);

        let width = (scope_x1 - scope_x0).max(1);
        let n = width.min(SCOPE_SAMPLES as i32) as usize;

        let mut prev_x = scope_x0;
        let mut prev_y = scope_y;

        for px in 0..n {
            let idx = (self.scope_head + SCOPE_SAMPLES - n + px) % SCOPE_SAMPLES;
            let sample = self.scope_buf[idx];

            let x = scope_x0 + (px as i32 * width / n as i32);
            let y = scope_y - (sample as i32 * scope_amp / 32767);
            let y = y.clamp(scope_y - scope_amp, scope_y + scope_amp);

            if px > 0 {
                renderer.draw_line(prev_x, prev_y, x, y, 0xFF_00_FF_44);
            }

            prev_x = x;
            prev_y = y;
        }
    }

    fn draw_ui(&self, renderer: &mut Renderer) {
        let w = renderer.width() as i32;
        let h = renderer.height() as i32;
        let (freq, _) = NOTES[self.note_idx];

        renderer.draw_str(8, 8, "AUDIO TEST — PCM/I2S", 0xFF_FF_FF_00, 0xFF_00_00_00);
        renderer.draw_str(8, 20, "44100 Hz  Stereo  16-bit", 0xFF_80_80_80, 0xFF_00_00_00);

        let mut note_label = [0u8; 32];
        let label = fmt_str(&mut note_label, "Nota: ", note_name(freq), freq);
        renderer.draw_str(8, h as usize - 32, label, 0xFF_FF_AA_00, 0xFF_00_00_00);

        let mut cnt_buf = [0u8; 20];
        let cnt = fmt_u64(&mut cnt_buf, self.frame_count);
        renderer.draw_str(
            w as usize - 8 - cnt.len() * 8,
            8,
            cnt,
            0xFF_60_60_60,
            0xFF_00_00_00,
        );

        let vu_h = (h / 2) as usize;
        let vu_y = (h / 4) as usize;

        let last_s = self.scope_buf[(self.scope_head + SCOPE_SAMPLES - 1) % SCOPE_SAMPLES];
        let vu_fill = ((last_s.unsigned_abs() as usize) * vu_h / 32767).min(vu_h);

        let vu_color = if vu_fill > vu_h * 3 / 4 {
            0xFF_FF_30_30
        } else if vu_fill > vu_h / 2 {
            0xFF_FF_AA_00
        } else {
            0xFF_00_CC_44
        };

        renderer.fill_rect(16, vu_y + vu_h - vu_fill, 12, vu_fill, vu_color);
        renderer.fill_rect(16, vu_y, 12, vu_h - vu_fill, 0xFF_10_20_10);

        renderer.fill_rect(w as usize - 28, vu_y + vu_h - vu_fill, 12, vu_fill, vu_color);
        renderer.fill_rect(w as usize - 28, vu_y, 12, vu_h - vu_fill, 0xFF_10_20_10);
    }
}

impl Demo for AudioTestDemo {
    fn render(&mut self, renderer: &mut Renderer, _frame: &FrameContext) {
        if !self.initialized {
            audio::init();

            let (freq, dur) = NOTES[0];
            self.note_frames = dur;
            self.mixer.play_tone(freq, AMPLITUDE, 0);

            // Pré-carrega os dois buffers para o DMA começar suave
            self.generate_audio();
            self.generate_audio();

            self.initialized = true;
        }

        self.sync_audio();

        if self.note_frames > 0 {
            self.note_frames -= 1;
        } else {
            self.next_note();
        }

        renderer.clear_black();
        self.draw_scope(renderer);
        self.draw_ui(renderer);

        self.frame_count = self.frame_count.wrapping_add(1);
    }
}

// ---------------------------------------------------------------------------
// Utilitários de formatação sem alloc
// ---------------------------------------------------------------------------

fn note_name(freq: u32) -> &'static str {
    match freq {
        440 => "La4",
        494 => "Si4",
        523 => "Do5",
        587 => "Re5",
        659 => "Mi5",
        698 => "Fa5",
        784 => "Sol5",
        880 => "La5",
        0   => "---",
        _   => "???",
    }
}

fn fmt_str<'a>(buf: &'a mut [u8], prefix: &str, name: &str, freq: u32) -> &'a str {
    let mut pos = 0;
    for b in prefix.bytes() {
        if pos < buf.len() { buf[pos] = b; pos += 1; }
    }
    for b in name.bytes() {
        if pos < buf.len() { buf[pos] = b; pos += 1; }
    }
    if pos < buf.len() { buf[pos] = b' '; pos += 1; }
    if freq > 0 {
        let mut tmp = [0u8; 10];
        let s = fmt_u32(&mut tmp, freq);
        for b in s.bytes() {
            if pos < buf.len() { buf[pos] = b; pos += 1; }
        }
        for b in b" Hz".iter() {
            if pos < buf.len() { buf[pos] = *b; pos += 1; }
        }
    }
    core::str::from_utf8(&buf[..pos]).unwrap_or("?")
}

fn fmt_u32(buf: &mut [u8], mut n: u32) -> &str {
    if n == 0 {
        buf[0] = b'0';
        return core::str::from_utf8(&buf[..1]).unwrap_or("0");
    }
    let mut end = buf.len();
    while n > 0 && end > 0 {
        end -= 1;
        buf[end] = b'0' + (n % 10) as u8;
        n /= 10;
    }
    let start = end;
    let total = buf.len() - start;
    buf.copy_within(start..start + total, 0);
    core::str::from_utf8(&buf[..total]).unwrap_or("?")
}

fn fmt_u64(buf: &mut [u8], mut n: u64) -> &str {
    if n == 0 {
        buf[0] = b'0';
        return core::str::from_utf8(&buf[..1]).unwrap_or("0");
    }
    let mut end = buf.len();
    while n > 0 && end > 0 {
        end -= 1;
        buf[end] = b'0' + (n % 10) as u8;
        n /= 10;
    }
    let start = end;
    let total = buf.len() - start;
    buf.copy_within(start..start + total, 0);
    core::str::from_utf8(&buf[..total]).unwrap_or("?")
}