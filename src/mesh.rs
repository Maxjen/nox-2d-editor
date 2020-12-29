use wgpu::util::DeviceExt;
use crate::{
    wgpu_state::WgpuState,
    mesh_pipeline::MeshPipeline,
    loader::MeshData,
    asset::{Handle, Assets},
    texture::Texture,
};

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    position: [f32; 3],
    tex_coords: [f32; 2],
}

impl Vertex {
    pub fn desc<'a>() -> wgpu::VertexBufferDescriptor<'a> {
        use std::mem;
        wgpu::VertexBufferDescriptor {
            stride: mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::InputStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttributeDescriptor {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float3,
                },
                wgpu::VertexAttributeDescriptor {
                    offset: mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float2,
                },
            ],
        }
    }
}

#[derive(Debug)]
pub struct Mesh {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub num_indices: u32,
    pub texture: Handle<Texture>,
    pub texture_bind_group: wgpu::BindGroup,
}

impl Mesh {
    pub fn new(
        state: &WgpuState,
        pipeline: &MeshPipeline,
        textures: &mut Assets<Texture>,
        mesh_data: &MeshData
    ) -> Self {
        let mut vertex_data = vec![0.0; mesh_data.vertices.len() * 5];
        for i in 0..mesh_data.vertices.len() {
            vertex_data[i * 5] = mesh_data.vertices[i].x;
            vertex_data[i * 5 + 1] = mesh_data.vertices[i].y;
            vertex_data[i * 5 + 2] = 0.0;
            vertex_data[i * 5 + 3] = mesh_data.vertices[i].u;
            vertex_data[i * 5 + 4] = 1.0 - mesh_data.vertices[i].v;
        }
        let vertex_buffer = state.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertex_data),
            usage: wgpu::BufferUsage::VERTEX,
        });
        let index_buffer = state.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(&mesh_data.triangles),
            usage: wgpu::BufferUsage::INDEX,
        });
        let num_indices = mesh_data.triangles.len() as u32 * 3;

        let texture = {
            let texture_path = std::path::Path::new(env!("OUT_DIR")).join("data").join(mesh_data.texture.clone());
            Texture::get_or_load(textures, &state.device, &state.queue, texture_path).unwrap()
        };

        let texture_bind_group = {
            let texture = textures.get(&texture).unwrap();
            state.device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &pipeline.texture_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&texture.view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&texture.sampler),
                    },
                ],
                label: Some("diffuse_bind_group"),
            })
        };

        Self {
            vertex_buffer,
            index_buffer,
            num_indices,
            texture,
            texture_bind_group,
        }
    }
}
