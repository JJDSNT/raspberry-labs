// src/demos/gfx3d_triangle.rs

#![allow(dead_code)]

use crate::demos::Demo;
use crate::gfx::renderer::Renderer;
use crate::gpu::backend::{GpuBackend, GpuError};
use crate::gpu::soft::backend::SoftwareBackend;
use crate::gfx3d::command::Vertex;
use crate::math::math3d::{Camera, Mat4, Vec3, project_world_point};
use crate::media::FrameContext;

pub struct Gfx3dTriangleDemo {
    angle: f32,
}

impl Gfx3dTriangleDemo {
    pub fn new() -> Self {
        Self { angle: 0.0 }
    }

    fn render_inner(
        &mut self,
        renderer: &mut Renderer,
        _frame: &FrameContext,
    ) -> Result<(), GpuError> {
        let width = renderer.width();
        let height = renderer.height();

        if width == 0 || height == 0 {
            return Ok(());
        }

        let pixels: &mut [u32] = renderer.back_buffer();
        let mut backend = SoftwareBackend::new(width as u32, height as u32, pixels);

        backend.init()?;
        backend.begin_frame(width as u32, height as u32)?;
        backend.clear(0x081020ff)?;

        self.angle += 0.03;
        if self.angle > core::f32::consts::PI * 2.0 {
            self.angle -= core::f32::consts::PI * 2.0;
        }

        let model =
            Mat4::translation(0.0, 0.0, 4.0)
            * Mat4::rotation_xyz(self.angle * 0.7, self.angle, self.angle * 0.25);

        let camera = Camera::new(
            Vec3::new(0.0, 0.0, 0.0),  // posição
            Vec3::new(0.0, 0.0, 1.0),  // olhando para frente
            Vec3::new(0.0, 1.0, 0.0),  // up
            core::f32::consts::PI / 3.0,
            width as f32 / height as f32,
            0.1,
            100.0,
        );

        let view = camera.view_matrix();
        let proj = camera.projection_matrix();

        let p0 = project_world_point(
            Vec3::new(-1.0, -1.0, 0.0),
            model,
            view,
            proj,
            width,
            height,
        );

        let p1 = project_world_point(
            Vec3::new(1.0, -1.0, 0.0),
            model,
            view,
            proj,
            width,
            height,
        );

        let p2 = project_world_point(
            Vec3::new(0.0, 1.0, 0.0),
            model,
            view,
            proj,
            width,
            height,
        );

        if p0.visible && p1.visible && p2.visible {
            let vertices = [
                Vertex::new(p0.screen.x, p0.screen.y, p0.depth, 0xff0000ff),
                Vertex::new(p1.screen.x, p1.screen.y, p1.depth, 0x00ff00ff),
                Vertex::new(p2.screen.x, p2.screen.y, p2.depth, 0x0000ffff),
            ];

            backend.draw_triangle(&vertices)?;
        }

        backend.end_frame()?;
        Ok(())
    }
}

impl Demo for Gfx3dTriangleDemo {
    fn render(&mut self, renderer: &mut Renderer, frame: &FrameContext) {
        let _ = self.render_inner(renderer, frame);
    }
}