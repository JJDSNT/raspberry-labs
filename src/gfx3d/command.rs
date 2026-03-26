#![allow(dead_code)]

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vertex {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub color: u32,
}

impl Vertex {
    pub const fn new(x: f32, y: f32, z: f32, color: u32) -> Self {
        Self { x, y, z, color }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrimitiveType {
    Triangles,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GpuCommand {
    ClearColor {
        rgba8: u32,
    },
    DrawTriangle {
        v0: Vertex,
        v1: Vertex,
        v2: Vertex,
    },
}