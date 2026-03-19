// src/math/math3d.rs
//
// Math3D básico para demo engine bare-metal.
// Fornece:
//
// - Vec2, Vec3, Vec4
// - Mat4
// - operações vetoriais
// - rotações / translação / escala
// - projeção em perspectiva
// - projeção de ponto 3D para coordenadas de tela
//
// Compatível com no_std.

#![allow(dead_code)]

use core::ops::{Add, AddAssign, Div, Mul, Neg, Sub, SubAssign};

// -----------------------------------------------------------------------------
// Constantes
// -----------------------------------------------------------------------------

pub const PI: f32 = core::f32::consts::PI;
pub const TAU: f32 = core::f32::consts::TAU;
pub const DEG_TO_RAD: f32 = PI / 180.0;
pub const RAD_TO_DEG: f32 = 180.0 / PI;

// -----------------------------------------------------------------------------
// Helpers escalares
// -----------------------------------------------------------------------------

#[inline]
pub fn deg_to_rad(deg: f32) -> f32 {
    deg * DEG_TO_RAD
}

#[inline]
pub fn rad_to_deg(rad: f32) -> f32 {
    rad * RAD_TO_DEG
}

#[inline]
pub fn clamp(x: f32, min_v: f32, max_v: f32) -> f32 {
    if x < min_v {
        min_v
    } else if x > max_v {
        max_v
    } else {
        x
    }
}

#[inline]
pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

#[inline]
pub fn inv_lerp(a: f32, b: f32, v: f32) -> f32 {
    let d = b - a;
    if d == 0.0 {
        0.0
    } else {
        (v - a) / d
    }
}

#[inline]
pub fn remap(in_min: f32, in_max: f32, out_min: f32, out_max: f32, v: f32) -> f32 {
    let t = inv_lerp(in_min, in_max, v);
    lerp(out_min, out_max, t)
}

// -----------------------------------------------------------------------------
// Aproximações trigonométricas
// -----------------------------------------------------------------------------
//
// Para bare-metal/labs, isso evita depender demais de libm.
// Se vocês já tiverem trig melhor em outro módulo, pode trocar.
//
// Estratégia:
// - wrap do ângulo em [-PI, PI]
// - aproximação polinomial rápida
//
// É suficiente para wireframe / perspectiva / demos simples.

#[inline]
pub fn wrap_pi(mut x: f32) -> f32 {
    while x > PI {
        x -= TAU;
    }
    while x < -PI {
        x += TAU;
    }
    x
}

#[inline]
pub fn sin_approx(x: f32) -> f32 {
    let x = wrap_pi(x);

    // Aproximação rápida:
    // y = Bx + Cx|x|, seguida de correção cúbica
    const B: f32 = 4.0 / PI;
    const C: f32 = -4.0 / (PI * PI);
    const P: f32 = 0.225;

    let y = B * x + C * x * absf(x);
    P * (y * absf(y) - y) + y
}

#[inline]
pub fn cos_approx(x: f32) -> f32 {
    sin_approx(x + PI * 0.5)
}

#[inline]
pub fn absf(x: f32) -> f32 {
    if x < 0.0 { -x } else { x }
}

#[inline]
pub fn sqrt_approx(x: f32) -> f32 {
    if x <= 0.0 {
        return 0.0;
    }

    // Newton-Raphson simples
    let mut g = x;
    for _ in 0..5 {
        g = 0.5 * (g + x / g);
    }
    g
}

// -----------------------------------------------------------------------------
// Vec2
// -----------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    #[inline]
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    #[inline]
    pub const fn zero() -> Self {
        Self { x: 0.0, y: 0.0 }
    }

    #[inline]
    pub fn dot(self, other: Self) -> f32 {
        self.x * other.x + self.y * other.y
    }

    #[inline]
    pub fn length_sq(self) -> f32 {
        self.dot(self)
    }

    #[inline]
    pub fn length(self) -> f32 {
        sqrt_approx(self.length_sq())
    }

    #[inline]
    pub fn normalized(self) -> Self {
        let len = self.length();
        if len == 0.0 {
            Self::zero()
        } else {
            self / len
        }
    }

    #[inline]
    pub fn lerp(self, other: Self, t: f32) -> Self {
        Self::new(
            crate_lerp(self.x, other.x, t),
            crate_lerp(self.y, other.y, t),
        )
    }
}

impl Add for Vec2 {
    type Output = Self;
    #[inline]
    fn add(self, rhs: Self) -> Self {
        Self::new(self.x + rhs.x, self.y + rhs.y)
    }
}

