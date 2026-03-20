// src/demos/juggler.rs
//
// Juggler — port bare metal do clássico demo Amiga (Eric Graham, 1986).
// Portado de scene_juggler.rs / juggler-in-rust v0.2.0.
//
// Arquitetura:
//   - JugglerDemo implementa o trait Demo
//   - A cena é construída em arrays estáticos a cada frame (sem alloc)
//   - O raytracer em math/raytracer.rs traça cada pixel
//   - Frame renderizado pixel a pixel no back_buffer do renderer
//
// Quando os cores adicionais estiverem disponíveis, dividir o buffer
// em N fatias horizontais e passar uma para cada core.

use crate::demos::Demo;
use crate::gfx::renderer::Renderer;
use crate::math::raytracer::{
    color_to_argb, trace_ray, v3_add, v3_len, v3_scale, v3_sub,
    Camera, Light, Scene, Sphere, Texture, Vec3,
};

const MAX_SPHERES: usize = 128;
const MAX_LIGHTS:  usize = 4;
const REFLECT_DEPTH: usize = 3;

const SKY_COLOR:     (f32, f32, f32) = (0.1, 0.1, 1.0);
const BOUNCE_CYCLE:  f32 = 1.0;
const CAMERA_CYCLE:  f32 = 15.0;

// ---------------------------------------------------------------------------
// Estado do demo
// ---------------------------------------------------------------------------

pub struct JugglerDemo {
    frame:     u32,
    fps:       f32,
    camera:    Camera,
    spheres:   [Sphere; MAX_SPHERES],
    n_spheres: usize,
    lights:    [Light;  MAX_LIGHTS],
    n_lights:  usize,
}

impl JugglerDemo {
    pub fn new() -> Self {
        let dummy_sphere = Sphere {
            pos: [0.0,0.0,0.0], r: 0.0,
            texture: Texture::Color(0.0,0.0,0.0),
            specular: -1.0, reflective: 0.0, skip_lighting: false,
        };
        let dummy_light = Light::Ambient { intensity: 0.0 };
        let dummy_camera = Camera {
            pos:     [0.0, 4.0, -10.0],
            right:   [1.0, 0.0,   0.0],
            up:      [0.0, 1.0,   0.0],
            forward: [0.0, 0.0,   1.0],
        };

        Self {
            frame:     0,
            fps:       24.0,
            camera:    dummy_camera,
            spheres:   [dummy_sphere; MAX_SPHERES],
            n_spheres: 0,
            lights:    [dummy_light; MAX_LIGHTS],
            n_lights:  0,
        }
    }

    // -----------------------------------------------------------------------
    // Construção da cena — portado de populate_scene()
    // -----------------------------------------------------------------------

