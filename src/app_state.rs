use legion::*;
use rapier2d::{dynamics::{BodyStatus, RigidBodyBuilder, RigidBodySet, JointSet}, geometry::{ColliderBuilder, ColliderSet}};
use crate::{
    static_data::{DataAccessor, Component, RigidBodyStatus, Shape},
    asset::Assets,
    hierarchy::{Parent, Children},
    transform::{Transform2D, LocalTransform, GlobalTransform},
    events::Events,
    command::Command,
    texture::Texture,
    mesh,
    wgpu_state::WgpuState,
    physics::{RigidBodyHandle, ColliderHandle},
};
use std::{
    collections::HashMap,
    path::Path,
};
use smallvec::smallvec;
use winit::event::{ElementState, KeyboardInput, VirtualKeyCode};

pub struct AppState {
    pub data_accessor: DataAccessor,
    entities: Vec<Entity>,
    entity_indices: HashMap<String, usize>,
    pub root_entities: Vec<Entity>,
}

impl AppState {
    pub fn new() -> Self {
        let mut data_accessor = DataAccessor::new();
        data_accessor.load_collections_in_directory(&Path::new("data"));
        Self {
            data_accessor,
            entities: Vec::new(),
            entity_indices: HashMap::new(),
            root_entities: Vec::new(),
        }
    }
}

#[system]
pub fn handle_input(
    /*command_buffer: &mut CommandBuffer,
    #[resource] app_state: &mut AppState,
    #[resource] wgpu_state: &mut WgpuState,
    #[resource] pipeline: &mesh::Pipeline,
    #[resource] textures: &mut Assets::<Texture>,*/
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
                            /*let index = {
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
                            app_state.set_current_mesh_index(index, wgpu_state, pipeline, textures, command_buffer);*/
                        }
                    },
                    _ => {},
                }
            },
            _ => {},
        }
    }
}

pub fn handle_commands(world: &mut World, resources: &mut Resources) {
    let commands = resources.get_mut::<Events::<Command>>().unwrap().events.clone();
    for command in &commands {
        match command {
            Command::SetCurrentScene(index) => {
                set_current_scene(*index, world, resources);
            }
        }
    }
}

fn remove_all_entities(world: &mut World, resources: &mut Resources) {
    let mut app_state = resources.get_mut::<AppState>().unwrap();
    let mut rigid_body_set = resources.get_mut::<RigidBodySet>().unwrap();
    let mut collider_set = resources.get_mut::<ColliderSet>().unwrap();
    let mut joint_set = resources.get_mut::<JointSet>().unwrap();

    for entity in app_state.entities.drain(..) {
        if let Some(entry) = world.entry(entity) {
            if let Ok(rigid_body_handle) = entry.get_component::<RigidBodyHandle>() {
                rigid_body_set.remove(rigid_body_handle.0, &mut collider_set, &mut joint_set);
            }
            if let Ok(collider_handle) = entry.get_component::<ColliderHandle>() {
                collider_set.remove(collider_handle.0, &mut rigid_body_set, false);
            }
        }
        world.remove(entity);
    }

    app_state.entity_indices.clear();
    app_state.root_entities.clear();
}

fn set_current_scene(index: usize, world: &mut World, resources: &mut Resources) {
    remove_all_entities(world, resources);

    let mut root_nodes = Vec::new();
    {
        let app_state = resources.get::<AppState>().unwrap();
        let collection = &app_state.data_accessor.collections[index];

        info!("Set current collection to {}", index);

        for (i, node) in collection.scene_nodes.iter().enumerate() {
            if let None = node.get_parent() {
                root_nodes.push(i);
            }
        }
    }

    for node_index in root_nodes {
        spawn_entities_recursive(node_index, index, index, None, world, resources);
    }
}

