#![allow(dead_code)]

use crate::gpu::backend::{GpuBackend, GpuError};
use crate::gfx3d::command::{GpuCommand, Vertex};
use crate::gfx3d::context::GfxContext;

pub struct GraphicsApi<B: GpuBackend> {
    ctx: GfxContext<B>,
}

impl<B: GpuBackend> GraphicsApi<B> {
    pub fn new(ctx: GfxContext<B>) -> Self {
        Self { ctx }
    }

    pub fn init(&mut self) -> Result<(), GpuError> {
        self.ctx.init()
    }

    pub fn clear(&mut self, rgba8: u32) -> Result<(), GpuError> {
        self.ctx.backend_mut().clear(rgba8)
    }

    pub fn draw_triangle(
        &mut self,
        v0: Vertex,
        v1: Vertex,
        v2: Vertex,
    ) -> Result<(), GpuError> {
        self.ctx.draw_immediate_triangle(v0, v1, v2)
    }

    pub fn submit(&mut self, commands: &[GpuCommand]) -> Result<B::Fence, GpuError> {
        self.ctx.submit(commands)
    }

    pub fn begin_frame(&mut self) -> Result<(), GpuError> {
        self.ctx.begin_frame()
    }

    pub fn end_frame(&mut self) -> Result<(), GpuError> {
        self.ctx.end_frame()
    }

    pub fn wait(&mut self, fence: &B::Fence) -> Result<(), GpuError> {
        self.ctx.wait(fence)
    }

    pub fn context(&self) -> &GfxContext<B> {
        &self.ctx
    }

    pub fn context_mut(&mut self) -> &mut GfxContext<B> {
        &mut self.ctx
    }
}