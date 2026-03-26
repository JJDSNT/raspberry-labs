#![allow(dead_code)]

use crate::gpu::backend::{GpuBackend, GpuError};
use crate::gfx3d::command::{GpuCommand, Vertex};

pub struct GfxContext<B: GpuBackend> {
    backend: B,
    frame_width: u32,
    frame_height: u32,
}

impl<B: GpuBackend> GfxContext<B> {
    pub fn new(backend: B, frame_width: u32, frame_height: u32) -> Self {
        Self {
            backend,
            frame_width,
            frame_height,
        }
    }

    pub fn init(&mut self) -> Result<(), GpuError> {
        self.backend.init()
    }

    pub fn begin_frame(&mut self) -> Result<(), GpuError> {
        self.backend.begin_frame(self.frame_width, self.frame_height)
    }

    pub fn submit(&mut self, commands: &[GpuCommand]) -> Result<B::Fence, GpuError> {
        self.backend.submit(commands)
    }

    pub fn end_frame(&mut self) -> Result<(), GpuError> {
        self.backend.end_frame()
    }

    pub fn wait(&mut self, fence: &B::Fence) -> Result<(), GpuError> {
        self.backend.wait(fence)
    }

    pub fn backend(&self) -> &B {
        &self.backend
    }

    pub fn backend_mut(&mut self) -> &mut B {
        &mut self.backend
    }

    pub fn draw_immediate_triangle(
        &mut self,
        v0: Vertex,
        v1: Vertex,
        v2: Vertex,
    ) -> Result<(), GpuError> {
        self.backend.draw_triangle(&[v0, v1, v2])
    }
}