impl Sub for Vec2 {
    type Output = Self;
    #[inline]
    fn sub(self, rhs: Self) -> Self {
        Self::new(self.x - rhs.x, self.y - rhs.y)
    }
}

impl Mul<f32> for Vec2 {
    type Output = Self;
    #[inline]
    fn mul(self, rhs: f32) -> Self {
        Self::new(self.x * rhs, self.y * rhs)
    }
}

impl Div<f32> for Vec2 {
    type Output = Self;
    #[inline]
    fn div(self, rhs: f32) -> Self {
        Self::new(self.x / rhs, self.y / rhs)
    }
}

impl AddAssign for Vec2 {
    #[inline]
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

impl SubAssign for Vec2 {
    #[inline]
    fn sub_assign(&mut self, rhs: Self) {
        self.x -= rhs.x;
        self.y -= rhs.y;
    }
}

impl Neg for Vec2 {
    type Output = Self;
    #[inline]
    fn neg(self) -> Self {
        Self::new(-self.x, -self.y)
    }
}

// -----------------------------------------------------------------------------
// Vec3
// -----------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vec3 {
    #[inline]
    pub const fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    #[inline]
    pub const fn zero() -> Self {
        Self::new(0.0, 0.0, 0.0)
    }

    #[inline]
    pub const fn one() -> Self {
        Self::new(1.0, 1.0, 1.0)
    }

    #[inline]
    pub const fn up() -> Self {
        Self::new(0.0, 1.0, 0.0)
    }

    #[inline]
    pub const fn right() -> Self {
        Self::new(1.0, 0.0, 0.0)
    }

    #[inline]
    pub const fn forward() -> Self {
        Self::new(0.0, 0.0, 1.0)
    }

    #[inline]
    pub fn dot(self, other: Self) -> f32 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    #[inline]
    pub fn cross(self, other: Self) -> Self {
        Self::new(
            self.y * other.z - self.z * other.y,
            self.z * other.x - self.x * other.z,
            self.x * other.y - self.y * other.x,
        )
    }

    #[inline]
    pub fn length_sq(self) -> f32 {
        self.dot(self)
    }

    #[inline]
    pub fn length(self) -> f32 {
        sqrt_approx(self.length_sq())
    }

    #[inline]
    pub fn normalized(self) -> Self {
        let len = self.length();
        if len == 0.0 {
            Self::zero()
        } else {
            self / len
        }
    }

    #[inline]
    pub fn with_w(self, w: f32) -> Vec4 {
        Vec4::new(self.x, self.y, self.z, w)
    }

    #[inline]
    pub fn lerp(self, other: Self, t: f32) -> Self {
        Self::new(
            crate_lerp(self.x, other.x, t),
            crate_lerp(self.y, other.y, t),
            crate_lerp(self.z, other.z, t),
        )
    }
}

impl Add for Vec3 {
    type Output = Self;
    #[inline]
    fn add(self, rhs: Self) -> Self {
        Self::new(self.x + rhs.x, self.y + rhs.y, self.z + rhs.z)
    }
}

impl Sub for Vec3 {
    type Output = Self;
    #[inline]
    fn sub(self, rhs: Self) -> Self {
        Self::new(self.x - rhs.x, self.y - rhs.y, self.z - rhs.z)
    }
}

impl Mul<f32> for Vec3 {
    type Output = Self;
    #[inline]
    fn mul(self, rhs: f32) -> Self {
        Self::new(self.x * rhs, self.y * rhs, self.z * rhs)
    }
}

impl Div<f32> for Vec3 {
    type Output = Self;
    #[inline]
    fn div(self, rhs: f32) -> Self {
        Self::new(self.x / rhs, self.y / rhs, self.z / rhs)
    }
}

impl AddAssign for Vec3 {
    #[inline]
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
        self.z += rhs.z;
    }
}

impl SubAssign for Vec3 {
    #[inline]
    fn sub_assign(&mut self, rhs: Self) {
        self.x -= rhs.x;
        self.y -= rhs.y;
        self.z -= rhs.z;
    }
}

impl Neg for Vec3 {
    type Output = Self;
    #[inline]
    fn neg(self) -> Self {
        Self::new(-self.x, -self.y, -self.z)
    }
}

// -----------------------------------------------------------------------------
// Vec4
// -----------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Vec4 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

impl Vec4 {
    #[inline]
    pub const fn new(x: f32, y: f32, z: f32, w: f32) -> Self {
        Self { x, y, z, w }
    }

