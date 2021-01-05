use legion::*;
use legion::systems::CommandBuffer;
use rapier2d::{dynamics::{BodyStatus, RigidBody, RigidBodyBuilder, RigidBodySet}, geometry::{Collider, ColliderBuilder, ColliderSet}};
use crate::{
    loader,
    loader::{Scene, Component, RigidBodyStatus, Shape},
    asset::Assets,
    transform::Transform2D,
    events::Events,
    texture::Texture,
    mesh,
    wgpu_state::WgpuState,
    physics::{RigidBodyHandle, ColliderHandle},
};
use std::collections::HashMap;
use winit::event::{ElementState, KeyboardInput, VirtualKeyCode};

pub struct AppState {
    scene: Scene,
    mesh_indices: HashMap<String, usize>,
    rigid_body_indices: HashMap<String, usize>,
    collider_indices: HashMap<String, usize>,
    mesh_entities: Vec<Entity>,
    current_mesh_index: Option<usize>,
}

impl AppState {
    pub fn new() -> Self {
        let scene = loader::load_ron();
        let mut mesh_indices = HashMap::new();
        for (i, mesh) in scene.meshes.iter().enumerate() {
            mesh_indices.insert(mesh.mesh_name.clone(), i);
        }
        let mut rigid_body_indices = HashMap::new();
        for (i, rigid_body) in scene.rigid_bodies.iter().enumerate() {
            rigid_body_indices.insert(rigid_body.name.clone(), i);
        }
        let mut collider_indices = HashMap::new();
        for (i, collider) in scene.colliders.iter().enumerate() {
            collider_indices.insert(collider.name.clone(), i);
        }
        Self {
            scene: loader::load_ron(),
            mesh_entities: Vec::new(),
            current_mesh_index: None,
            mesh_indices,
            rigid_body_indices,
            collider_indices,
        }
    }
    
    pub fn remove_mesh_entities(&mut self, command_buffer: &mut CommandBuffer) {
        for entity in self.mesh_entities.drain(..) {
            command_buffer.remove(entity);
        }
    }

    pub fn set_current_mesh_index(
        &mut self,
        mesh_index: Option<usize>,
        wgpu_state: &mut WgpuState,
        pipeline: &mesh::Pipeline,
        textures: &mut Assets<Texture>,
        command_buffer: &mut CommandBuffer
    ) {
        if self.current_mesh_index == mesh_index { return; }
        self.current_mesh_index = mesh_index;
        info!("Set current mesh index to {:?}", self.current_mesh_index);

        self.remove_mesh_entities(command_buffer);

        if let Some(index) = self.current_mesh_index {
            let transform = Transform2D::new(&glam::Vec3::zero(), 0.0);
            let mesh = mesh::Mesh::new(wgpu_state, pipeline, textures, &self.scene.meshes[index]);
            let params = mesh::PipelineParams::new(wgpu_state, pipeline, &transform.build_matrix());
            let entity = command_buffer.push((transform, mesh, params));
            self.mesh_entities.push(entity);
        }
    }

    fn get_transform(components: &Vec<Component>) -> Option<Transform2D> {
        for component in components {
            if let Component::Transform { translation, rotation } = component {
                return Some(Transform2D::new(&glam::Vec3::new(translation.0, translation.1, 0.0), *rotation));
            }
        }
        None
    }

    fn get_rigid_body(&self, components: &Vec<Component>, transform: &Transform2D) -> Option<RigidBody> {
        for component in components {
            if let Component::RigidBody(name) = component {
                let rigid_body_index = self.rigid_body_indices.get(name).unwrap();
                let rigid_body_data = &self.scene.rigid_bodies[*rigid_body_index];
                let status = match rigid_body_data.status {
                    RigidBodyStatus::Static => BodyStatus::Static,
                    RigidBodyStatus::Dynamic => BodyStatus::Dynamic,
                    RigidBodyStatus::Kinematic => BodyStatus::Kinematic,
                };
                return Some(RigidBodyBuilder::new(status).translation(transform.translation.x / 64.0, transform.translation.y / 64.0).build());
            }
        }
        None
    }

    fn get_collider(&self, components: &Vec<Component>) -> Option<Collider> {
        for component in components {
            if let Component::Collider(name) = component {
                let collider_index = self.collider_indices.get(name).unwrap();
                let collider_data = &self.scene.colliders[*collider_index];
                if let Shape::Cuboid(hx, hy) = collider_data.shape {
                    return Some(ColliderBuilder::cuboid(hx / 64.0, hy / 64.0).build());
                }
            }
        }
        None
    }

    pub fn spawn_scene(
        &mut self,
        wgpu_state: &mut WgpuState,
        pipeline: &mesh::Pipeline,
        textures: &mut Assets<Texture>,
        rigid_body_set: &mut RigidBodySet,
        collider_set: &mut ColliderSet,
        command_buffer: &mut CommandBuffer
    ) {
        self.current_mesh_index = None;
        self.remove_mesh_entities(command_buffer);

        for node in &self.scene.scene_nodes {
            let entity = command_buffer.push(());

            // Add transform first because other components depend on it.
            let transform = AppState::get_transform(&node.components).unwrap();
            command_buffer.add_component(entity, transform.clone());

            // Add rigid body and collider.
            let rigid_body = self.get_rigid_body(&node.components, &transform);
            if let Some(rigid_body) = rigid_body {
                let rigid_body_handle = rigid_body_set.insert(rigid_body);
                command_buffer.add_component(entity, RigidBodyHandle(rigid_body_handle));
                let collider = self.get_collider(&node.components);
                if let Some(collider) = collider {
                    let collider_handle = collider_set.insert(collider, rigid_body_handle, rigid_body_set);
                    command_buffer.add_component(entity, ColliderHandle(collider_handle));
                }
            }

            // Add other components.
            for component in &node.components {
                match component {
                    Component::Mesh(mesh_name) => {
                        let mesh_index = self.mesh_indices.get(mesh_name).unwrap();
                        let mesh = mesh::Mesh::new(wgpu_state, pipeline, textures, &self.scene.meshes[*mesh_index]);
                        command_buffer.add_component(entity, mesh);
                        let params = mesh::PipelineParams::new(wgpu_state, pipeline, &transform.build_matrix());
                        command_buffer.add_component(entity, params);
                    }
                    _ => {}
                }
            }
            self.mesh_entities.push(entity);
        }
    }
}

#[system]
pub fn handle_input(
    command_buffer: &mut CommandBuffer,
    #[resource] app_state: &mut AppState,
    #[resource] wgpu_state: &mut WgpuState,
    #[resource] pipeline: &mesh::Pipeline,
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
                                        Some(0)
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
