use legion::*;
use legion::systems::CommandBuffer;
use crate::{
    loader,
    loader::Scene,
    asset::Assets,
    mesh::Mesh,
    events::Events,
    wgpu_state::WgpuState,
    mesh_pipeline::MeshPipeline,
    texture::Texture,
};
use winit::event::{ElementState, KeyboardInput, VirtualKeyCode};

pub struct AppState {
    scene: Scene,
    mesh_entities: Vec<Entity>,
    current_mesh_index: Option<usize>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            scene: loader::load_ron(),
            mesh_entities: Vec::new(),
            current_mesh_index: None,
        }
    }
    
    pub fn remove_mesh_entities(&mut self, command_buffer: &mut CommandBuffer) {
        for entity in self.mesh_entities.drain(..) {
            command_buffer.remove(entity);
        }
    }

    pub fn set_current_mesh_index(
        &mut self, mesh_index: Option<usize>,
        wgpu_state: &mut WgpuState,
        pipeline: &MeshPipeline,
        textures: &mut Assets<Texture>,
        command_buffer: &mut CommandBuffer
    ) {
        if self.current_mesh_index == mesh_index { return; }
        self.current_mesh_index = mesh_index;
        info!("Set current mesh index to {:?}", self.current_mesh_index);

        self.remove_mesh_entities(command_buffer);

        if let Some(index) = self.current_mesh_index {
            let mesh = Mesh::new(wgpu_state, pipeline, textures, &self.scene.meshes[index]);
            let entity = command_buffer.push((mesh,));
            self.mesh_entities.push(entity);
        }
    }
}

#[system]
pub fn handle_input(
    command_buffer: &mut CommandBuffer,
    #[resource] app_state: &mut AppState,
    #[resource] wgpu_state: &mut WgpuState,
    #[resource] pipeline: &MeshPipeline,
    #[resource] textures: &mut Assets::<Texture>,
    #[resource] input_events: &Events::<KeyboardInput>,
) {
    for event in &input_events.events {
        match event {
            KeyboardInput {
                state,
                virtual_keycode: Some(keycode),
                ..
            } => {
                let is_pressed = *state == ElementState::Pressed;
                match keycode {
                    VirtualKeyCode::Space => {
                        if is_pressed {
                            let index = {
                                let num_meshes = app_state.scene.meshes.len();
                                if num_meshes > 0 {
                                    if let Some(current_index) = app_state.current_mesh_index {
                                        Some((current_index + 1) % num_meshes)
                                    }
                                    else {
                                        None
                                    }
                                } else {
                                    None
                                }
                            };
                            app_state.set_current_mesh_index(index, wgpu_state, pipeline, textures, command_buffer);
                        }
                    },
                    _ => {},
                }
            },
            _ => {},
        }
    }
}