    #[inline]
    pub const fn from_vec3(v: Vec3, w: f32) -> Self {
        Self::new(v.x, v.y, v.z, w)
    }

    #[inline]
    pub fn xyz(self) -> Vec3 {
        Vec3::new(self.x, self.y, self.z)
    }

    #[inline]
    pub fn perspective_divide(self) -> Vec3 {
        if self.w == 0.0 {
            Vec3::zero()
        } else {
            Vec3::new(self.x / self.w, self.y / self.w, self.z / self.w)
        }
    }
}

// -----------------------------------------------------------------------------
// Mat4
// -----------------------------------------------------------------------------
//
// Row-major, multiplicação de vetor coluna:
// result = M * v
//
// Layout:
//
// [ m00 m01 m02 m03 ]
// [ m10 m11 m12 m13 ]
// [ m20 m21 m22 m23 ]
// [ m30 m31 m32 m33 ]

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Mat4 {
    pub m: [[f32; 4]; 4],
}

impl Default for Mat4 {
    #[inline]
    fn default() -> Self {
        Self::identity()
    }
}

impl Mat4 {
    #[inline]
    pub const fn new(m: [[f32; 4]; 4]) -> Self {
        Self { m }
    }

    #[inline]
    pub const fn identity() -> Self {
        Self {
            m: [
                [1.0, 0.0, 0.0, 0.0], //
                [0.0, 1.0, 0.0, 0.0], //
                [0.0, 0.0, 1.0, 0.0], //
                [0.0, 0.0, 0.0, 1.0],
            ],
        }
    }

    #[inline]
    pub const fn zero() -> Self {
        Self { m: [[0.0; 4]; 4] }
    }

    #[inline]
    pub fn translation(x: f32, y: f32, z: f32) -> Self {
        Self {
            m: [
                [1.0, 0.0, 0.0, x], //
                [0.0, 1.0, 0.0, y], //
                [0.0, 0.0, 1.0, z], //
                [0.0, 0.0, 0.0, 1.0],
            ],
        }
    }

    #[inline]
    pub fn translation_v(v: Vec3) -> Self {
        Self::translation(v.x, v.y, v.z)
    }

    #[inline]
    pub fn scale(x: f32, y: f32, z: f32) -> Self {
        Self {
            m: [
                [x, 0.0, 0.0, 0.0], //
                [0.0, y, 0.0, 0.0], //
                [0.0, 0.0, z, 0.0], //
                [0.0, 0.0, 0.0, 1.0],
            ],
        }
    }

    #[inline]
    pub fn uniform_scale(s: f32) -> Self {
        Self::scale(s, s, s)
    }

    #[inline]
    pub fn rotation_x(angle_rad: f32) -> Self {
        let s = sin_approx(angle_rad);
        let c = cos_approx(angle_rad);

        Self {
            m: [
                [1.0, 0.0, 0.0, 0.0], //
                [0.0, c, -s, 0.0],    //
                [0.0, s, c, 0.0],     //
                [0.0, 0.0, 0.0, 1.0],
            ],
        }
    }

    #[inline]
    pub fn rotation_y(angle_rad: f32) -> Self {
        let s = sin_approx(angle_rad);
        let c = cos_approx(angle_rad);

        Self {
            m: [
                [c, 0.0, s, 0.0],     //
                [0.0, 1.0, 0.0, 0.0], //
                [-s, 0.0, c, 0.0],    //
                [0.0, 0.0, 0.0, 1.0],
            ],
        }
    }

    #[inline]
    pub fn rotation_z(angle_rad: f32) -> Self {
        let s = sin_approx(angle_rad);
        let c = cos_approx(angle_rad);

        Self {
            m: [
                [c, -s, 0.0, 0.0],    //
                [s, c, 0.0, 0.0],     //
                [0.0, 0.0, 1.0, 0.0], //
                [0.0, 0.0, 0.0, 1.0],
            ],
        }
    }

    #[inline]
    pub fn rotation_xyz(rx: f32, ry: f32, rz: f32) -> Self {
        Self::rotation_z(rz) * Self::rotation_y(ry) * Self::rotation_x(rx)
    }

    #[inline]
    pub fn transpose(self) -> Self {
        let m = self.m;
        Self {
            m: [
                [m[0][0], m[1][0], m[2][0], m[3][0]],
                [m[0][1], m[1][1], m[2][1], m[3][1]],
                [m[0][2], m[1][2], m[2][2], m[3][2]],
                [m[0][3], m[1][3], m[2][3], m[3][3]],
            ],
        }
    }

