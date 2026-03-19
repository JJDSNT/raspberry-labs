// src/math/surfaces.rs
//
// Fórmulas e helpers para superfícies matemáticas.
// Exemplos:
//
// - paraboloide:        z = x² + y²
// - sela:               z = x² - y²
// - ripple / ondas:     z = sin(r) / r
// - seno/cosseno:       z = sin(x) * cos(y)
// - cone
// - gaussian
//
// Também inclui geração de malha regular (grid mesh) e cálculo aproximado
// de normais por diferenças finitas.
//
// Compatível com no_std.

#![allow(dead_code)]

use crate::math::math3d::{cos_approx, sin_approx, sqrt_approx, Vec3};

// -----------------------------------------------------------------------------
// Tipos básicos
// -----------------------------------------------------------------------------

pub type SurfaceFn = fn(x: f32, y: f32) -> f32;

#[derive(Clone, Copy, Debug)]
pub struct Domain2D {
    pub x_min: f32,
    pub x_max: f32,
    pub y_min: f32,
    pub y_max: f32,
}

impl Domain2D {
    #[inline]
    pub const fn new(x_min: f32, x_max: f32, y_min: f32, y_max: f32) -> Self {
        Self {
            x_min,
            x_max,
            y_min,
            y_max,
        }
    }

    #[inline]
    pub fn width(&self) -> f32 {
        self.x_max - self.x_min
    }

    #[inline]
    pub fn height(&self) -> f32 {
        self.y_max - self.y_min
    }

    #[inline]
    pub fn sample_x(&self, ix: usize, nx: usize) -> f32 {
        if nx <= 1 {
            self.x_min
        } else {
            self.x_min + self.width() * (ix as f32 / (nx - 1) as f32)
        }
    }

