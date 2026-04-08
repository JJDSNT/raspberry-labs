// src/audio/mixer.rs
//
// Misturador de tons via síntese digital (tabela de seno, fase Q16).
// Suporta N_VOICES vozes simultâneas. Cada voz é um oscilador de onda senoidal.
//
// Formato de saída: slice [u32] com `STEREO_WORDS` entradas,
//   [L0, R0, L1, R1, ...], cada u32 contém a amostra i16 nos 16 bits
//   inferiores (compatível com o driver PCM em modo não-packed).

// ---------------------------------------------------------------------------
// Tabela de seno: 256 entradas, sin(i × 2π/256) × 32767, valores i16
// ---------------------------------------------------------------------------
#[rustfmt::skip]
static SIN_TABLE: [i16; 256] = [
       0,   804,  1608,  2410,  3212,  4011,  4808,  5602,
    6393,  7179,  7962,  8739,  9512, 10278, 11039, 11793,
   12539, 13279, 14010, 14732, 15446, 16151, 16846, 17530,
   18204, 18868, 19519, 20159, 20787, 21403, 22005, 22594,
   23170, 23731, 24279, 24811, 25329, 25832, 26319, 26790,
   27245, 27683, 28105, 28510, 28898, 29268, 29621, 29956,
   30273, 30571, 30852, 31113, 31356, 31580, 31785, 31971,
   32137, 32285, 32412, 32521, 32610, 32679, 32728, 32758,
   32767, 32758, 32728, 32679, 32610, 32521, 32412, 32285,
   32137, 31971, 31785, 31580, 31356, 31113, 30852, 30571,
   30273, 29956, 29621, 29268, 28898, 28510, 28105, 27683,
   27245, 26790, 26319, 25832, 25329, 24811, 24279, 23731,
   23170, 22594, 22005, 21403, 20787, 20159, 19519, 18868,
   18204, 17530, 16846, 16151, 15446, 14732, 14010, 13279,
   12539, 11793, 11039, 10278,  9512,  8739,  7962,  7179,
    6393,  5602,  4808,  4011,  3212,  2410,  1608,   804,
       0,  -804, -1608, -2410, -3212, -4011, -4808, -5602,
   -6393, -7179, -7962, -8739, -9512,-10278,-11039,-11793,
  -12539,-13279,-14010,-14732,-15446,-16151,-16846,-17530,
  -18204,-18868,-19519,-20159,-20787,-21403,-22005,-22594,
  -23170,-23731,-24279,-24811,-25329,-25832,-26319,-26790,
  -27245,-27683,-28105,-28510,-28898,-29268,-29621,-29956,
  -30273,-30571,-30852,-31113,-31356,-31580,-31785,-31971,
  -32137,-32285,-32412,-32521,-32610,-32679,-32728,-32758,
  -32767,-32758,-32728,-32679,-32610,-32521,-32412,-32285,
  -32137,-31971,-31785,-31580,-31356,-31113,-30852,-30571,
  -30273,-29956,-29621,-29268,-28898,-28510,-28105,-27683,
  -27245,-26790,-26319,-25832,-25329,-24811,-24279,-23731,
  -23170,-22594,-22005,-21403,-20787,-20159,-19519,-18868,
  -18204,-17530,-16846,-16151,-15446,-14732,-14010,-13279,
  -12539,-11793,-11039,-10278, -9512, -8739, -7962, -7179,
   -6393, -5602, -4808, -4011, -3212, -2410, -1608,  -804,
];

#[inline(always)]
fn sin_q16(phase: u32) -> i16 {
    SIN_TABLE[((phase >> 8) & 0xFF) as usize]
}

// ---------------------------------------------------------------------------
// Voz individual
// ---------------------------------------------------------------------------

pub const N_VOICES: usize = 4;

#[derive(Clone, Copy)]
struct Voice {
    phase:     u32,  // fase atual Q16
    step:      u32,  // incremento por amostra = freq × 65536 / sample_rate
    amplitude: i16,  // amplitude 0..32767
    active:    bool,
}

impl Voice {
    const fn silent() -> Self {
        Voice { phase: 0, step: 0, amplitude: 0, active: false }
    }
}

// ---------------------------------------------------------------------------
// Mixer
// ---------------------------------------------------------------------------

pub struct Mixer {
    voices:      [Voice; N_VOICES],
    sample_rate: u32,
}

impl Mixer {
    pub const fn new(sample_rate: u32) -> Self {
        Mixer {
            voices: [Voice::silent(); N_VOICES],
            sample_rate,
        }
    }

    /// Inicia/atualiza uma voz (`voice` < N_VOICES).
    /// `freq_hz=0` → silencia a voz.
    pub fn play_tone(&mut self, freq_hz: u32, amplitude: i16, voice: usize) {
        if voice >= N_VOICES { return; }
        let step = if freq_hz > 0 {
            (freq_hz * 65536) / self.sample_rate
        } else {
            0
        };
        self.voices[voice].step      = step;
        self.voices[voice].amplitude = amplitude;
        self.voices[voice].active    = step > 0;
        // Preserva fase para evitar clique ao trocar de frequência
    }

    /// Silencia todas as vozes.
    #[allow(dead_code)]
    pub fn silence(&mut self) {
        for v in self.voices.iter_mut() {
            v.active = false;
            v.step   = 0;
        }
    }

    /// Preenche o buffer PCM com o mix das vozes ativas.
    ///
    /// Formato: `buf[2i]` = amostra Left i, `buf[2i+1]` = amostra Right i.
    /// Cada entry é u32 com i16 nos 16 bits inferiores.
    /// O slice deve ter `STEREO_WORDS` entradas (= BUFFER_SAMPLES × 2).
    pub fn generate(&mut self, buf: &mut [u32]) {
        let n_samples = buf.len() / 2;

        // Copia fases localmente para avançar sem afetar state das vozes ainda
        let mut phases = [0u32; N_VOICES];
        for i in 0..N_VOICES {
            phases[i] = self.voices[i].phase;
        }

        for i in 0..n_samples {
            let mut acc: i32 = 0;
            for vi in 0..N_VOICES {
                if self.voices[vi].active {
                    let s   = sin_q16(phases[vi]);
                    acc    += (s as i32 * self.voices[vi].amplitude as i32) >> 15;
                    phases[vi] = phases[vi].wrapping_add(self.voices[vi].step);
                }
            }
            // Clipa em i16 e grava L e R separadamente
            let s = acc.clamp(-32767, 32767) as i16 as u16 as u32;
            buf[i * 2]     = s; // CH1 (Left)  — nos 16 bits inferiores
            buf[i * 2 + 1] = s; // CH2 (Right) — nos 16 bits inferiores
        }

        // Persiste as fases atualizadas
        for i in 0..N_VOICES {
            self.voices[i].phase = phases[i];
        }
    }

    /// Amostra instantânea da voz 0 (para visualização de osciloscópio).
    #[allow(dead_code)]
    pub fn peek_sample(&self) -> i16 {
        let v = &self.voices[0];
        if !v.active { return 0; }
        let s = sin_q16(v.phase);
        ((s as i32 * v.amplitude as i32) >> 15) as i16
    }
}