    fn build_scene(&mut self, secs: f32) {
        self.n_spheres = 0;
        self.n_lights  = 0;

        let bounce_phase   = (secs % BOUNCE_CYCLE) / BOUNCE_CYCLE;
        let body_bounce    = 0.15 * libm::sinf(bounce_phase * core::f32::consts::TAU);
        let body_bounce_90 = 0.15 * libm::cosf(bounce_phase * core::f32::consts::TAU);

        // Materiais protótipo
        let juggling = mat(Texture::Color(0.9,0.9,0.9), 100.0, 0.8, false);
        let body     = mat(Texture::Color(1.0,0.1,0.1), 100.0, 0.0, false);
        let skin     = mat(Texture::Color(1.0,0.7,0.7), 100.0, 0.0, false);
        let hair     = mat(Texture::Color(0.2,0.1,0.1), 100.0, 0.0, false);
        let eye      = mat(Texture::Color(0.1,0.1,1.0), 100.0, 0.0, false);

        // Chão
        self.push(Sphere {
            pos: [0.0,-5000.0,0.0], r: 5000.0,
            texture: Texture::CheckerXZ {
                color1: (1.0,1.0,0.0), color2: (0.0,1.0,0.0), scale: 4.0,
            },
            specular: -1.0, reflective: 0.0, skip_lighting: false,
        });

        // Céu
        self.push(Sphere {
            pos: [0.0,0.0,0.0], r: 10000.0,
            texture: Texture::GradientY {
                color1: (0.1,0.1,1.0), color2: (0.7,0.7,1.0),
            },
            specular: -1.0, reflective: 0.0, skip_lighting: true,
        });

        // Cabeça
        self.push(mk(&skin, [0.0,  6.1+body_bounce,  0.2+body_bounce_90], 0.5));
        self.push(mk(&hair, [0.0,  6.12+body_bounce, 0.22+body_bounce_90], 0.5));
        self.push(mk(&skin, [0.0,  5.5+body_bounce,  0.2+body_bounce_90], 0.2));
        self.push(mk(&eye,  [-0.2, 6.1+body_bounce,  -0.2+body_bounce_90], 0.15));
        self.push(mk(&eye,  [ 0.2, 6.1+body_bounce,  -0.2+body_bounce_90], 0.15));

        // Corpo
        self.line(
            mk(&body, [0.0, 4.6+body_bounce, 0.2+body_bounce_90], 0.8),
            mk(&body, [0.0, 3.3+body_bounce, 0.0], 0.6),
            8, true,
        );

        let lh: Vec3 = [-2.0, 3.1, -1.0];
        let rh: Vec3 = [ 1.9, 3.8, -1.0];

        // Braço esquerdo
        let ls = mk(&skin, [-0.7, 5.1+body_bounce, 0.2+body_bounce_90], 0.2);
        let le = mk(&skin, [-1.2+body_bounce/1.4, 4.2+body_bounce, -0.2+body_bounce_90], 0.2);
        let lw = mk(&skin, v3_add(lh, [-body_bounce_90*1.5, body_bounce, body_bounce_90]), 0.1);
        self.line(ls, le, 9, false);
        self.line(le, lw, 8, true);

        // Braço direito
        let rs = mk(&skin, [0.7, 5.1+body_bounce, 0.2+body_bounce_90], 0.2);
        let re = mk(&skin, [1.2+body_bounce/1.4, 4.2+body_bounce, -0.2+body_bounce_90], 0.2);
        let rw = mk(&skin, v3_add(rh, [body_bounce_90*1.5, body_bounce, body_bounce_90]), 0.1);
        self.line(rs, re, 9, false);
        self.line(re, rw, 8, true);

        // Perna esquerda
        self.line(mk(&skin, [-0.6, 2.9+body_bounce, 0.0], 0.2),
                  mk(&skin, [-0.7, 1.6+body_bounce/2.0, -0.6+body_bounce/1.4], 0.2), 8, false);
        self.line(mk(&skin, [-0.7, 1.6+body_bounce/2.0, -0.6+body_bounce/1.4], 0.2),
                  mk(&skin, [-0.6, 0.0, 0.0], 0.1), 8, true);

        // Perna direita
        self.line(mk(&skin, [0.6, 2.9+body_bounce, 0.0], 0.2),
                  mk(&skin, [0.7, 1.6+body_bounce/2.0, -0.6+body_bounce/1.4], 0.2), 8, false);
        self.line(mk(&skin, [0.7, 1.6+body_bounce/2.0, -0.6+body_bounce/1.4], 0.2),
                  mk(&skin, [0.6, 0.0, 0.0], 0.1), 8, true);

        // Bolas de malabarismo
        let diff = v3_sub(rh, lh);

        let p1 = bounce_phase;
        let mut b1 = v3_add(lh, v3_scale(diff, p1));
        b1[1] += 2.1 * libm::sinf(p1 * core::f32::consts::PI) + 0.4;
        b1[2] -= 0.3;
        self.push(mk(&juggling, b1, 0.6));

        let p2 = bounce_phase / 2.0;
        let mut b2 = v3_add(rh, v3_scale(diff, -p2));
        b2[1] += 4.2 * libm::sinf(p2 * core::f32::consts::PI) + 0.4;
        b2[2] -= 0.3;
        self.push(mk(&juggling, b2, 0.6));

        let p3 = bounce_phase / 2.0 + 0.5;
        let mut b3 = v3_add(rh, v3_scale(diff, -p3));
        b3[1] += 4.2 * libm::sinf(p3 * core::f32::consts::PI) + 0.4;
        b3[2] -= 0.3;
        self.push(mk(&juggling, b3, 0.6));

        // Luzes
        self.push_light(Light::Ambient { intensity: 0.45 });
        self.push_light(Light::Point   { intensity: 0.55, pos: [50.0, 150.0, -100.0] });

        // Câmera orbital
        let cam_phase = (secs % CAMERA_CYCLE) / CAMERA_CYCLE;
        let cam_angle = cam_phase * core::f32::consts::TAU;
        let dist      = 10.0f32;

        let mut camera = Camera {
            pos:     [dist * libm::sinf(cam_angle), 4.0, -dist * libm::cosf(cam_angle)],
            right:   [1.0, 0.0, 0.0],
            up:      [0.0, 1.0, 0.0],
            forward: [0.0, 0.0, 1.0],
        };
        camera.look_at([0.0, 4.0, 0.0]);
        self.camera = camera;
    }

