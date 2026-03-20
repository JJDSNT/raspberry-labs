// src/math/raytracer.rs
//
// Raytracer puro — sem std, sem alloc, sem noção de demo ou buffer.
//
// Portado de renderer.rs do projeto juggler-in-rust (v0.2.0).
// Substituições em relação ao original:
//   - vecmath crate → Vec3 inline com libm::sqrtf
//   - f64 → f32 (VFPv4 do Pi 3B acelera f32 significativamente)
//   - Vec<Sphere>/Vec<Light> → slices estáticos (&[Sphere], &[Light])
//   - Sem threads, sem Arc, sem Mutex
//
// API pública:
//   trace_ray(scene, origin, dir, t_min, t_max, depth) → (f32,f32,f32)
//   color_to_argb(r,g,b) → u32

// ---------------------------------------------------------------------------
// Vec3
// ---------------------------------------------------------------------------

pub type Vec3 = [f32; 3];

#[inline] pub fn v3_add(a: Vec3, b: Vec3) -> Vec3 { [a[0]+b[0], a[1]+b[1], a[2]+b[2]] }
#[inline] pub fn v3_sub(a: Vec3, b: Vec3) -> Vec3 { [a[0]-b[0], a[1]-b[1], a[2]-b[2]] }
#[inline] pub fn v3_scale(a: Vec3, s: f32)  -> Vec3 { [a[0]*s, a[1]*s, a[2]*s] }
#[inline] pub fn v3_dot(a: Vec3, b: Vec3)   -> f32  { a[0]*b[0] + a[1]*b[1] + a[2]*b[2] }
#[inline] pub fn v3_len(a: Vec3)            -> f32  { libm::sqrtf(v3_dot(a, a)) }
#[inline] pub fn v3_len2(a: Vec3)           -> f32  { v3_dot(a, a) }

#[inline]
pub fn v3_normalized(a: Vec3) -> Vec3 {
    let len = v3_len(a);
    if len < 1e-8 { return [0.0, 0.0, 0.0]; }
    v3_scale(a, 1.0 / len)
}

#[inline]
pub fn v3_cross(a: Vec3, b: Vec3) -> Vec3 {
    [
        a[1]*b[2] - a[2]*b[1],
        a[2]*b[0] - a[0]*b[2],
        a[0]*b[1] - a[1]*b[0],
    ]
}

// ---------------------------------------------------------------------------
// Camera
// ---------------------------------------------------------------------------

#[derive(Clone, Copy)]
pub struct Camera {
    pub pos:     Vec3,
    pub right:   Vec3,
    pub up:      Vec3,
    pub forward: Vec3,
}

impl Camera {
    pub fn look_at(&mut self, target: Vec3) {
        self.forward = v3_normalized(v3_sub(target, self.pos));
        self.right   = v3_normalized(v3_cross(self.up, self.forward));
        self.up      = v3_normalized(v3_cross(self.forward, self.right));
    }
}

// ---------------------------------------------------------------------------
// Texture
// ---------------------------------------------------------------------------

#[derive(Clone, Copy)]
pub enum Texture {
    Color(f32, f32, f32),
    CheckerXZ {
        color1: (f32, f32, f32),
        color2: (f32, f32, f32),
        scale:  f32,
    },
    GradientY {
        color1: (f32, f32, f32),
        color2: (f32, f32, f32),
    },
}

// ---------------------------------------------------------------------------
// Sphere
// ---------------------------------------------------------------------------

#[derive(Clone, Copy)]
pub struct Sphere {
    pub pos:           Vec3,
    pub r:             f32,
    pub texture:       Texture,
    pub specular:      f32,
    pub reflective:    f32,
    pub skip_lighting: bool,
}

// ---------------------------------------------------------------------------
// Light
// ---------------------------------------------------------------------------

#[derive(Clone, Copy)]
pub enum Light {
    Ambient   { intensity: f32 },
    Point     { intensity: f32, pos: Vec3 },
    Directional { intensity: f32, dir: Vec3 },
}

// ---------------------------------------------------------------------------
// Scene — referências para slices, sem alloc
// ---------------------------------------------------------------------------

pub struct Scene<'a> {
    pub camera:    Camera,
    pub spheres:   &'a [Sphere],
    pub lights:    &'a [Light],
    pub sky_color: (f32, f32, f32),
}

// ---------------------------------------------------------------------------
// Constantes
// ---------------------------------------------------------------------------

const EPSILON: f32 = 0.0001;

// ---------------------------------------------------------------------------
// API pública
// ---------------------------------------------------------------------------

/// Traça um raio e retorna a cor (r, g, b) em [0.0, 1.0].
pub fn trace_ray(
    scene:           &Scene,
    ray_origin:      Vec3,
    ray_dir:         Vec3,
    t_min:           f32,
    t_max:           f32,
    recursion_depth: usize,
) -> (f32, f32, f32) {
    let (closest, closest_t) =
        intersect_closest(scene, ray_origin, ray_dir, t_min, t_max);

    let sphere = match closest {
        Some(s) => s,
        None    => return scene.sky_color,
    };

    let hit_pos    = v3_add(ray_origin, v3_scale(ray_dir, closest_t));
    let hit_normal = v3_normalized(v3_sub(hit_pos, sphere.pos));

    let intensity = if sphere.skip_lighting {
        1.0
    } else {
        compute_lighting(scene, ray_dir, hit_pos, hit_normal, sphere.specular)
    };

    let (mut r, mut g, mut b) = sample_texture(&sphere.texture, hit_pos, sphere);

    r *= intensity;
    g *= intensity;
    b *= intensity;

    if recursion_depth > 0 && sphere.reflective > 0.0 {
        let refl_dir = reflect_ray(v3_scale(ray_dir, -1.0), hit_normal);
        let (rr, rg, rb) =
            trace_ray(scene, hit_pos, refl_dir, EPSILON, f32::INFINITY, recursion_depth - 1);

        let rf = sphere.reflective;
        r = r * (1.0 - rf) + rr * rf;
        g = g * (1.0 - rf) + rg * rf;
        b = b * (1.0 - rf) + rb * rf;
    }

    (r, g, b)
}