    #[inline]
    pub fn sample_y(&self, iy: usize, ny: usize) -> f32 {
        if ny <= 1 {
            self.y_min
        } else {
            self.y_min + self.height() * (iy as f32 / (ny - 1) as f32)
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct SurfaceSample {
    pub position: Vec3,
    pub normal: Vec3,
}

impl SurfaceSample {
    #[inline]
    pub const fn new(position: Vec3, normal: Vec3) -> Self {
        Self { position, normal }
    }
}

// -----------------------------------------------------------------------------
// Helpers escalares
// -----------------------------------------------------------------------------

#[inline]
fn absf(x: f32) -> f32 {
    if x < 0.0 { -x } else { x }
}

#[inline]
fn clamp(x: f32, min_v: f32, max_v: f32) -> f32 {
    if x < min_v {
        min_v
    } else if x > max_v {
        max_v
    } else {
        x
    }
}

#[inline]
fn safe_div(n: f32, d: f32) -> f32 {
    if absf(d) < 1e-6 {
        0.0
    } else {
        n / d
    }
}

// -----------------------------------------------------------------------------
// Superfícies clássicas
// -----------------------------------------------------------------------------

/// Paraboloide elíptico:
/// z = x² + y²
#[inline]
pub fn paraboloid(x: f32, y: f32) -> f32 {
    x * x + y * y
}

/// Paraboloide invertido:
/// z = -(x² + y²)
#[inline]
pub fn inverted_paraboloid(x: f32, y: f32) -> f32 {
    -(x * x + y * y)
}

/// Sela / paraboloide hiperbólico:
/// z = x² - y²
#[inline]
pub fn saddle(x: f32, y: f32) -> f32 {
    x * x - y * y
}

/// Plano:
/// z = ax + by + c
#[inline]
pub fn plane(x: f32, y: f32) -> f32 {
    0.3 * x + 0.2 * y
}

/// Onda senoidal:
/// z = sin(x) * cos(y)
#[inline]
pub fn sine_cosine(x: f32, y: f32) -> f32 {
    sin_approx(x) * cos_approx(y)
}

/// Ripple radial:
/// z = sin(r) / r
#[inline]
pub fn ripple(x: f32, y: f32) -> f32 {
    let r = sqrt_approx(x * x + y * y);
    if r < 1e-4 {
        1.0
    } else {
        sin_approx(r) / r
    }
}

/// Cone:
/// z = sqrt(x² + y²)
#[inline]
pub fn cone(x: f32, y: f32) -> f32 {
    sqrt_approx(x * x + y * y)
}

/// Cone invertido:
/// z = -sqrt(x² + y²)
#[inline]
pub fn inverted_cone(x: f32, y: f32) -> f32 {
    -sqrt_approx(x * x + y * y)
}

/// Gaussiana:
/// z = e^(-(x²+y²))
///
/// Aqui usamos uma aproximação racional simples para evitar exp().
#[inline]
pub fn gaussian(x: f32, y: f32) -> f32 {
    let r2 = x * x + y * y;
    1.0 / (1.0 + r2 + 0.5 * r2 * r2)
}

/// Superfície "egg carton":
/// z = sin(x) + cos(y)
#[inline]
pub fn egg_crate(x: f32, y: f32) -> f32 {
    sin_approx(x) + cos_approx(y)
}

/// Ondas diagonais:
/// z = sin(x + y)
#[inline]
pub fn diagonal_waves(x: f32, y: f32) -> f32 {
    sin_approx(x + y)
}

/// Poço circular:
/// z = -(sin(r)/r)
#[inline]
pub fn circular_well(x: f32, y: f32) -> f32 {
    -ripple(x, y)
}

/// "Monkey saddle" simplificada:
/// z = x³ - 3xy²
#[inline]
pub fn monkey_saddle(x: f32, y: f32) -> f32 {
    x * x * x - 3.0 * x * y * y
}

/// Superfície ondulada mais dramática:
/// z = sin(2x) * cos(2y)
#[inline]
pub fn waves2(x: f32, y: f32) -> f32 {
    sin_approx(2.0 * x) * cos_approx(2.0 * y)
}

// -----------------------------------------------------------------------------
// Enum de superfícies prontas
// -----------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SurfaceKind {
    Paraboloid,
    InvertedParaboloid,
    Saddle,
    Plane,
    SineCosine,
    Ripple,
    Cone,
    InvertedCone,
    Gaussian,
    EggCrate,
    DiagonalWaves,
    CircularWell,
    MonkeySaddle,
    Waves2,
}

impl SurfaceKind {
    #[inline]
    pub fn eval(self, x: f32, y: f32) -> f32 {
        match self {
            SurfaceKind::Paraboloid => paraboloid(x, y),
            SurfaceKind::InvertedParaboloid => inverted_paraboloid(x, y),
            SurfaceKind::Saddle => saddle(x, y),
            SurfaceKind::Plane => plane(x, y),
            SurfaceKind::SineCosine => sine_cosine(x, y),
            SurfaceKind::Ripple => ripple(x, y),
            SurfaceKind::Cone => cone(x, y),
            SurfaceKind::InvertedCone => inverted_cone(x, y),
            SurfaceKind::Gaussian => gaussian(x, y),
            SurfaceKind::EggCrate => egg_crate(x, y),
            SurfaceKind::DiagonalWaves => diagonal_waves(x, y),
            SurfaceKind::CircularWell => circular_well(x, y),
            SurfaceKind::MonkeySaddle => monkey_saddle(x, y),
            SurfaceKind::Waves2 => waves2(x, y),
        }
    }
}

// -----------------------------------------------------------------------------
// Amostragem e normais
// -----------------------------------------------------------------------------

/// Avalia a superfície z = f(x, y) e devolve um ponto 3D.
#[inline]
pub fn sample_surface(f: SurfaceFn, x: f32, y: f32) -> Vec3 {
    Vec3::new(x, y, f(x, y))
}

/// Avalia uma `SurfaceKind`.
#[inline]
pub fn sample_surface_kind(kind: SurfaceKind, x: f32, y: f32) -> Vec3 {
    Vec3::new(x, y, kind.eval(x, y))
}

/// Calcula uma normal aproximada por diferenças finitas.
///
/// Para z = f(x,y), montamos tangentes:
/// Tx = (eps, 0, f(x+eps,y)-f(x,y))
/// Ty = (0, eps, f(x,y+eps)-f(x,y))
/// normal = Tx x Ty
#[inline]
pub fn estimate_normal(f: SurfaceFn, x: f32, y: f32, eps: f32) -> Vec3 {
    let z = f(x, y);
    let zx = f(x + eps, y);
    let zy = f(x, y + eps);

    let tx = Vec3::new(eps, 0.0, zx - z);
    let ty = Vec3::new(0.0, eps, zy - z);

    tx.cross(ty).normalized()
}

/// Versão para enum.
#[inline]
pub fn estimate_normal_kind(kind: SurfaceKind, x: f32, y: f32, eps: f32) -> Vec3 {
    let f = |sx: f32, sy: f32| kind.eval(sx, sy);
    let z = f(x, y);
    let zx = f(x + eps, y);
    let zy = f(x, y + eps);

    let tx = Vec3::new(eps, 0.0, zx - z);
    let ty = Vec3::new(0.0, eps, zy - z);

    tx.cross(ty).normalized()
}

/// Amostra completa: posição + normal.
#[inline]
pub fn sample_with_normal(f: SurfaceFn, x: f32, y: f32, eps: f32) -> SurfaceSample {
    SurfaceSample::new(
        sample_surface(f, x, y),
        estimate_normal(f, x, y, eps),
    )
}

/// Amostra completa via enum.
#[inline]
pub fn sample_with_normal_kind(kind: SurfaceKind, x: f32, y: f32, eps: f32) -> SurfaceSample {
    SurfaceSample::new(
        sample_surface_kind(kind, x, y),
        estimate_normal_kind(kind, x, y, eps),
    )
}

// -----------------------------------------------------------------------------
// Geração de grid / malha
// -----------------------------------------------------------------------------

/// Número máximo padrão que pode ser útil para labs pequenos.
/// Não é obrigatório usar; só ajuda como referência.
pub const DEFAULT_NORMAL_EPS: f32 = 0.01;

/// Preenche um buffer de vértices com uma superfície amostrada em grade regular.
///
/// Layout:
/// vertices[iy * nx + ix]
///
/// Retorna `true` se conseguiu preencher tudo; `false` se o slice tiver tamanho
/// insuficiente.
pub fn generate_surface_vertices(
    f: SurfaceFn,
    domain: Domain2D,
    nx: usize,
    ny: usize,
    z_scale: f32,
    out_vertices: &mut [Vec3],
) -> bool {
    if out_vertices.len() < nx * ny {
        return false;
    }

    for iy in 0..ny {
        let y = domain.sample_y(iy, ny);

        for ix in 0..nx {
            let x = domain.sample_x(ix, nx);
            let z = f(x, y) * z_scale;
            out_vertices[iy * nx + ix] = Vec3::new(x, y, z);
        }
    }

    true
}

/// Igual à função acima, mas usando `SurfaceKind`.
pub fn generate_surface_vertices_kind(
    kind: SurfaceKind,
    domain: Domain2D,
    nx: usize,
    ny: usize,
    z_scale: f32,
    out_vertices: &mut [Vec3],
) -> bool {
    if out_vertices.len() < nx * ny {
        return false;
    }

    for iy in 0..ny {
        let y = domain.sample_y(iy, ny);

        for ix in 0..nx {
            let x = domain.sample_x(ix, nx);
            let z = kind.eval(x, y) * z_scale;
            out_vertices[iy * nx + ix] = Vec3::new(x, y, z);
        }
    }

    true
}

/// Preenche buffers de posição e normal.
///
/// Retorna `true` se os slices tiverem tamanho suficiente.
pub fn generate_surface_samples(
    f: SurfaceFn,
    domain: Domain2D,
    nx: usize,
    ny: usize,
    z_scale: f32,
    normal_eps: f32,
    out_positions: &mut [Vec3],
    out_normals: &mut [Vec3],
) -> bool {
    if out_positions.len() < nx * ny || out_normals.len() < nx * ny {
        return false;
    }

    for iy in 0..ny {
        let y = domain.sample_y(iy, ny);

        for ix in 0..nx {
            let x = domain.sample_x(ix, nx);

            let z = f(x, y) * z_scale;
            let n = estimate_normal(
                |sx, sy| f(sx, sy) * z_scale,
                x,
                y,
                normal_eps,
            );

            let idx = iy * nx + ix;
            out_positions[idx] = Vec3::new(x, y, z);
            out_normals[idx] = n;
        }
    }

    true
}

/// Versão com enum.
pub fn generate_surface_samples_kind(
    kind: SurfaceKind,
    domain: Domain2D,
    nx: usize,
    ny: usize,
    z_scale: f32,
    normal_eps: f32,
    out_positions: &mut [Vec3],
    out_normals: &mut [Vec3],
) -> bool {
    if out_positions.len() < nx * ny || out_normals.len() < nx * ny {
        return false;
    }

    for iy in 0..ny {
        let y = domain.sample_y(iy, ny);

        for ix in 0..nx {
            let x = domain.sample_x(ix, nx);

            let z = kind.eval(x, y) * z_scale;
            let n = estimate_normal_kind_scaled(kind, x, y, z_scale, normal_eps);

            let idx = iy * nx + ix;
            out_positions[idx] = Vec3::new(x, y, z);
            out_normals[idx] = n;
        }
    }

    true
}

#[inline]
fn estimate_normal_kind_scaled(
    kind: SurfaceKind,
    x: f32,
    y: f32,
    z_scale: f32,
    eps: f32,
) -> Vec3 {
    let f = |sx: f32, sy: f32| kind.eval(sx, sy) * z_scale;
    let z = f(x, y);
    let zx = f(x + eps, y);
    let zy = f(x, y + eps);

    let tx = Vec3::new(eps, 0.0, zx - z);
    let ty = Vec3::new(0.0, eps, zy - z);

    tx.cross(ty).normalized()
}

// -----------------------------------------------------------------------------
// Helpers para wireframe
// -----------------------------------------------------------------------------

/// Conta quantas linhas um grid `nx x ny` gera em wireframe.
#[inline]
pub fn wireframe_line_count(nx: usize, ny: usize) -> usize {
    if nx == 0 || ny == 0 {
        return 0;
    }

    let horizontal = (nx - 1) * ny;
    let vertical = nx * (ny - 1);
    horizontal + vertical
}

/// Gera pares de índices (a, b) para desenhar wireframe do grid.
///
/// Cada célula se conecta:
/// - horizontalmente
/// - verticalmente
///
/// Retorna `true` se `out_indices` tiver espaço suficiente.
/// O formato é:
/// out_indices[2*i + 0] = a
/// out_indices[2*i + 1] = b
pub fn generate_wireframe_indices(
    nx: usize,
    ny: usize,
    out_indices: &mut [usize],
) -> bool {
    let lines = wireframe_line_count(nx, ny);
    let needed = lines * 2;

    if out_indices.len() < needed {
        return false;
    }

    let mut k = 0;

    for iy in 0..ny {
        for ix in 0..nx {
            let i = iy * nx + ix;

            if ix + 1 < nx {
                out_indices[k] = i;
                out_indices[k + 1] = i + 1;
                k += 2;
            }

            if iy + 1 < ny {
                out_indices[k] = i;
                out_indices[k + 1] = i + nx;
                k += 2;
            }
        }
    }

    true
}

// -----------------------------------------------------------------------------
// Helpers de altura / shading simples
// -----------------------------------------------------------------------------

/// Normaliza uma altura z de [z_min, z_max] para [0, 1].
#[inline]
pub fn normalize_height(z: f32, z_min: f32, z_max: f32) -> f32 {
    let d = z_max - z_min;
    if absf(d) < 1e-6 {
        0.0
    } else {
        clamp((z - z_min) / d, 0.0, 1.0)
    }
}

/// Lambert simples com direção de luz já normalizada.
/// Retorna intensidade em [0, 1].
#[inline]
pub fn lambert(normal: Vec3, light_dir: Vec3) -> f32 {
    let n = normal.normalized();
    let l = light_dir.normalized();
    clamp(n.dot(l), 0.0, 1.0)
}

/// Mapeia intensidade [0,1] para ARGB grayscale.
#[inline]
pub fn grayscale_from_intensity(i: f32) -> u32 {
    let v = (clamp(i, 0.0, 1.0) * 255.0) as u32;
    0xFF00_0000 | (v << 16) | (v << 8) | v
}

/// Mapeia altura normalizada [0,1] para uma paleta simples heatmap.
#[inline]
pub fn color_from_height01(h: f32) -> u32 {
    let h = clamp(h, 0.0, 1.0);

    let r = (255.0 * h) as u32;
    let g = (255.0 * (1.0 - absf(2.0 * h - 1.0))) as u32;
    let b = (255.0 * (1.0 - h)) as u32;

    0xFF00_0000 | (r << 16) | (g << 8) | b
}

// -----------------------------------------------------------------------------
// Helpers de bounds
// -----------------------------------------------------------------------------

#[derive(Clone, Copy, Debug)]
pub struct HeightRange {
    pub min: f32,
    pub max: f32,
}

impl HeightRange {
    #[inline]
    pub const fn new(min: f32, max: f32) -> Self {
        Self { min, max }
    }
}

/// Faz uma varredura na superfície para estimar altura mínima/máxima no domínio.
pub fn estimate_height_range(
    f: SurfaceFn,
    domain: Domain2D,
    nx: usize,
    ny: usize,
    z_scale: f32,
) -> HeightRange {
    let mut min_z = 0.0;
    let mut max_z = 0.0;
    let mut first = true;

    for iy in 0..ny {
        let y = domain.sample_y(iy, ny);

        for ix in 0..nx {
            let x = domain.sample_x(ix, nx);
            let z = f(x, y) * z_scale;

            if first {
                min_z = z;
                max_z = z;
                first = false;
            } else {
                if z < min_z {
                    min_z = z;
                }
                if z > max_z {
                    max_z = z;
                }
            }
        }
    }

    HeightRange::new(min_z, max_z)
}

/// Versão via enum.
pub fn estimate_height_range_kind(
    kind: SurfaceKind,
    domain: Domain2D,
    nx: usize,
    ny: usize,
    z_scale: f32,
) -> HeightRange {
    estimate_height_range(|x, y| kind.eval(x, y), domain, nx, ny, z_scale)
}

// -----------------------------------------------------------------------------
// Superfícies parametrizadas simples
// -----------------------------------------------------------------------------

#[derive(Clone, Copy, Debug)]
pub struct SurfaceParams {
    pub amplitude: f32,
    pub frequency: f32,
    pub bias: f32,
}

impl SurfaceParams {
    #[inline]
    pub const fn new(amplitude: f32, frequency: f32, bias: f32) -> Self {
        Self {
            amplitude,
            frequency,
            bias,
        }
    }
}

/// z = A * sin(Fx) * cos(Fy) + bias
#[inline]
pub fn sine_cosine_param(x: f32, y: f32, p: SurfaceParams) -> f32 {
    p.amplitude * sin_approx(p.frequency * x) * cos_approx(p.frequency * y) + p.bias
}

/// z = A * sin(F*r) / r + bias
#[inline]
pub fn ripple_param(x: f32, y: f32, p: SurfaceParams) -> f32 {
    let r = sqrt_approx(x * x + y * y);
    if r < 1e-4 {
        p.amplitude + p.bias
    } else {
        p.amplitude * safe_div(sin_approx(p.frequency * r), r) + p.bias
    }
}

/// z = A * (x² - y²) + bias
#[inline]
pub fn saddle_param(x: f32, y: f32, p: SurfaceParams) -> f32 {
    p.amplitude * (x * x - y * y) + p.bias
}