    #[inline]
    pub fn mul_vec4(self, v: Vec4) -> Vec4 {
        Vec4::new(
            self.m[0][0] * v.x + self.m[0][1] * v.y + self.m[0][2] * v.z + self.m[0][3] * v.w,
            self.m[1][0] * v.x + self.m[1][1] * v.y + self.m[1][2] * v.z + self.m[1][3] * v.w,
            self.m[2][0] * v.x + self.m[2][1] * v.y + self.m[2][2] * v.z + self.m[2][3] * v.w,
            self.m[3][0] * v.x + self.m[3][1] * v.y + self.m[3][2] * v.z + self.m[3][3] * v.w,
        )
    }

    #[inline]
    pub fn transform_point3(self, v: Vec3) -> Vec3 {
        self.mul_vec4(Vec4::from_vec3(v, 1.0)).perspective_divide()
    }

    #[inline]
    pub fn transform_vector3(self, v: Vec3) -> Vec3 {
        self.mul_vec4(Vec4::from_vec3(v, 0.0)).xyz()
    }

    /// Matriz de perspectiva estilo RH simples.
    ///
    /// `fov_y_rad`: campo de visão vertical em radianos
    /// `aspect`: largura / altura
    /// `z_near`, `z_far`: planos de corte
    #[inline]
    pub fn perspective(fov_y_rad: f32, aspect: f32, z_near: f32, z_far: f32) -> Self {
        let f = 1.0 / tan_half_approx(fov_y_rad * 0.5);
        let nf = 1.0 / (z_near - z_far);

        Self {
            m: [
                [f / aspect, 0.0, 0.0, 0.0],
                [0.0, f, 0.0, 0.0],
                [0.0, 0.0, (z_far + z_near) * nf, (2.0 * z_far * z_near) * nf],
                [0.0, 0.0, -1.0, 0.0],
            ],
        }
    }

    /// Look-at RH simples.
    #[inline]
    pub fn look_at(eye: Vec3, target: Vec3, up: Vec3) -> Self {
        let f = (target - eye).normalized();
        let s = f.cross(up).normalized();
        let u = s.cross(f);

        Self {
            m: [
                [s.x, s.y, s.z, -s.dot(eye)],
                [u.x, u.y, u.z, -u.dot(eye)],
                [-f.x, -f.y, -f.z, f.dot(eye)],
                [0.0, 0.0, 0.0, 1.0],
            ],
        }
    }
}

impl Mul for Mat4 {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self {
        let mut out = Self::zero();

        for r in 0..4 {
            for c in 0..4 {
                out.m[r][c] = self.m[r][0] * rhs.m[0][c]
                    + self.m[r][1] * rhs.m[1][c]
                    + self.m[r][2] * rhs.m[2][c]
                    + self.m[r][3] * rhs.m[3][c];
            }
        }

        out
    }
}

impl Mul<Vec4> for Mat4 {
    type Output = Vec4;

    #[inline]
    fn mul(self, rhs: Vec4) -> Vec4 {
        self.mul_vec4(rhs)
    }
}

// -----------------------------------------------------------------------------
// Camera / Projection helpers
// -----------------------------------------------------------------------------

#[derive(Clone, Copy, Debug)]
pub struct Camera {
    pub position: Vec3,
    pub target: Vec3,
    pub up: Vec3,
    pub fov_y_rad: f32,
    pub aspect: f32,
    pub z_near: f32,
    pub z_far: f32,
}

impl Camera {
    #[inline]
    pub fn new(
        position: Vec3,
        target: Vec3,
        up: Vec3,
        fov_y_rad: f32,
        aspect: f32,
        z_near: f32,
        z_far: f32,
    ) -> Self {
        Self {
            position,
            target,
            up,
            fov_y_rad,
            aspect,
            z_near,
            z_far,
        }
    }

    #[inline]
    pub fn view_matrix(&self) -> Mat4 {
        Mat4::look_at(self.position, self.target, self.up)
    }

    #[inline]
    pub fn projection_matrix(&self) -> Mat4 {
        Mat4::perspective(self.fov_y_rad, self.aspect, self.z_near, self.z_far)
    }

