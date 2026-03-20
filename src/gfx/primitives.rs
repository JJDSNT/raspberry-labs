// src/gfx/primitives.rs
//
// Primitivas de desenho 2D para bare metal.
// Todas as operações trabalham diretamente sobre um slice de pixels ARGB32.
// Nenhuma dependência de float — apenas aritmética inteira.
//
// API:
//   line(buf, w, h, x0, y0, x1, y1, color)   — linha de Bresenham
//   circle(buf, w, h, cx, cy, r, color)        — círculo de Bresenham (borda)
//   fill_circle(buf, w, h, cx, cy, r, color)   — círculo preenchido
//   ellipse(buf, w, h, cx, cy, rx, ry, color)  — elipse de Bresenham (borda)
//   fill_ellipse(buf, w, h, cx, cy, rx, ry, color) — elipse preenchida

// ---------------------------------------------------------------------------
// Linha — algoritmo de Bresenham
// ---------------------------------------------------------------------------

pub fn line(
    buf: &mut [u32], w: usize, h: usize,
    x0: i32, y0: i32, x1: i32, y1: i32,
    color: u32,
) {
    let mut x0 = x0;
    let mut y0 = y0;

    let dx = (x1 - x0).abs();
    let dy = (y1 - y0).abs();
    let sx: i32 = if x0 < x1 { 1 } else { -1 };
    let sy: i32 = if y0 < y1 { 1 } else { -1 };
    let mut err = dx - dy;

    loop {
        put(buf, w, h, x0, y0, color);

        if x0 == x1 && y0 == y1 { break; }

        let e2 = 2 * err;
        if e2 > -dy {
            err -= dy;
            x0 += sx;
        }
        if e2 < dx {
            err += dx;
            y0 += sy;
        }
    }
}

// ---------------------------------------------------------------------------
// Círculo — algoritmo de Bresenham (ponto médio)
// ---------------------------------------------------------------------------

pub fn circle(
    buf: &mut [u32], w: usize, h: usize,
    cx: i32, cy: i32, r: i32,
    color: u32,
) {
    if r <= 0 { return; }

    let mut x = 0i32;
    let mut y = r;
    let mut d = 1 - r;

    while x <= y {
        plot8(buf, w, h, cx, cy, x, y, color);

        if d < 0 {
            d += 2 * x + 3;
        } else {
            d += 2 * (x - y) + 5;
            y -= 1;
        }
        x += 1;
    }
}

/// Círculo preenchido.
pub fn fill_circle(
    buf: &mut [u32], w: usize, h: usize,
    cx: i32, cy: i32, r: i32,
    color: u32,
) {
    if r <= 0 { return; }

    let mut x = 0i32;
    let mut y = r;
    let mut d = 1 - r;

    while x <= y {
        hspan(buf, w, h, cx - y, cx + y, cy + x, color);
        hspan(buf, w, h, cx - y, cx + y, cy - x, color);
        hspan(buf, w, h, cx - x, cx + x, cy + y, color);
        hspan(buf, w, h, cx - x, cx + x, cy - y, color);

        if d < 0 {
            d += 2 * x + 3;
        } else {
            d += 2 * (x - y) + 5;
            y -= 1;
        }
        x += 1;
    }
}

// ---------------------------------------------------------------------------
// Elipse — algoritmo de Bresenham (ponto médio)
// ---------------------------------------------------------------------------

pub fn ellipse(
    buf: &mut [u32], w: usize, h: usize,
    cx: i32, cy: i32, rx: i32, ry: i32,
    color: u32,
) {
    if rx <= 0 || ry <= 0 { return; }

    let mut x = 0i32;
    let mut y = ry;

    let rx2 = rx * rx;
    let ry2 = ry * ry;
    let two_rx2 = 2 * rx2;
    let two_ry2 = 2 * ry2;

    // Região 1
    let mut px = 0i32;
    let mut py = two_rx2 * y;
    let mut d = ry2 - rx2 * ry + rx2 / 4;

    while px < py {
        plot4(buf, w, h, cx, cy, x, y, color);

        x += 1;
        px += two_ry2;

        if d < 0 {
            d += ry2 + px;
        } else {
            y -= 1;
            py -= two_rx2;
            d += ry2 + px - py;
        }
    }

    // Região 2
    d = ry2 * (x * x) + rx2 * ((y - 1) * (y - 1)) - rx2 * ry2;

    while y >= 0 {
        plot4(buf, w, h, cx, cy, x, y, color);

        y -= 1;
        py -= two_rx2;

        if d > 0 {
            d += rx2 - py;
        } else {
            x += 1;
            px += two_ry2;
            d += rx2 - py + px;
        }
    }
}

