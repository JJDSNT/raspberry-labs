// src/media/clock.rs
//
// MediaClock
// ----------
// Clock global de sincronização para render, animação, demos e, no futuro,
// apoio à sincronização com áudio.
//
// Responsabilidades:
// - ler tempo monotônico bruto do kernel (ticks)
// - calcular delta por frame
// - manter acumulador para fixed timestep
// - expor alpha para interpolação
//
// Não depende de gfx nem de audio.
// É uma camada acima de kernel/time.rs.
//

use crate::kernel::time;

#[derive(Clone, Copy, Debug)]
pub struct FrameContext {
    pub frame: u64,
    pub now_ticks: u64,
    pub total_ticks: u64,
    pub frame_dt_ticks: u64,
    pub frame_dt_secs: f32,
    pub alpha: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct FixedStepContext {
    pub step: u64,
    pub dt_ticks: u64,
    pub dt_secs: f32,
    pub total_ticks: u64,
    pub total_secs: f32,
}

pub struct MediaClock {
    ticks_per_second: u64,
    fixed_step_ticks: u64,
    max_frame_ticks: u64,

    started: bool,
    last_tick: u64,
    now_tick: u64,

    frame_count: u64,
    step_count: u64,

    total_ticks: u64,
    frame_dt_ticks: u64,
    accumulator_ticks: u64,
}

impl MediaClock {
    /// Cria um clock com:
    /// - `ticks_per_second`: frequência do clock monotônico
    /// - `fixed_hz`: frequência do update fixo (ex.: 60, 120)
    ///
    /// O clamp de frame é configurado para 250ms por padrão.
    pub const fn new(ticks_per_second: u64, fixed_hz: u64) -> Self {
        Self::with_max_frame_seconds(ticks_per_second, fixed_hz, 0.250)
    }

    /// Igual a `new`, mas permite definir o clamp máximo de delta por frame.
    ///
    /// Exemplo:
    /// - `max_frame_seconds = 0.050`  => clamp em 50ms
    /// - `max_frame_seconds = 0.250`  => clamp em 250ms
    pub const fn with_max_frame_seconds(
        ticks_per_second: u64,
        fixed_hz: u64,
        max_frame_seconds: f32,
    ) -> Self {
        let fixed_step_ticks = if fixed_hz == 0 {
            1
        } else {
            let ticks = ticks_per_second / fixed_hz;
            if ticks == 0 { 1 } else { ticks }
        };

        let max_frame_ticks = seconds_to_ticks_const(ticks_per_second, max_frame_seconds);

        Self {
            ticks_per_second,
            fixed_step_ticks,
            max_frame_ticks: if max_frame_ticks == 0 { 1 } else { max_frame_ticks },

            started: false,
            last_tick: 0,
            now_tick: 0,

            frame_count: 0,
            step_count: 0,

            total_ticks: 0,
            frame_dt_ticks: 0,
            accumulator_ticks: 0,
        }
    }

    /// Reinicia o clock usando o tick atual do sistema.
    pub fn reset(&mut self) {
        let now = time::ticks();

        self.started = true;
        self.last_tick = now;
        self.now_tick = now;

        self.frame_count = 0;
        self.step_count = 0;
        self.total_ticks = 0;
        self.frame_dt_ticks = 0;
        self.accumulator_ticks = 0;
    }

    /// Inicia o clock, se ainda não tiver sido iniciado.
    pub fn start_if_needed(&mut self) {
        if !self.started {
            self.reset();
        }
    }

    /// Começa um novo frame:
    /// - lê o tick atual
    /// - calcula delta
    /// - aplica clamp
    /// - acumula tempo para fixed timestep
    ///
    /// Retorna o contexto do frame atual.
    pub fn begin_frame(&mut self) -> FrameContext {
        self.start_if_needed();

        let now = time::ticks();
        let raw_dt = now.wrapping_sub(self.last_tick);
        let clamped_dt = raw_dt.min(self.max_frame_ticks);

        self.last_tick = now;
        self.now_tick = now;
        self.frame_dt_ticks = clamped_dt;
        self.total_ticks = self.total_ticks.wrapping_add(clamped_dt);
        self.accumulator_ticks = self.accumulator_ticks.wrapping_add(clamped_dt);
        self.frame_count = self.frame_count.wrapping_add(1);

        FrameContext {
            frame: self.frame_count,
            now_ticks: self.now_tick,
            total_ticks: self.total_ticks,
            frame_dt_ticks: self.frame_dt_ticks,
            frame_dt_secs: self.ticks_to_secs_f32(self.frame_dt_ticks),
            alpha: self.alpha(),
        }
    }