fn spawn_entities_recursive(
    index: usize,
    source_collection: usize,
    collection_to_spawn_in: usize,
    parent_name: Option<String>,
    world: &mut World,
    resources: &mut Resources
) {
    let (entity_name, children) = {
        let mut app_state = resources.get_mut::<AppState>().unwrap();
        let wgpu_state = resources.get::<WgpuState>().unwrap();
        let pipeline = resources.get_mut::<mesh::Pipeline>().unwrap();
        let mut textures = resources.get_mut::<Assets<Texture>>().unwrap();
        let mut rigid_body_set = resources.get_mut::<RigidBodySet>().unwrap();
        let mut collider_set = resources.get_mut::<ColliderSet>().unwrap();

        let (node, children) = {
            let collection = &app_state.data_accessor.collections[source_collection];
            let node_name = &collection.scene_nodes[index].name;
            app_state.data_accessor.build_node(node_name, collection_to_spawn_in).unwrap()
        };

        let entity = world.push(());
        let mut entry = world.entry(entity).unwrap();

        let mut transform = None;
        let mut rigid_body_handle = None;

        // Components are sorted, so that they get created in the right order.
        for component in &node.components {
            match component {
                Component::Transform{ translation, rotation } => {
                    let new_transform = Transform2D::new(&glam::Vec3::new(translation.0, translation.1, 0.0), *rotation);
                    transform = Some(new_transform.clone());
                    entry.add_component(LocalTransform(new_transform.clone()));
                    entry.add_component(GlobalTransform(new_transform.clone()));
                }
                Component::Mesh(mesh_name) => {
                    let transform = transform.as_ref().unwrap();
                    let mesh_data = app_state.data_accessor.get_mesh(mesh_name, collection_to_spawn_in).unwrap();
                    let mesh = mesh::Mesh::new(&wgpu_state, &pipeline, &mut textures, mesh_data);
                    entry.add_component(mesh);
                    let params = mesh::PipelineParams::new(&wgpu_state, &pipeline, &transform.build_matrix());
                    entry.add_component(params);
                }
                Component::RigidBody(name) => {
                    let transform = transform.as_ref().unwrap();
                    let rigid_body_data = app_state.data_accessor.get_rigid_body(name, collection_to_spawn_in).unwrap();
                    let status = match rigid_body_data.status {
                        RigidBodyStatus::Static => BodyStatus::Static,
                        RigidBodyStatus::Dynamic => BodyStatus::Dynamic,
                        RigidBodyStatus::Kinematic => BodyStatus::Kinematic,
                    };
                    let rigid_body =
                        RigidBodyBuilder::new(status).translation(transform.translation.x / 64.0, transform.translation.y / 64.0).build();
                    let new_rigid_body_handle = rigid_body_set.insert(rigid_body);
                    rigid_body_handle = Some(new_rigid_body_handle);
                    entry.add_component(RigidBodyHandle(new_rigid_body_handle));
                }
                Component::Collider(name) => {
                    let collider_data = app_state.data_accessor.get_collider(name, collection_to_spawn_in).unwrap();
                    if let Shape::Cuboid(hx, hy) = collider_data.shape {
                        let collider = ColliderBuilder::cuboid(hx / 64.0, hy / 64.0).build();
                        let collider_handle = collider_set.insert(collider, rigid_body_handle.unwrap(), &mut rigid_body_set);
                        entry.add_component(ColliderHandle(collider_handle));
                    }
                }
                _ => {}
            }
        }

        let entity_name = if source_collection == collection_to_spawn_in {
            node.name.clone()
        } else {
            let mut name = parent_name.as_ref().unwrap().clone();
            name.push_str(&node.name);
            name
        };
        let entity_index = app_state.entities.len();
        app_state.entity_indices.insert(entity_name.clone(), entity_index);
        app_state.entities.push(entity);

        if let Some(parent_name) = parent_name {
            let parent_index = app_state.entity_indices.get(&parent_name).unwrap();
            let parent = app_state.entities[*parent_index];
            entry.add_component(Parent(parent));
            let mut parent_entry = world.entry(parent).unwrap();
            if let Ok(children) = parent_entry.get_component_mut::<Children>() {
                children.0.push(entity);
            }
            else {
                parent_entry.add_component(Children(smallvec![entity]));
            }
        }
        else {
            app_state.root_entities.push(entity);
        }
        (entity_name.clone(), children)
    };

    for (child_index, collection) in &children {
        spawn_entities_recursive(*child_index, *collection, collection_to_spawn_in, Some(entity_name.clone()), world, resources);
    }
}