/// Elipse preenchida.
pub fn fill_ellipse(
    buf: &mut [u32], w: usize, h: usize,
    cx: i32, cy: i32, rx: i32, ry: i32,
    color: u32,
) {
    if rx <= 0 || ry <= 0 { return; }

    let mut x = 0i32;
    let mut y = ry;

    let rx2 = rx * rx;
    let ry2 = ry * ry;
    let two_rx2 = 2 * rx2;
    let two_ry2 = 2 * ry2;

    let mut px = 0i32;
    let mut py = two_rx2 * y;
    let mut d = ry2 - rx2 * ry + rx2 / 4;

    while px < py {
        hspan(buf, w, h, cx - x, cx + x, cy + y, color);
        hspan(buf, w, h, cx - x, cx + x, cy - y, color);

        x += 1;
        px += two_ry2;

        if d < 0 {
            d += ry2 + px;
        } else {
            y -= 1;
            py -= two_rx2;
            d += ry2 + px - py;
        }
    }

    d = ry2 * (x * x) + rx2 * ((y - 1) * (y - 1)) - rx2 * ry2;

    while y >= 0 {
        hspan(buf, w, h, cx - x, cx + x, cy + y, color);
        hspan(buf, w, h, cx - x, cx + x, cy - y, color);

        y -= 1;
        py -= two_rx2;

        if d > 0 {
            d += rx2 - py;
        } else {
            x += 1;
            px += two_ry2;
            d += rx2 - py + px;
        }
    }
}

// ---------------------------------------------------------------------------
// Helpers internos
// ---------------------------------------------------------------------------

/// Escreve um pixel com clipping.
#[inline]
fn put(buf: &mut [u32], w: usize, h: usize, x: i32, y: i32, color: u32) {
    if x >= 0 && y >= 0 {
        let (x, y) = (x as usize, y as usize);
        if x < w && y < h {
            buf[y * w + x] = color;
        }
    }
}

/// Linha horizontal de x0 a x1 (inclusive), com clipping.
#[inline]
fn hspan(buf: &mut [u32], w: usize, h: usize, x0: i32, x1: i32, y: i32, color: u32) {
    if y < 0 || y as usize >= h { return; }
    let y = y as usize;
    let x0 = x0.max(0) as usize;
    let x1 = (x1.min(w as i32 - 1)) as usize;
    if x0 > x1 { return; }
    buf[y * w + x0..=y * w + x1].fill(color);
}

/// Plota os 8 pontos simétricos de um círculo.
#[inline]
fn plot8(buf: &mut [u32], w: usize, h: usize, cx: i32, cy: i32, x: i32, y: i32, color: u32) {
    put(buf, w, h, cx + x, cy + y, color);
    put(buf, w, h, cx - x, cy + y, color);
    put(buf, w, h, cx + x, cy - y, color);
    put(buf, w, h, cx - x, cy - y, color);
    put(buf, w, h, cx + y, cy + x, color);
    put(buf, w, h, cx - y, cy + x, color);
    put(buf, w, h, cx + y, cy - x, color);
    put(buf, w, h, cx - y, cy - x, color);
}

/// Plota os 4 pontos simétricos de uma elipse.
#[inline]
fn plot4(buf: &mut [u32], w: usize, h: usize, cx: i32, cy: i32, x: i32, y: i32, color: u32) {
    put(buf, w, h, cx + x, cy + y, color);
    put(buf, w, h, cx - x, cy + y, color);
    put(buf, w, h, cx + x, cy - y, color);
    put(buf, w, h, cx - x, cy - y, color);
}