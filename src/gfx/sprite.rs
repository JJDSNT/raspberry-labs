// src/gfx/sprite.rs
//
// Sprite system básico para o engine bare-metal.
//
// Objetivos:
// - representar imagens de sprite em ARGB8888
// - permitir múltiplas instâncias do mesmo sprite
// - suportar prioridade, visibilidade, flip e source rect
// - oferecer um batch sem alocação dinâmica
//
// Observação:
// - este arquivo NÃO depende de gfx::renderer diretamente
// - ele modela dados e batching
// - o Renderer pode consumir SpriteBatch depois
//

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Sprite<'a> {
    pub pixels: &'a [u32],
    pub width: usize,
    pub height: usize,
}

impl<'a> Sprite<'a> {
    #[inline]
    pub const fn new(pixels: &'a [u32], width: usize, height: usize) -> Self {
        Self {
            pixels,
            width,
            height,
        }
    }

    #[inline]
    pub fn is_valid(&self) -> bool {
        self.width > 0
            && self.height > 0
            && self.pixels.len() >= self.width.saturating_mul(self.height)
    }

    #[inline]
    pub const fn full_rect(&self) -> SpriteRect {
        SpriteRect {
            x: 0,
            y: 0,
            w: self.width,
            h: self.height,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SpriteRect {
    pub x: usize,
    pub y: usize,
    pub w: usize,
    pub h: usize,
}

impl SpriteRect {
    #[inline]
    pub const fn new(x: usize, y: usize, w: usize, h: usize) -> Self {
        Self { x, y, w, h }
    }

    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.w == 0 || self.h == 0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SpriteFlags {
    bits: u8,
}

impl SpriteFlags {
    pub const NONE: Self = Self { bits: 0 };
    pub const FLIP_X: Self = Self { bits: 1 << 0 };
    pub const FLIP_Y: Self = Self { bits: 1 << 1 };

    #[inline]
    pub const fn empty() -> Self {
        Self::NONE
    }

    #[inline]
    pub const fn contains(self, other: Self) -> bool {
        (self.bits & other.bits) == other.bits
    }

    #[inline]
    pub const fn with(self, other: Self) -> Self {
        Self {
            bits: self.bits | other.bits,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct SpriteInstance<'a> {
    pub sprite: &'a Sprite<'a>,
    pub src: SpriteRect,
    pub x: i32,
    pub y: i32,
    pub visible: bool,
    pub priority: i16,
    pub flags: SpriteFlags,
}

impl<'a> SpriteInstance<'a> {
    #[inline]
    pub fn new(sprite: &'a Sprite<'a>, x: i32, y: i32) -> Self {
        Self {
            sprite,
            src: sprite.full_rect(),
            x,
            y,
            visible: true,
            priority: 0,
            flags: SpriteFlags::NONE,
        }
    }

    #[inline]
    pub fn with_src(mut self, src: SpriteRect) -> Self {
        self.src = src;
        self
    }

    #[inline]
    pub fn with_priority(mut self, priority: i16) -> Self {
        self.priority = priority;
        self
    }

    #[inline]
    pub fn with_flags(mut self, flags: SpriteFlags) -> Self {
        self.flags = flags;
        self
    }

    #[inline]
    pub fn flip_x(mut self) -> Self {
        self.flags = self.flags.with(SpriteFlags::FLIP_X);
        self
    }

    #[inline]
    pub fn flip_y(mut self) -> Self {
        self.flags = self.flags.with(SpriteFlags::FLIP_Y);
        self
    }

    #[inline]
    pub fn hidden(mut self) -> Self {
        self.visible = false;
        self
    }

    #[inline]
    pub fn is_valid(&self) -> bool {
        if !self.visible || !self.sprite.is_valid() || self.src.is_empty() {
            return false;
        }

        let max_w = self.sprite.width.saturating_sub(self.src.x);
        let max_h = self.sprite.height.saturating_sub(self.src.y);

        self.src.w <= max_w && self.src.h <= max_h
    }
}

pub struct SpriteBatch<'a, const N: usize> {
    items: [Option<SpriteInstance<'a>>; N],
    len: usize,
}

impl<'a, const N: usize> SpriteBatch<'a, N> {
    #[inline]
    pub const fn new() -> Self {
        Self {
            items: [None; N],
            len: 0,
        }
    }

    #[inline]
    pub const fn capacity(&self) -> usize {
        N
    }

    #[inline]
    pub const fn len(&self) -> usize {
        self.len
    }

    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    #[inline]
    pub const fn is_full(&self) -> bool {
        self.len >= N
    }

    #[inline]
    pub fn clear(&mut self) {
        let len = self.len;
        let mut i = 0;
        while i < len {
            self.items[i] = None;
            i += 1;
        }
        self.len = 0;
    }

    #[inline]
    pub fn push(&mut self, instance: SpriteInstance<'a>) -> Result<(), &'static str> {
        if self.len >= N {
            return Err("sprite batch full");
        }

        self.items[self.len] = Some(instance);
        self.len += 1;
        Ok(())
    }

    #[inline]
    pub fn push_visible(&mut self, instance: SpriteInstance<'a>) -> Result<(), &'static str> {
        if !instance.visible {
            return Ok(());
        }

        self.push(instance)
    }

    #[inline]
    pub fn get(&self, index: usize) -> Option<&SpriteInstance<'a>> {
        if index >= self.len {
            None
        } else {
            self.items[index].as_ref()
        }
    }

    #[inline]
    pub fn get_mut(&mut self, index: usize) -> Option<&mut SpriteInstance<'a>> {
        if index >= self.len {
            None
        } else {
            self.items[index].as_mut()
        }
    }

    #[inline]
    pub fn iter(&self) -> SpriteBatchIter<'_, 'a, N> {
        SpriteBatchIter {
            batch: self,
            index: 0,
        }
    }

    /// Itera em ordem de prioridade crescente.
    ///
    /// Regra:
    /// - menor prioridade desenha primeiro
    /// - maior prioridade desenha por cima
    ///
    /// Em caso de empate, mantém a ordem de inserção.
    pub fn for_each_sorted<F>(&self, mut f: F)
    where
        F: FnMut(&SpriteInstance<'a>),
    {
        let mut order = [0usize; N];
        let len = self.len;

        let mut i = 0;
        while i < len {
            order[i] = i;
            i += 1;
        }

        // insertion sort estável por prioridade
        let mut i = 1;
        while i < len {
            let key = order[i];
            let key_prio = self.items[key]
                .as_ref()
                .map(|s| s.priority)
                .unwrap_or(i16::MIN);

            let mut j = i;
            while j > 0 {
                let prev = order[j - 1];
                let prev_prio = self.items[prev]
                    .as_ref()
                    .map(|s| s.priority)
                    .unwrap_or(i16::MIN);

                if prev_prio <= key_prio {
                    break;
                }

                order[j] = order[j - 1];
                j -= 1;
            }

            order[j] = key;
            i += 1;
        }

        let mut i = 0;
        while i < len {
            if let Some(instance) = self.items[order[i]].as_ref() {
                if instance.visible && instance.is_valid() {
                    f(instance);
                }
            }
            i += 1;
        }
    }
}

pub struct SpriteBatchIter<'b, 'a, const N: usize> {
    batch: &'b SpriteBatch<'a, N>,
    index: usize,
}

impl<'b, 'a, const N: usize> Iterator for SpriteBatchIter<'b, 'a, N> {
    type Item = &'b SpriteInstance<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        while self.index < self.batch.len {
            let idx = self.index;
            self.index += 1;

            if let Some(item) = self.batch.items[idx].as_ref() {
                return Some(item);
            }
        }

        None
    }
}

/// Helper utilitário para acesso a pixel já respeitando source rect + flip.
///
/// Retorna `None` se `(local_x, local_y)` estiver fora da source rect.
#[inline]
pub fn sprite_pixel(instance: &SpriteInstance<'_>, local_x: usize, local_y: usize) -> Option<u32> {
    if !instance.is_valid() {
        return None;
    }

    if local_x >= instance.src.w || local_y >= instance.src.h {
        return None;
    }

    let sx = if instance.flags.contains(SpriteFlags::FLIP_X) {
        instance.src.x + (instance.src.w - 1 - local_x)
    } else {
        instance.src.x + local_x
    };

    let sy = if instance.flags.contains(SpriteFlags::FLIP_Y) {
        instance.src.y + (instance.src.h - 1 - local_y)
    } else {
        instance.src.y + local_y
    };

    let sprite = instance.sprite;
    let idx = sy.saturating_mul(sprite.width).saturating_add(sx);

    sprite.pixels.get(idx).copied()
}