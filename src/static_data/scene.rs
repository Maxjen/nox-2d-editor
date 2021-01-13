use std::cmp::Ordering;
use serde::{Serialize, Deserialize};
use smallvec::SmallVec;

#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct SceneNode {
    pub name: String,

    #[serde(default)]
    pub prefab: String,

    pub components: Vec<Component>,

    #[serde(skip)]
    pub children: SmallVec<[usize; 8]>,
}

impl SceneNode {
    pub fn get_parent(&self) -> Option<&String> {
        for component in &self.components {
            if let Component::Parent(name) = component {
                return Some(name);
            }
        }
        None
    }

    pub fn get_parent_mut(&mut self) -> Option<&mut Component> {
        for component in &mut self.components {
            if let Component::Parent(_) = component {
                return Some(component);
            }
        }
        None
    }

    pub fn set_parent(&mut self, parent_name: String) {
        if let Some(Component::Parent(name)) = self.get_parent_mut() {
            *name = parent_name;
        }
        else {
            self.components.push(Component::Parent(parent_name));
        }
    }

    pub fn get_transform_mut(&mut self) -> Option<&mut Component> {
        for component in &mut self.components {
            if let Component::Transform{..} = component {
                return Some(component);
            }
        }
        None
    }

    pub fn set_transform(&mut self, new_translation: &(f32, f32), new_rotation: f32) {
        if let Some(Component::Transform{ translation, rotation }) = self.get_transform_mut() {
            *translation = *new_translation;
            *rotation = new_rotation;
        }
        else {
            self.components.push(Component::Transform { translation: *new_translation, rotation: new_rotation });
        }
    }

    pub fn get_mesh_mut(&mut self) -> Option<&mut Component> {
        for component in &mut self.components {
            if let Component::Mesh(_) = component {
                return Some(component);
            }
        }
        None
    }

    pub fn set_mesh(&mut self, mesh_name: String) {
        if let Some(Component::Mesh(name)) = self.get_mesh_mut() {
            *name = mesh_name
        }
        else {
            self.components.push(Component::Mesh(mesh_name));
        }
    }

    pub fn get_rigid_body_mut(&mut self) -> Option<&mut Component> {
        for component in &mut self.components {
            if let Component::RigidBody(_) = component {
                return Some(component);
            }
        }
        None
    }

    pub fn set_rigid_body(&mut self, rigid_body_name: String) {
        if let Some(Component::RigidBody(name)) = self.get_rigid_body_mut() {
            *name = rigid_body_name
        }
        else {
            self.components.push(Component::RigidBody(rigid_body_name));
        }
    }
    
    pub fn get_collider_mut(&mut self) -> Option<&mut Component> {
        for component in &mut self.components {
            if let Component::Collider(_) = component {
                return Some(component);
            }
        }
        None
    }

    pub fn set_collider(&mut self, collider_name: String) {
        if let Some(Component::Collider(name)) = self.get_collider_mut() {
            *name = collider_name
        }
        else {
            self.components.push(Component::Collider(collider_name));
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Component {
    Transform { translation: (f32, f32), rotation: f32 },
    Parent(String),
    Mesh(String),
    RigidBody(String),
    Collider(String),
}

pub fn compare_components(component_1: &Component, component_2: &Component) -> Ordering {
    let get_priority = |component: &Component| {
        match component {
            Component::Transform{..} => 0,
            Component::Parent(_) => 1,
            Component::Mesh(_) => 2,
            Component::RigidBody(_) => 3,
            Component::Collider(_) => 4,
        }
    };
    let priority_1 = get_priority(component_1);
    let priority_2 = get_priority(component_2);
    if priority_1 < priority_2 {
        Ordering::Less
    }
    else if priority_1 > priority_2 {
        Ordering::Greater
    }
    else
    {
        Ordering::Equal
    }
}