    // -----------------------------------------------------------------------
    // Renderização — pixel a pixel
    // -----------------------------------------------------------------------

    fn render_scene(&self, renderer: &mut Renderer) {
        let w = renderer.width();
        let h = renderer.height();
        if w == 0 || h == 0 { return; }

        let scene = Scene {
            camera:    self.camera,
            spheres:   &self.spheres[..self.n_spheres],
            lights:    &self.lights[..self.n_lights],
            sky_color: SKY_COLOR,
        };

        let buf = renderer.back_buffer();

        for y in 0..h {
            for x in 0..w {
                let vx =  (x as f32 / (w - 1) as f32) - 0.5;
                let vy = 0.5 - (y as f32 / (h - 1) as f32);

                let ray_dir = v3_add(
                    v3_add(scene.camera.forward, v3_scale(scene.camera.right, vx)),
                    v3_scale(scene.camera.up, vy),
                );

                let t_min = v3_len(ray_dir);
                let (r, g, b) = trace_ray(
                    &scene, scene.camera.pos, ray_dir,
                    t_min, f32::INFINITY, REFLECT_DEPTH,
                );

                buf[y * w + x] = color_to_argb(r, g, b);
            }
        }
    }

    // -----------------------------------------------------------------------
    // Helpers internos
    // -----------------------------------------------------------------------

    #[inline]
    fn push(&mut self, s: Sphere) {
        if self.n_spheres < MAX_SPHERES {
            self.spheres[self.n_spheres] = s;
            self.n_spheres += 1;
        }
    }

    #[inline]
    fn push_light(&mut self, l: Light) {
        if self.n_lights < MAX_LIGHTS {
            self.lights[self.n_lights] = l;
            self.n_lights += 1;
        }
    }

    fn line(&mut self, start: Sphere, end: Sphere, n: usize, inclusive: bool) {
        let last = n - 1;
        let dir  = v3_sub(end.pos, start.pos);
        let dr   = end.r - start.r;
        for i in 0..=last {
            if !inclusive && i == last { break; }
            let t = i as f32 / last as f32;
            let mut s = start;
            s.pos = v3_add(start.pos, v3_scale(dir, t));
            s.r   = start.r + dr * t;
            self.push(s);
        }
    }
}

impl Demo for JugglerDemo {
    fn render(&mut self, renderer: &mut Renderer) {
        if self.frame == 0 {
            crate::log!("JUGGLER", "first frame w={} h={}", renderer.width(), renderer.height());
        }
        if self.frame % 10 == 0 {
            crate::log!("JUGGLER", "frame={}", self.frame);
        }

        let secs = self.frame as f32 / self.fps;
        self.build_scene(secs);

        if self.frame == 0 {
            crate::log!("JUGGLER", "scene built spheres={} lights={}", self.n_spheres, self.n_lights);
        }

        self.render_scene(renderer);

        if self.frame == 0 {
            crate::log!("JUGGLER", "first frame done");
        }

        self.frame = self.frame.wrapping_add(1);
    }
}

// ---------------------------------------------------------------------------
// Helpers livres
// ---------------------------------------------------------------------------

/// Cria um material protótipo (pos e r serão definidos depois com mk).
#[inline]
fn mat(texture: Texture, specular: f32, reflective: f32, skip_lighting: bool) -> Sphere {
    Sphere { pos: [0.0,0.0,0.0], r: 0.0, texture, specular, reflective, skip_lighting }
}

/// Clona um protótipo com posição e raio específicos.
#[inline]
fn mk(proto: &Sphere, pos: Vec3, r: f32) -> Sphere {
    let mut s = *proto; s.pos = pos; s.r = r; s
}