    #[inline]
    pub fn view_projection_matrix(&self) -> Mat4 {
        self.projection_matrix() * self.view_matrix()
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ProjectedPoint {
    pub screen: Vec2,
    pub depth: f32,
    pub visible: bool,
}

/// Projeta um ponto já em espaço de câmera, usando perspectiva manual simples.
///
/// Convenção:
/// - câmera olhando para +Z
/// - ponto com z <= 0 está atrás da câmera
#[inline]
pub fn project_camera_space_point(
    p: Vec3,
    screen_w: usize,
    screen_h: usize,
    focal_len: f32,
) -> ProjectedPoint {
    if p.z <= 0.0001 {
        return ProjectedPoint {
            screen: Vec2::zero(),
            depth: p.z,
            visible: false,
        };
    }

    let inv_z = 1.0 / p.z;
    let x_ndc = p.x * focal_len * inv_z;
    let y_ndc = p.y * focal_len * inv_z;

    let hw = screen_w as f32 * 0.5;
    let hh = screen_h as f32 * 0.5;

    let sx = hw + x_ndc * hw;
    let sy = hh - y_ndc * hh;

    ProjectedPoint {
        screen: Vec2::new(sx, sy),
        depth: p.z,
        visible: sx >= 0.0 && sx < screen_w as f32 && sy >= 0.0 && sy < screen_h as f32,
    }
}

/// Pipeline completo:
/// model -> view -> projection -> screen
#[inline]
pub fn project_world_point(
    point: Vec3,
    model: Mat4,
    view: Mat4,
    proj: Mat4,
    screen_w: usize,
    screen_h: usize,
) -> ProjectedPoint {
    let mvp = proj * view * model;
    let clip = mvp * point.with_w(1.0);

    if clip.w == 0.0 {
        return ProjectedPoint {
            screen: Vec2::zero(),
            depth: clip.z,
            visible: false,
        };
    }

    let ndc = clip.perspective_divide();

    let visible = ndc.x >= -1.0
        && ndc.x <= 1.0
        && ndc.y >= -1.0
        && ndc.y <= 1.0
        && ndc.z >= -1.0
        && ndc.z <= 1.0;

    let sx = (ndc.x + 1.0) * 0.5 * screen_w as f32;
    let sy = (1.0 - (ndc.y + 1.0) * 0.5) * screen_h as f32;

    ProjectedPoint {
        screen: Vec2::new(sx, sy),
        depth: ndc.z,
        visible,
    }
}

// -----------------------------------------------------------------------------
// Mesh helpers simples
// -----------------------------------------------------------------------------

#[derive(Clone, Copy, Debug)]
pub struct Triangle3 {
    pub a: Vec3,
    pub b: Vec3,
    pub c: Vec3,
}

impl Triangle3 {
    #[inline]
    pub const fn new(a: Vec3, b: Vec3, c: Vec3) -> Self {
        Self { a, b, c }
    }

    #[inline]
    pub fn normal(self) -> Vec3 {
        let ab = self.b - self.a;
        let ac = self.c - self.a;
        ab.cross(ac).normalized()
    }

    #[inline]
    pub fn center(self) -> Vec3 {
        (self.a + self.b + self.c) / 3.0
    }
}

#[inline]
pub fn backface_cull(tri: Triangle3, camera_pos: Vec3) -> bool {
    let n = tri.normal();
    let to_cam = (camera_pos - tri.center()).normalized();
    n.dot(to_cam) <= 0.0
}

// -----------------------------------------------------------------------------
// Objetos básicos para wireframe
// -----------------------------------------------------------------------------

pub const CUBE_VERTICES: [Vec3; 8] = [
    Vec3::new(-1.0, -1.0, -1.0),
    Vec3::new(1.0, -1.0, -1.0),
    Vec3::new(1.0, 1.0, -1.0),
    Vec3::new(-1.0, 1.0, -1.0),
    Vec3::new(-1.0, -1.0, 1.0),
    Vec3::new(1.0, -1.0, 1.0),
    Vec3::new(1.0, 1.0, 1.0),
    Vec3::new(-1.0, 1.0, 1.0),
];

pub const CUBE_EDGES: [(usize, usize); 12] = [
    (0, 1),
    (1, 2),
    (2, 3),
    (3, 0),
    (4, 5),
    (5, 6),
    (6, 7),
    (7, 4),
    (0, 4),
    (1, 5),
    (2, 6),
    (3, 7),
];

// -----------------------------------------------------------------------------
// Helpers internos
// -----------------------------------------------------------------------------

#[inline]
fn tan_half_approx(x: f32) -> f32 {
    let s = sin_approx(x);
    let c = cos_approx(x);

    if absf(c) < 0.00001 {
        if s >= 0.0 { 1e6 } else { -1e6 }
    } else {
        s / c
    }
}

#[inline]
fn crate_lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}