/// Converte (r, g, b) em [0,1] para ARGB8888.
#[inline]
pub fn color_to_argb(r: f32, g: f32, b: f32) -> u32 {
    let r = (r.min(1.0).max(0.0) * 255.0) as u32;
    let g = (g.min(1.0).max(0.0) * 255.0) as u32;
    let b = (b.min(1.0).max(0.0) * 255.0) as u32;
    0xFF00_0000 | (r << 16) | (g << 8) | b
}

// ---------------------------------------------------------------------------
// Interseção
// ---------------------------------------------------------------------------

fn intersect_closest<'a>(
    scene:      &'a Scene,
    ray_origin: Vec3,
    ray_dir:    Vec3,
    t_min:      f32,
    t_max:      f32,
) -> (Option<&'a Sphere>, f32) {
    let mut closest_t      = f32::INFINITY;
    let mut closest_sphere = None;

    for sphere in scene.spheres {
        let (t1, t2) = intersect_sphere(ray_origin, ray_dir, sphere);

        if t1 >= t_min && t1 <= t_max && t1 < closest_t {
            closest_t      = t1;
            closest_sphere = Some(sphere);
        }
        if t2 >= t_min && t2 <= t_max && t2 < closest_t {
            closest_t      = t2;
            closest_sphere = Some(sphere);
        }
    }

    (closest_sphere, closest_t)
}

fn intersect_sphere(ray_origin: Vec3, ray_dir: Vec3, sphere: &Sphere) -> (f32, f32) {
    let co = v3_sub(ray_origin, sphere.pos);
    let a  = v3_dot(ray_dir, ray_dir);
    let b  = 2.0 * v3_dot(co, ray_dir);
    let c  = v3_dot(co, co) - sphere.r * sphere.r;

    let disc = b * b - 4.0 * a * c;
    if disc < 0.0 {
        return (f32::INFINITY, f32::INFINITY);
    }

    let sq = libm::sqrtf(disc);
    let t1 = (-b + sq) / (2.0 * a);
    let t2 = (-b - sq) / (2.0 * a);
    (t1, t2)
}

// ---------------------------------------------------------------------------
// Iluminação
// ---------------------------------------------------------------------------

fn compute_lighting(
    scene:      &Scene,
    ray_dir:    Vec3,
    hit_pos:    Vec3,
    hit_normal: Vec3,
    specular:   f32,
) -> f32 {
    let mut total = 0.0f32;

    for light in scene.lights {
        let (light_intensity, light_dir, t_max) = match light {
            Light::Ambient { intensity } => {
                total += intensity;
                continue;
            }
            Light::Point { intensity, pos } => {
                (*intensity, v3_sub(*pos, hit_pos), 1.0f32)
            }
            Light::Directional { intensity, dir } => {
                (*intensity, *dir, f32::INFINITY)
            }
        };

        // Sombra
        let (shadow, _) = intersect_closest(scene, hit_pos, light_dir, EPSILON, t_max);
        if shadow.is_some() { continue; }

        // Difuso
        let n_dot_l = v3_dot(hit_normal, light_dir);
        if n_dot_l > 0.0 {
            let norm = n_dot_l / (v3_len(hit_normal) * v3_len(light_dir));
            total += light_intensity * norm;
        }

        // Especular
        if specular >= 0.0 {
            let view_dir  = v3_scale(ray_dir, -1.0);
            let refl_dir  = reflect_ray(light_dir, hit_normal);
            let r_dot_v   = v3_dot(refl_dir, view_dir);
            if r_dot_v > 0.0 {
                let norm = r_dot_v / (v3_len(refl_dir) * v3_len(ray_dir));
                total += libm::powf(norm, specular);
            }
        }
    }

    total
}

fn reflect_ray(ray: Vec3, normal: Vec3) -> Vec3 {
    v3_sub(v3_scale(normal, 2.0 * v3_dot(normal, ray)), ray)
}

// ---------------------------------------------------------------------------
// Textura
// ---------------------------------------------------------------------------

fn sample_texture(texture: &Texture, hit_pos: Vec3, sphere: &Sphere) -> (f32, f32, f32) {
    match texture {
        Texture::Color(r, g, b) => (*r, *g, *b),

        Texture::CheckerXZ { color1, color2, scale } => {
            let s05 = scale / 2.0;
            let s2  = scale * 2.0;
            let x   = hit_pos[0] - s05;
            let z   = hit_pos[2] - s05;
            let xt  = (libm::fmodf(x.abs(), s2) >= *scale) ^ (x < 0.0);
            let zt  = (libm::fmodf(z.abs(), s2) >= *scale) ^ (z < 0.0);
            if xt ^ zt { *color2 } else { *color1 }
        }

        Texture::GradientY { color1, color2 } => {
            let mut y = (hit_pos[1] - sphere.pos[1]) / sphere.r;
            y = y.max(-1.0).min(1.0);
            let ny = 1.0 - y;
            (
                color1.0 * y + color2.0 * ny,
                color1.1 * y + color2.1 * ny,
                color1.2 * y + color2.2 * ny,
            )
        }
    }
}