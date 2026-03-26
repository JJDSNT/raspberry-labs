#![allow(dead_code)]

use crate::gfx3d::command::{GpuCommand, Vertex};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GpuError {
    Unsupported,
    InvalidState,
    InvalidParameter,
    OutOfMemory,
    HardwareFault,
}

pub trait GpuBackend {
    type Buffer;
    type Pipeline;
    type Fence;

    fn init(&mut self) -> Result<(), GpuError>;

    fn begin_frame(&mut self, width: u32, height: u32) -> Result<(), GpuError>;

    fn clear(&mut self, rgba8: u32) -> Result<(), GpuError>;

    fn draw_triangle(&mut self, vertices: &[Vertex]) -> Result<(), GpuError>;

    fn submit(&mut self, commands: &[GpuCommand]) -> Result<Self::Fence, GpuError>;

    fn end_frame(&mut self) -> Result<(), GpuError>;

    fn wait(&mut self, fence: &Self::Fence) -> Result<(), GpuError>;
}