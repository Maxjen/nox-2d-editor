use std::iter;

use legion::*;
use legion::world::SubWorld;
use wgpu::util::DeviceExt;
use crate::{
    wgpu_state::WgpuState,
    mesh,
    camera::Camera,
};

/*#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: glam::Mat4 = glam::Mat4::from_cols_array([
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.0,
    0.0, 0.0, 0.5, 1.0,
]);*/

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct Uniforms {
    view_proj: [f32; 16],
}

impl Uniforms {
    fn new() -> Self {
        Self {
            view_proj: glam::Mat4::identity().to_cols_array(),
        }
    }

    fn update_view_proj(&mut self, camera: &Camera) {
        //#[rustfmt::skip]
        /*let opengl_to_wgpu: glam::Mat4 = glam::Mat4::from_cols_array(&[
            1.0, 0.0, 0.0, 0.0,
            0.0, 1.0, 0.0, 0.0,
            0.0, 0.0, 0.5, 0.0,
            0.0, 0.0, 0.5, 1.0,
        ]);*/
        self.view_proj = (/*opengl_to_wgpu * */camera.build_view_projection_matrix()).to_cols_array();
    }
}

pub struct MeshPipeline {
    render_pipeline: wgpu::RenderPipeline,
    pub texture_bind_group_layout: wgpu::BindGroupLayout,
    uniforms: Uniforms,
    uniform_buffer: wgpu::Buffer,
    uniform_bind_group: wgpu::BindGroup,
}

impl MeshPipeline {
    pub fn new(wgpu_state: &WgpuState) -> Self {
        let texture_bind_group_layout =
            wgpu_state.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStage::FRAGMENT,
                        ty: wgpu::BindingType::SampledTexture {
                            multisampled: false,
                            dimension: wgpu::TextureViewDimension::D2,
                            component_type: wgpu::TextureComponentType::Uint,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStage::FRAGMENT,
                        ty: wgpu::BindingType::Sampler { comparison: false },
                        count: None,
                    },
                ],
                label: Some("texture_bind_group_layout"),
            });
        
        let uniforms = Uniforms::new();
        //uniforms.update_view_proj(&camera);

        let uniform_buffer = wgpu_state.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Uniform Buffer"),
            contents: bytemuck::cast_slice(&[uniforms]),
            usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
        });

        let uniform_bind_group_layout =
            wgpu_state.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStage::VERTEX,
                    ty: wgpu::BindingType::UniformBuffer {
                        dynamic: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("uniform_bind_group_layout"),
            });

        let uniform_bind_group = wgpu_state.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &uniform_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(uniform_buffer.slice(..)),
            }],
            label: Some("uniform_bind_group"),
        });

        let vs_module = wgpu_state.device.create_shader_module(wgpu::include_spirv!("../shaders/shader.vert.spv"));
        let fs_module = wgpu_state.device.create_shader_module(wgpu::include_spirv!("../shaders/shader.frag.spv"));

        let render_pipeline_layout = wgpu_state.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[&texture_bind_group_layout, &uniform_bind_group_layout],
            push_constant_ranges: &[],
        });

        let render_pipeline = wgpu_state.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex_stage: wgpu::ProgrammableStageDescriptor {
                module: &vs_module,
                entry_point: "main",
            },
            fragment_stage: Some(wgpu::ProgrammableStageDescriptor {
                module: &fs_module,
                entry_point: "main",
            }),
            rasterization_state: Some(wgpu::RasterizationStateDescriptor {
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: wgpu::CullMode::Back,
                depth_bias: 0,
                depth_bias_slope_scale: 0.0,
                depth_bias_clamp: 0.0,
                clamp_depth: false,
            }),
            primitive_topology: wgpu::PrimitiveTopology::TriangleList,
            color_states: &[wgpu::ColorStateDescriptor {
                format: wgpu_state.sc_desc.format,
                //color_blend: wgpu::BlendDescriptor::REPLACE,
                color_blend: wgpu::BlendDescriptor {
                    src_factor: wgpu::BlendFactor::SrcAlpha,
                    dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                    operation: wgpu::BlendOperation::Add,
                },
                alpha_blend: wgpu::BlendDescriptor::REPLACE,
                write_mask: wgpu::ColorWrite::ALL,
            }],
            depth_stencil_state: None,
            vertex_state: wgpu::VertexStateDescriptor {
                index_format: wgpu::IndexFormat::Uint32,
                vertex_buffers: &[mesh::Vertex::desc()],
            },
            sample_count: 1,
            sample_mask: !0,
            alpha_to_coverage_enabled: false,
        });

        Self {
            render_pipeline,
            texture_bind_group_layout,
            uniforms,
            uniform_buffer,
            uniform_bind_group,
        }
    }
}

#[system]
pub fn update_camera_buffer(
    #[resource] state: &mut WgpuState,
    #[resource] pipeline: &mut MeshPipeline,
    #[resource] camera: &Camera,
) {
    pipeline.uniforms.update_view_proj(camera);
    state.queue.write_buffer(
        &pipeline.uniform_buffer,
        0,
        bytemuck::cast_slice(&[pipeline.uniforms]),
    );
}

#[system]
#[read_component(mesh::Mesh)]
pub fn render_meshes(world: &mut SubWorld, #[resource] state: &mut WgpuState, #[resource] pipeline: &MeshPipeline) {
    let frame = {
        let frame_result = state.swap_chain.get_current_frame();
        if frame_result.is_err() {
            state.render_result = Err(frame_result.err().unwrap());
            //return Ok(());
            return;
        }
        //self.swap_chain.get_current_frame().unwrap().output
        frame_result.unwrap().output
    };

    let mut encoder = state
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

    {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                attachment: &frame.view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.1,
                        g: 0.2,
                        b: 0.3,
                        a: 1.0,
                    }),
                    store: true,
                },
            }],
            depth_stencil_attachment: None,
        });

        render_pass.set_pipeline(&pipeline.render_pipeline);
        render_pass.set_bind_group(1, &pipeline.uniform_bind_group, &[]);

        let mut query = <&mesh::Mesh>::query();
        for mesh in query.iter(world) {
            render_pass.set_bind_group(0, &mesh.texture_bind_group, &[]);
            render_pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
            render_pass.set_index_buffer(mesh.index_buffer.slice(..));
            render_pass.draw_indexed(0..mesh.num_indices, 0, 0..1);
        }
    }

    state.queue.submit(iter::once(encoder.finish()));

    state.render_result = Ok(());
}
