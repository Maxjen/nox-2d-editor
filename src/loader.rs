//use serde::{Serialize, Deserialize};
//use ron::ser::{to_string_pretty, PrettyConfig};
//use std::fs::File;
//use std::io::prelude::*;
use ron::de::from_reader;
use serde::{Serialize, Deserialize};
use std::fs::File;

#[derive(Default, Serialize, Deserialize, Debug)]
#[serde(default)]
pub struct Scene {
    pub meshes: Vec<MeshData>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MeshData {
    mesh_name: String,
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

pub fn load_ron() -> Scene {
    let file = File::open("data/meshes.ron").unwrap();
    let scene: Scene = match from_reader(file) {
        Ok(x) => x,
        Err(e) => {
            println!("Failed to load ron: {}", e);
            std::process::exit(1);
        }
    };
    scene
}

/*fn main() -> std::io::Result<()> {
    let mut file = File::open("data/meshes.ron")?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;
    let deserialized: Scene = serde_json::from_str(&content)?;
    println!("{:?}", deserialized.meshes[0]);

    let mut file2 = File::create("data/meshes.ron").unwrap();
    let pretty = PrettyConfig::default();
    let ron_string = to_string_pretty(&deserialized, pretty).unwrap();
    file2.write(ron_string.as_bytes()).unwrap();

    Ok(())
}*/
