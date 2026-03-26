#![allow(dead_code)]

use crate::gpu::backend::{GpuBackend, GpuError};
use crate::gfx3d::command::{GpuCommand, Vertex};

use super::framebuffer::SoftFramebuffer;
use super::rasterizer::SoftwareRasterizer;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SoftBufferHandle(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SoftPipelineHandle(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SoftFence {
    pub frame_id: u64,
}

pub struct SoftwareBackend<'a> {
    width: u32,
    height: u32,
    rasterizer: SoftwareRasterizer<'a>,
    initialized: bool,
    frame_active: bool,
    current_frame: u64,
}

impl<'a> SoftwareBackend<'a> {
    pub fn new(width: u32, height: u32, pixels: &'a mut [u32]) -> Self {
        let fb = SoftFramebuffer::new(width as usize, height as usize, pixels);
        let rasterizer = SoftwareRasterizer::new(fb);

        Self {
            width,
            height,
            rasterizer,
            initialized: false,
            frame_active: false,
            current_frame: 0,
        }
    }

    pub fn framebuffer(&self) -> &SoftFramebuffer<'a> {
        self.rasterizer.framebuffer()
    }

    pub fn framebuffer_mut(&mut self) -> &mut SoftFramebuffer<'a> {
        self.rasterizer.framebuffer_mut()
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    fn ensure_initialized(&self) -> Result<(), GpuError> {
        if !self.initialized {
            return Err(GpuError::InvalidState);
        }
        Ok(())
    }

    fn ensure_frame_active(&self) -> Result<(), GpuError> {
        if !self.frame_active {
            return Err(GpuError::InvalidState);
        }
        Ok(())
    }

    fn execute_command(&mut self, cmd: &GpuCommand) -> Result<(), GpuError> {
        match *cmd {
            GpuCommand::ClearColor { rgba8 } => {
                self.clear(rgba8)?;
            }
            GpuCommand::DrawTriangle { v0, v1, v2 } => {
                let vertices = [v0, v1, v2];
                self.draw_triangle(&vertices)?;
            }
        }
        Ok(())
    }
}

impl<'a> GpuBackend for SoftwareBackend<'a> {
    type Buffer = SoftBufferHandle;
    type Pipeline = SoftPipelineHandle;
    type Fence = SoftFence;

    fn init(&mut self) -> Result<(), GpuError> {
        self.initialized = true;
        Ok(())
    }

    fn begin_frame(&mut self, width: u32, height: u32) -> Result<(), GpuError> {
        self.ensure_initialized()?;

        if width != self.width || height != self.height {
            return Err(GpuError::InvalidParameter);
        }

        if self.frame_active {
            return Err(GpuError::InvalidState);
        }

        self.frame_active = true;
        Ok(())
    }

    fn clear(&mut self, rgba8: u32) -> Result<(), GpuError> {
        self.ensure_initialized()?;
        self.ensure_frame_active()?;

        self.rasterizer.clear(rgba8);
        Ok(())
    }

    fn draw_triangle(&mut self, vertices: &[Vertex]) -> Result<(), GpuError> {
        self.ensure_initialized()?;
        self.ensure_frame_active()?;

        if vertices.len() != 3 {
            return Err(GpuError::InvalidParameter);
        }

        self.rasterizer
            .draw_triangle(vertices[0], vertices[1], vertices[2]);

        Ok(())
    }

    fn submit(&mut self, commands: &[GpuCommand]) -> Result<Self::Fence, GpuError> {
        self.ensure_initialized()?;
        self.ensure_frame_active()?;

        for cmd in commands {
            self.execute_command(cmd)?;
        }

        self.current_frame = self.current_frame.wrapping_add(1);

        Ok(SoftFence {
            frame_id: self.current_frame,
        })
    }

    fn end_frame(&mut self) -> Result<(), GpuError> {
        self.ensure_initialized()?;

        if !self.frame_active {
            return Err(GpuError::InvalidState);
        }

        self.frame_active = false;
        Ok(())
    }

    fn wait(&mut self, fence: &Self::Fence) -> Result<(), GpuError> {
        self.ensure_initialized()?;

        if fence.frame_id > self.current_frame {
            return Err(GpuError::InvalidParameter);
        }

        Ok(())
    }
}