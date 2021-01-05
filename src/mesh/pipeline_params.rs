use crate::
{
    wgpu_state::WgpuState,
    mesh::Pipeline,
    transform::Transform2D,
};
use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct Uniforms {
    model_matrix: [f32; 16],
}

impl Uniforms {
    fn new(model_matrix: &glam::Mat4) -> Self {
        Self {
            model_matrix: model_matrix.to_cols_array(),
        }
    }

    fn set_model_matrix(&mut self, model_matrix: &glam::Mat4) {
        self.model_matrix = model_matrix.to_cols_array();
    }
}

pub struct PipelineParams {
    uniforms: Uniforms,
    uniform_buffer: wgpu::Buffer,
    pub uniform_bind_group: wgpu::BindGroup,
}

impl PipelineParams {
    pub fn new(wgpu_state: &WgpuState, pipeline: &Pipeline, model_matrix: &glam::Mat4) -> Self {
        let uniforms = Uniforms::new(model_matrix);

        let uniform_buffer = wgpu_state.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Uniform Buffer"),
            contents: bytemuck::cast_slice(&[uniforms]),
            usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
        });

        let uniform_bind_group = wgpu_state.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &pipeline.mesh_uniform_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(uniform_buffer.slice(..)),
            }],
            label: Some("uniform_bind_group"),
        });

        Self {
            uniforms,
            uniform_buffer,
            uniform_bind_group,
        }
    }

    pub fn update_from_transform(&mut self, transform: &Transform2D, queue: &wgpu::Queue) {
        self.uniforms.set_model_matrix(&transform.build_matrix());

        queue.write_buffer(
            &self.uniform_buffer,
            0,
            bytemuck::cast_slice(&[self.uniforms]),
        );
    }
}