    /// Retorna `true` enquanto ainda houver update fixo pendente.
    #[inline]
    pub fn has_fixed_step(&self) -> bool {
        self.accumulator_ticks >= self.fixed_step_ticks
    }

    /// Consome um passo fixo, se houver.
    ///
    /// Uso típico:
    ///
    /// while let Some(step) = clock.next_fixed_step() {
    ///     demo.update(step.dt_secs);
    /// }
    pub fn next_fixed_step(&mut self) -> Option<FixedStepContext> {
        if self.accumulator_ticks < self.fixed_step_ticks {
            return None;
        }

        self.accumulator_ticks -= self.fixed_step_ticks;
        self.step_count = self.step_count.wrapping_add(1);

        Some(FixedStepContext {
            step: self.step_count,
            dt_ticks: self.fixed_step_ticks,
            dt_secs: self.fixed_dt_secs(),
            total_ticks: self.total_ticks,
            total_secs: self.total_secs(),
        })
    }

    /// Fator de interpolação entre o último update fixo e o próximo.
    ///
    /// Valor esperado: [0.0, 1.0)
    #[inline]
    pub fn alpha(&self) -> f32 {
        if self.fixed_step_ticks == 0 {
            0.0
        } else {
            self.accumulator_ticks as f32 / self.fixed_step_ticks as f32
        }
    }

    #[inline]
    pub const fn ticks_per_second(&self) -> u64 {
        self.ticks_per_second
    }

    #[inline]
    pub const fn fixed_step_ticks(&self) -> u64 {
        self.fixed_step_ticks
    }

    #[inline]
    pub fn fixed_dt_secs(&self) -> f32 {
        self.ticks_to_secs_f32(self.fixed_step_ticks)
    }

    #[inline]
    pub const fn frame_count(&self) -> u64 {
        self.frame_count
    }

    #[inline]
    pub const fn step_count(&self) -> u64 {
        self.step_count
    }

    #[inline]
    pub const fn total_ticks(&self) -> u64 {
        self.total_ticks
    }

    #[inline]
    pub fn total_secs(&self) -> f32 {
        self.ticks_to_secs_f32(self.total_ticks)
    }

    #[inline]
    pub const fn now_ticks(&self) -> u64 {
        self.now_tick
    }

    #[inline]
    pub const fn frame_dt_ticks(&self) -> u64 {
        self.frame_dt_ticks
    }

    #[inline]
    pub fn frame_dt_secs(&self) -> f32 {
        self.ticks_to_secs_f32(self.frame_dt_ticks)
    }

    #[inline]
    pub const fn accumulator_ticks(&self) -> u64 {
        self.accumulator_ticks
    }

    #[inline]
    pub fn max_frame_secs(&self) -> f32 {
        self.ticks_to_secs_f32(self.max_frame_ticks)
    }

    #[inline]
    pub const fn is_started(&self) -> bool {
        self.started
    }

    #[inline]
    pub fn ticks_to_secs_f32(&self, ticks: u64) -> f32 {
        if self.ticks_per_second == 0 {
            0.0
        } else {
            ticks as f32 / self.ticks_per_second as f32
        }
    }

    #[inline]
    pub fn ticks_to_millis_f32(&self, ticks: u64) -> f32 {
        self.ticks_to_secs_f32(ticks) * 1_000.0
    }

    #[inline]
    pub fn secs_to_ticks(&self, secs: f32) -> u64 {
        if secs <= 0.0 || self.ticks_per_second == 0 {
            0
        } else {
            (secs * self.ticks_per_second as f32) as u64
        }
    }
}

const fn seconds_to_ticks_const(ticks_per_second: u64, secs: f32) -> u64 {
    if secs <= 0.0 || ticks_per_second == 0 {
        0
    } else {
        (secs * ticks_per_second as f32) as u64
    }
}