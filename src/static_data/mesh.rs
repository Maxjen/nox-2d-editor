use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct MeshData {
    pub mesh_name: String,
    pub texture: String,
    pub vertices: Vec<Vertex>,

    #[serde(default)]
    lines: Vec<Line>,

    #[serde(default)]
    pub triangles: Vec<Triangle>,

    #[serde(default)]
    quads: Vec<Quad>,
}

#[repr(C)]
#[derive(Serialize, Deserialize, Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub x: f32,
    pub y: f32,
    pub u: f32,
    pub v: f32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Line {
    v1: u32,
    v2: u32,
}

#[repr(C)]
#[derive(Serialize, Deserialize, Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Triangle {
    v1: u32,
    v2: u32,
    v3: u32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Quad {
    v1: u32,
    v2: u32,
    v3: u32,
    v4: u32,
}
